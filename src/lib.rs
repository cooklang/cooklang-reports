//! A Rust library for generating reports from [Cooklang][00] recipes using [Jinja2][01]-style templates.
//!
//! The template format is not yet fully documented.
//! Look at the tests in the repository for examples.
//!
//! [00]: https://cooklang.org/
//! [01]: https://jinja.palletsprojects.com/en/stable/
#[doc = include_str!("../README.md")]
use config::Config;
use cooklang::{Converter, CooklangParser, Cookware, Extensions, Metadata, ScaledRecipe, Section};
use filters::{format_price_filter, numeric_filter};
use functions::get_from_datastore;
use minijinja::Environment;
use model::Ingredient;
use serde::Serialize;
use thiserror::Error;
use yaml_datastore::Datastore;

pub mod config;
mod filters;
mod functions;
mod model;

/// Error type for this crate.
#[derive(Error, Debug)]
pub enum Error {
    /// An error occurred when parsing the recipe.
    #[error("error parsing recipe")]
    RecipeParseError(#[from] cooklang::error::SourceReport),

    /// An error occurred when generating a report from a template.
    #[error("template error")]
    TemplateError(#[from] minijinja::Error),
}

/// Context passed to the template
#[derive(Debug, Serialize)]
struct TemplateContext<'a> {
    scale: f64,
    datastore: Option<Datastore>,
    sections: &'a Vec<Section>,
    ingredients: Vec<Ingredient<'a>>,
    cookware: &'a Vec<Cookware>,
    metadata: &'a Metadata,
}

impl TemplateContext<'_> {
    fn new(recipe: &ScaledRecipe, scale: f64, datastore: Option<Datastore>) -> TemplateContext {
        TemplateContext {
            scale,
            datastore,
            sections: &recipe.sections,
            ingredients: recipe.ingredients.iter().map(Ingredient::from).collect(),
            cookware: &recipe.cookware,
            metadata: &recipe.metadata,
        }
    }
}

/// Render a recipe with the deault configuration.
///
/// This is equivalent to calling [`render_template_with_config`] with a default [`Config`].
///
/// # Errors
///
/// Returns [`RecipeParseError`][`Error::RecipeParseError`] if the recipe cannot be parsed by the
/// [`CooklangParser`][`cooklang::CooklangParser`].
///
/// Returns [`TemplateError`][`Error::TemplateError`] if the template has a syntax error or rendering fails.
pub fn render_template(recipe: &str, template: &str) -> Result<String, Error> {
    render_template_with_config(recipe, template, &Config::default())
}

/// Render a recipe to a String with the provided [`Config`].
///
/// On success, returns a String with the recipe as rendered by the template.
///
/// # Parameters
///
/// * `recipe` is a (hopefully valid) cooklang recipe as a string, ready to be parsed.
/// * `template` is a (hopefully valid) template. Format will be documented in the future.
/// * `config` is a [`Config`][`config::Config`] with options for rendering the recipe.
///
/// # Errors
///
/// Returns [`RecipeParseError`][`Error::RecipeParseError`] if the recipe cannot be parsed by the
/// [`CooklangParser`][`cooklang::CooklangParser`].
///
/// Returns [`TemplateError`][`Error::TemplateError`] if the template has a syntax error or rendering fails.
pub fn render_template_with_config(
    recipe: &str,
    template: &str,
    config: &Config,
) -> Result<String, Error> {
    // Parse and validate recipe string
    let recipe_parser = CooklangParser::new(Extensions::all(), Converter::default());
    let (unscaled_recipe, _warnings) = recipe_parser.parse(recipe).into_result()?;

    // Create final, scaled recipes
    let recipe = unscaled_recipe.scale(config.scale, &Converter::default());
    let datastore = config.datastore_path.as_ref().map(Datastore::open);

    let template_context = TemplateContext::new(&recipe, config.scale, datastore);
    let template_environment = template_environment(template)?;

    let template: minijinja::Template<'_, '_> = template_environment.get_template("base")?;
    Ok(template.render(template_context)?)
}

/// Build an environment for the given template.
fn template_environment(template: &str) -> Result<Environment<'_>, Error> {
    let mut env = Environment::new();
    env.add_template("base", template)?;
    env.add_function("db", get_from_datastore);
    env.add_filter("numeric", numeric_filter);
    env.add_filter("format_price", format_price_filter);
    Ok(env)
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use std::path::PathBuf;

    fn get_test_data_path() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test");
        path.push("data");
        path
    }

    #[test]
    fn simple_template_new() {
        // Use Pancakes.cook from test data
        let recipe_path = get_test_data_path().join("recipes").join("Pancakes.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        let template: &str = indoc! {"
            # Ingredients ({{ scale }}x)
            {%- for ingredient in ingredients %}
            - {{ ingredient.name }}
            {%- endfor %}
        "};

        // Test default scaling (1x)
        let result = render_template(&recipe, template).unwrap();
        let expected = indoc! {"
            # Ingredients (1.0x)
            - eggs
            - milk
            - flour"};
        assert_eq!(result, expected);

        // Test with 2x scaling, but only for the actual scale number
        let config: Config = Config::builder().scale(2.0).build();
        let result = render_template_with_config(&recipe, template, &config).unwrap();
        let expected = indoc! {"
            # Ingredients (2.0x)
            - eggs
            - milk
            - flour"};
        assert_eq!(result, expected);
    }

    #[test]
    fn test_datastore_access() {
        let datastore_path = get_test_data_path().join("db");

        // Use Pancakes.cook from test data
        let recipe_path = get_test_data_path().join("recipes").join("Pancakes.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        let template = indoc! {"
            # Eggs Info

            Density: {{ db('eggs.meta.density') }}
            Shelf Life: {{ db('eggs.meta.storage.shelf life') }} days
            Fridge Life: {{ db('eggs.meta.storage.fridge life') }} days
        "};

        let config = Config::builder().datastore_path(&datastore_path).build();
        let result = render_template_with_config(&recipe, template, &config).unwrap();
        let expected = indoc! {"
            # Eggs Info

            Density: 1.03
            Shelf Life: 30 days
            Fridge Life: 60 days"};

        assert_eq!(result, expected);
    }

    #[test]
    fn test_recipe_scaling() {
        // Use Pancakes.cook from test data
        let recipe_path = get_test_data_path().join("recipes").join("Pancakes.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        let template = indoc! {"
            # Ingredients ({{ scale }}x)
            {%- for ingredient in ingredients %}
            - {{ ingredient.name }}{% if ingredient.quantity %}: {{ ingredient.quantity }}{% if ingredient.unit %} {{ ingredient.unit }}{% endif %}{% endif %}
            {%- endfor %}
        "};

        // Test default scaling (1x)
        let result = render_template(&recipe, template).unwrap();
        let expected = indoc! {"
            # Ingredients (1.0x)
            - eggs: 3 large
            - milk: 250 ml
            - flour: 125 g"};
        assert_eq!(result, expected);

        // Test with 2x scaling
        let config = Config::builder().scale(2.0).build();
        let result = render_template_with_config(&recipe, template, &config).unwrap();
        let expected = indoc! {"
            # Ingredients (2.0x)
            - eggs: 6 large
            - milk: 500 ml
            - flour: 250 g"};
        assert_eq!(result, expected);

        // Test with 3x scaling
        let config = Config::builder().scale(3.0).build();
        let result = render_template_with_config(&recipe, template, &config).unwrap();
        let expected = indoc! {"
            # Ingredients (3.0x)
            - eggs: 9 large
            - milk: 750 ml
            - flour: 375 g"};
        assert_eq!(result, expected);

        // Test with 0.5x scaling
        let config = Config::builder().scale(0.5).build();
        let result = render_template_with_config(&recipe, template, &config).unwrap();
        let expected = indoc! {"
            # Ingredients (0.5x)
            - eggs: 1.5 large
            - milk: 125 ml
            - flour: 62.5 g"};
        assert_eq!(result, expected);
    }

    #[test]
    fn test_with_template_from_files() {
        // Use Pancakes.cook from test data
        let recipe_path = get_test_data_path().join("recipes").join("Pancakes.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        let template_path = get_test_data_path()
            .join("reports")
            .join("ingredients.md.jinja");
        let template = std::fs::read_to_string(template_path).unwrap();

        let result = render_template(&recipe, &template).unwrap();
        let expected = indoc! {"
            # Ingredients Report

            * eggs: 3 large
            * flour: 125 g
            * milk: 250 ml"};

        assert_eq!(result, expected);
    }

    #[test]
    fn test_with_template_from_files_with_db() {
        // Use Pancakes.cook from test data
        let recipe_path = get_test_data_path().join("recipes").join("Pancakes.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        // Use database path from test data
        let datastore_path = get_test_data_path().join("db");

        let template_path = get_test_data_path().join("reports").join("cost.md.jinja");
        let template = std::fs::read_to_string(template_path).unwrap();

        let config = Config::builder().datastore_path(datastore_path).build();
        let result = render_template_with_config(&recipe, &template, &config).unwrap();

        // Verify the report structure and content
        let expected = indoc! {"
            # Cost Report

            * eggs: $0.75
            * milk: $0.25
            * flour: $0.19

            Total: $1.19"};

        assert_eq!(result, expected);
    }

    #[test]
    fn cookware() {
        // Use Pancakes.cook from test data
        let recipe_path = get_test_data_path().join("recipes").join("Pancakes.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        let template: &str = indoc! {"
            # Cookware
            {%- for item in cookware %}
            - {{ item.name }}
            {%- endfor %}
        "};

        // Test default scaling (1x)
        let result = render_template(&recipe, template).unwrap();
        let expected = indoc! {"
            # Cookware
            - whisk
            - large bowl"};
        assert_eq!(result, expected);

        // TODO scaling? should it? No, right?
    }

    #[test]
    fn metadata() {
        // Use Pancakes.cook from test data
        let recipe_path = get_test_data_path()
            .join("recipes")
            .join("Chinese Udon Noodles.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        let template: &str = indoc! {"
            # Metadata
            {%- for key, value in metadata.map | items %}
            - {{ key }}: {{ value }}
            {%- endfor %}
        "};

        let result = render_template(&recipe, template).unwrap();
        let expected = indoc! {"
            # Metadata
            - title: Chinese-Style Udon Noodles
            - description: A quick, simple, yet satisfying take on a Chinese-style noodle dish.
            - author: Dan Fego
            - servings: 2
            - tags: [\"vegan\"]"};
        assert_eq!(result, expected);
    }

    #[test]
    fn sections() {
        let recipe_path = get_test_data_path()
            .join("recipes")
            .join("Contrived Eggs.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        let template: &str = indoc! {"
            # Recipe
            {%- for section in sections %}
            ## {{ section.name }}
            {%- endfor %}
        "};

        let result = render_template(&recipe, template).unwrap();
        let expected = indoc! {"
            # Recipe
            ## Preparation
            ## Cooking
            ## Consumption"};
        assert_eq!(result, expected);
    }

    #[test]
    fn sections_default() {
        let recipe_path = get_test_data_path().join("recipes").join("Pancakes.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        let template: &str = indoc! {"
            # Recipe
            {%- for section in sections %}
            {% if section.name %}
            ## {{ section.name }}
            {% endif %}
            {%- endfor %}
        "};

        let result = render_template(&recipe, template).unwrap();
        let expected = indoc! {"
        # Recipe
        "};
        assert_eq!(result, expected);
    }

    #[test]
    fn sections_with_text() {
        let recipe_path = get_test_data_path().join("recipes").join("Blog Post.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        // I hate the nesting in this template but I couldn't get the whitespace
        // modifiers to work the way I want. I hate jinja whitespace.
        let template: &str = indoc! {"
        # Blog
        {%- for section in sections %}
        {% if section.name %}
        ## {{ section.name }}
        {% endif %}
        {% for text in section.content -%}
        {{ text.value }}
        {%- if not loop.last %}

        {% endif -%}
        {% endfor -%}
        {%- endfor %}\n
        "};

        let result = render_template(&recipe, template).unwrap();
        println!("{result}");
        let expected = indoc! {"
        # Blog

        ## My Life Story

        This is a blog post about something.

        It has many paragraphs.

        ## Recipe

        Nope, just kidding.
        "};
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn sections_with_steps() {
        let recipe_path = get_test_data_path()
            .join("recipes")
            .join("Contrived Eggs.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        let template: &str = indoc! {"
            # Steps
            {%- for step in steps %}
              {{ step }}
            {%- endfor %}
        "};

        let result = render_template(&recipe, template).unwrap();
        let expected = indoc! {"
            # Steps
            1. Put butter into frying pan on low heat.
            2. Crack egg into pan.
            3. Fry egg on low heat until cooked.
            4. Enjoy."};
        assert_eq!(result, expected);
    }
}
