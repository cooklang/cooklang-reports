//! A Rust library for generating reports from [Cooklang][00] recipes using [Jinja2][01]-style templates.
//!
//! The template format is not yet fully documented.
//! Look at the tests in the repository for examples.
//!
//! [00]: https://cooklang.org/
//! [01]: https://jinja.palletsprojects.com/en/stable/
#[doc = include_str!("../README.md")]
use config::Config;
use cooklang::{Converter, CooklangParser, Extensions, ScaledRecipe};
use filters::{format_price_filter, numeric_filter};
use functions::get_from_datastore;
use minijinja::Environment;
use serde::Serialize;
use thiserror::Error;
use yaml_datastore::Datastore;

pub mod config;
mod filters;
mod functions;

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

/// An Ingredient that's used here instead of the parser's one, for template access.
#[derive(Serialize)]
struct Ingredient {
    name: String,
    quantity: Option<String>,
    note: Option<String>,
}

/// Return a vector of [`Ingredient`] for placing in [`RecipeContext`]
fn recipe_ingredients(recipe: &ScaledRecipe) -> Vec<Ingredient> {
    recipe
        .ingredients
        .iter()
        .map(|ingredient| Ingredient {
            name: ingredient.name.to_string(),
            quantity: ingredient.quantity.as_ref().map(ToString::to_string),
            note: ingredient.note.clone(),
        })
        .collect()
}

/// Context passed to the template.
///
/// The entire recipe is in here at this moment, flattened, for easy access to its fields.
#[derive(Serialize)]
struct RecipeContext {
    #[serde(flatten)]
    recipe: ScaledRecipe,
    scale: f64,
    datastore: Option<Datastore>,
    ingredients: Vec<Ingredient>,
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

    // Create final, scaled recipe
    let converter = Converter::default();
    let recipe = unscaled_recipe.scale(config.scale, &converter);

    let mut template_environment = Environment::new();
    template_environment.add_template("base", template)?;
    template_environment.add_function("db", get_from_datastore);
    template_environment.add_filter("numeric", numeric_filter);
    template_environment.add_filter("format_price", format_price_filter);

    let datastore = config.datastore_path.as_ref().map(Datastore::open);
    let ingredients = recipe_ingredients(&recipe);

    let context = RecipeContext {
        recipe,
        scale: config.scale,
        datastore,
        ingredients,
    };

    let tmpl = template_environment.get_template("base")?;
    Ok(tmpl.render(context)?)
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
}
