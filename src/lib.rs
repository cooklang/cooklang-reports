//! A Rust library for generating reports from [Cooklang][00] recipes using [Jinja2][01]-style templates.
//!
//! Templates are provided with multiple context variables:
//!
//! - `scale`: a float representing the recipe scaling factor (i.e. 1 by default)
//! - `sections`: the sections, containing steps and text, within the recipe
//! - `ingredients`: the list of ingredients in the recipe
//! - `cookware`: the list of cookware pieces in the recipe
//! - `metadata`: the dictionary of metadata from the recipe
//!
//! For more details about each of these, look through the source for the `models` module`.`
//!
//! [00]: https://cooklang.org/
//! [01]: https://jinja.palletsprojects.com/en/stable/
#[doc = include_str!("../README.md")]
use config::Config;
use cooklang::Recipe;
use filters::{format_price_filter, numeric_filter};
use functions::{get_from_datastore, get_ingredient_list};
use minijinja::Environment;
use model::{Cookware, Ingredient, Metadata, Section};
use parser::{get_converter, get_parser};
use serde::Serialize;
use yaml_datastore::Datastore;

pub mod config;
pub mod error;
mod filters;
mod functions;
mod model;
pub mod parser;

pub use error::Error;

/// Context passed to the template
#[derive(Debug, Serialize)]
struct TemplateContext {
    scale: f64,
    datastore: Option<Datastore>,
    base_path: Option<String>,
    sections: Vec<minijinja::Value>,
    ingredients: Vec<minijinja::Value>,
    cookware: Vec<minijinja::Value>,
    metadata: minijinja::Value,
}

impl TemplateContext {
    fn new(
        recipe: Recipe,
        scale: f64,
        datastore: Option<Datastore>,
        base_path: Option<String>,
    ) -> TemplateContext {
        TemplateContext {
            scale,
            datastore,
            base_path,
            sections: Section::from_recipe_sections(&recipe)
                .into_iter()
                .map(minijinja::Value::from_object)
                .collect(),
            ingredients: recipe
                .ingredients
                .into_iter()
                .map(Ingredient::from)
                .map(minijinja::Value::from)
                .collect(),
            cookware: recipe
                .cookware
                .into_iter()
                .map(Cookware::from)
                .map(minijinja::Value::from)
                .collect(),
            metadata: Metadata::from(recipe.metadata).into(),
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
    // Parse and validate recipe string using global parser
    let (mut recipe, warnings) = get_parser().parse(recipe).into_result()?;

    // Log warnings if present
    if warnings.has_warnings() {
        for warning in warnings.warnings() {
            eprintln!("Warning: {warning}");
        }
    }

    // Scale the recipe using global converter
    recipe.scale(config.scale, get_converter());
    let datastore = config.datastore_path.as_ref().map(Datastore::open);
    let base_path = config
        .base_path
        .as_ref()
        .and_then(|p| p.to_str())
        .map(String::from);

    let template_context = TemplateContext::new(recipe, config.scale, datastore, base_path);
    let template_environment = template_environment(template)?;

    let template: minijinja::Template<'_, '_> = template_environment.get_template("base")?;
    Ok(template.render(template_context)?)
}

/// Build an environment for the given template.
fn template_environment(template: &str) -> Result<Environment<'_>, Error> {
    let mut env = Environment::new();

    // Enable debug mode for better error messages
    env.set_debug(true);

    env.add_template("base", template)?;
    env.add_function("db", get_from_datastore);
    env.add_function("get_ingredient_list", get_ingredient_list);
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
    fn test_recursive_ingredients_with_base_path() {
        let base_path = get_test_data_path().join("recipes");

        // Use the actual Recipe With Reference.cook file
        let recipe_path = base_path.join("Recipe With Reference.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        let template = indoc! {"
            # Recursive Ingredients
            {%- set all = get_ingredient_list(ingredients) %}
            {%- for ingredient in all %}
            - {{ ingredient.name }}: {{ ingredient.quantities }}
            {%- endfor %}
        "};

        let config = Config::builder().base_path(&base_path).build();

        let result = render_template_with_config(&recipe, template, &config).unwrap();

        // Recipe With Reference.cook contains:
        // - @Pancakes.cook{2} - should be expanded to Pancakes ingredients scaled by 2
        // - @sugar{2%tbsp}
        // - @milk{200%ml}
        // Pancakes.cook contains: @eggs{3%large}, @milk{250%ml}, @flour{125%g}
        // With scaling of 2: eggs: 6 large, milk: 500 ml (plus 200 ml from direct), flour: 250 g
        // Combined ingredients should merge milk quantities
        let expected = indoc! {"
            # Recursive Ingredients
            - eggs: 6 large
            - flour: 250 g
            - milk: 700 ml
            - sugar: 2 tbsp"};

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
            - {{ ingredient.name }}: {{ ingredient.quantity }}
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
    fn metadata_render() {
        // Use Pancakes.cook from test data
        let recipe_path = get_test_data_path()
            .join("recipes")
            .join("Chinese Udon Noodles.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        let template: &str = indoc! {"
            # Metadata
            {{ metadata }}
        "};

        let result = render_template(&recipe, template).unwrap();
        let expected = indoc! {"
            # Metadata
            ---
            title: Chinese-Style Udon Noodles
            description: A quick, simple, yet satisfying take on a Chinese-style noodle dish.
            author: Dan Fego
            servings: 2
            tags:
            - vegan
            ---
            "};
        assert_eq!(result, expected);
    }

    #[test]
    fn metadata_enumerate() {
        // Use Pancakes.cook from test data
        let recipe_path = get_test_data_path()
            .join("recipes")
            .join("Chinese Udon Noodles.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        let template: &str = indoc! {"
            # Metadata
            {%- for key, value in metadata | items %}
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
    fn test_template_syntax_error() {
        let recipe = "@eggs{2}";
        let template = "{% for item in ingredients %}{{ item.name }}{% endfor"; // Missing %}

        let result = render_template(recipe, template);
        assert!(result.is_err());

        if let Err(e) = result {
            let formatted = e.format_with_source();
            // Check for enhanced error display features
            assert!(formatted.contains("syntax error"));
            assert!(formatted.contains("endfor")); // The problematic token
            assert!(formatted.contains("Hint:")); // Our helpful hints
            assert!(formatted.contains("Missing closing tags"));
        }
    }

    #[test]
    fn test_template_undefined_error() {
        let recipe = "@eggs{2}";
        let template = "{{ nonexistent_variable }}";

        let result = render_template(recipe, template);
        // Undefined variables render as empty strings by default in minijinja
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_template_attribute_error() {
        let recipe = "@eggs{2}";
        let template = "{% for item in ingredients %}{{ item.nonexistent }}{% endfor %}";

        let result = render_template(recipe, template);
        // Undefined attributes also render as empty by default
        assert!(result.is_ok());
    }

    #[test]
    fn test_template_invalid_function_call() {
        let recipe = "@eggs{2}";
        let template = "{{ unknown_function() }}";

        let result = render_template(recipe, template);
        assert!(result.is_err());

        if let Err(e) = result {
            let formatted = e.format_with_source();
            // Check for enhanced error display
            assert!(formatted.contains("unknown function"));
            assert!(formatted.contains("unknown_function()")); // The problematic expression
        }
    }

    #[test]
    fn test_recipe_references_with_servings_scaling() {
        let base_path = get_test_data_path().join("recipes");

        // Load the recipe with scaled references
        let recipe_path = base_path.join("Recipe With Scaled References.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        let template = indoc! {"
            # All Ingredients
            {%- set all = get_ingredient_list(ingredients) %}
            {%- for ingredient in all %}
            - {{ ingredient.name }}: {{ ingredient.quantities }}
            {%- endfor %}
        "};

        let config = Config::builder().base_path(&base_path).build();
        let result = render_template_with_config(&recipe, template, &config).unwrap();

        // Recipe With Servings has 4 servings, requesting 8 servings = 2x scale
        // Original: flour 200g, milk 300ml, eggs 2
        // Scaled 2x: flour 400g, milk 600ml, eggs 4

        // Recipe With Yield yields 500g, requesting 250g = 0.5x scale
        // Original: butter 100g, sugar 150g, flour 250g
        // Scaled 0.5x: butter 50g, sugar 75g, flour 125g

        // Pancakes scaled by 2x directly
        // Original: eggs 3, milk 250ml, flour 125g
        // Scaled 2x: eggs 6, milk 500ml, flour 250g

        // Combined:
        // - butter: 50g
        // - eggs: 6 large (from Pancakes), 4 (from Recipe With Servings)
        //   Note: these don't merge because units differ
        // - flour: 400g + 125g + 250g = 775g
        // - milk: 600ml + 500ml = 1100ml
        // - salt: 1 tsp
        // - sugar: 75g

        let expected = indoc! {"
            # All Ingredients
            - butter: 50 g
            - eggs: 6 large, 4
            - flour: 775 g
            - milk: 1100 ml
            - salt: 1 tsp
            - sugar: 75 g"};

        assert_eq!(result, expected);
    }

    #[test]
    fn test_recipe_references_yield_unit_mismatch() {
        let base_path = get_test_data_path().join("recipes");

        // Create a recipe that requests wrong units
        let recipe = indoc! {"
            ---
            title: Bad Yield Reference
            ---

            Make @./Recipe With Yield.cook{100%ml} incorrectly.
        "};

        let template = indoc! {"
            {%- set all = get_ingredient_list(ingredients) %}
            Error should happen before this
        "};

        let config = Config::builder().base_path(&base_path).build();
        let result = render_template_with_config(recipe, template, &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.format_with_source();
        assert!(
            err_msg.contains("Failed to scale recipe"),
            "Expected error about scaling recipe, got: {err_msg}"
        );
    }

    #[test]
    fn test_recipe_references_missing_servings() {
        let base_path = get_test_data_path().join("recipes");

        // Create a recipe without servings metadata
        let no_servings_path = base_path.join("No Servings.cook");
        std::fs::write(&no_servings_path, "Mix @flour{100%g} with @water{200%ml}.").unwrap();

        let recipe = indoc! {"
            ---
            title: Bad Servings Reference
            ---

            Make @./No Servings.cook{4%servings} incorrectly.
        "};

        let template = indoc! {"
            {%- set all = get_ingredient_list(ingredients) %}
            Error should happen before this
        "};

        let config = Config::builder().base_path(&base_path).build();
        let result = render_template_with_config(recipe, template, &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.format_with_source();
        assert!(
            err_msg.contains("Failed to scale recipe") && err_msg.contains("servings"),
            "Expected error about missing servings metadata, got: {err_msg}"
        );

        // Clean up
        std::fs::remove_file(no_servings_path).ok();
    }

    #[test]
    fn test_recipe_references_missing_yield() {
        let base_path = get_test_data_path().join("recipes");

        // Pancakes doesn't have yield metadata
        let recipe = indoc! {"
            ---
            title: Bad Yield Reference
            ---

            Make @./Pancakes.cook{500%g} incorrectly.
        "};

        let template = indoc! {"
            {%- set all = get_ingredient_list(ingredients) %}
            Error should happen before this
        "};

        let config = Config::builder().base_path(&base_path).build();
        let result = render_template_with_config(recipe, template, &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.format_with_source();
        assert!(
            err_msg.contains("Failed to scale recipe"),
            "Expected error about scaling recipe, got: {err_msg}"
        );
    }

    #[test]
    fn test_recursive_ingredients_without_expansion() {
        let base_path = get_test_data_path().join("recipes");

        // Use the actual Recipe With Reference.cook file
        let recipe_path = base_path.join("Recipe With Reference.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        // Test with expand_references = false
        let template = indoc! {"
            # Non-Recursive Ingredients
            {%- set all = get_ingredient_list(ingredients, false) %}
            {%- for ingredient in all %}
            - {{ ingredient.name }}: {{ ingredient.quantities }}
            {%- endfor %}
        "};

        let config = Config::builder().base_path(&base_path).build();

        let result = render_template_with_config(&recipe, template, &config).unwrap();

        // When not expanding references, Recipe With Reference.cook contains:
        // - @./Pancakes{2} - should remain as "Pancakes" with quantity 2
        // - @sugar{2%tbsp}
        // - @milk{200%ml}
        let expected = indoc! {"
            # Non-Recursive Ingredients
            - Pancakes: 2
            - milk: 200 ml
            - sugar: 2 tbsp"};

        assert_eq!(result, expected);
    }

    #[test]
    fn test_base_path_defaults_to_cwd() {
        // Test that base_path always defaults to current working directory
        let config_default = Config::default();
        assert!(config_default.base_path.is_some());
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(config_default.base_path.unwrap(), cwd);

        let config_built = Config::builder().scale(2.0).build();
        // After building, base_path should still be set to current working directory
        assert!(config_built.base_path.is_some());
        assert_eq!(config_built.base_path.unwrap(), cwd);
    }

    #[test]
    fn sections_with_text() {
        let recipe_path = get_test_data_path().join("recipes").join("Blog Post.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        // I hate the nesting in this template but I couldn't get the whitespace
        // modifiers to work the way I want. I hate jinja whitespace.
        let template: &str = indoc! {"
        {%- for section in sections -%}
        {{ section }}
        {%- endfor -%}\n
        "};

        let result = render_template(&recipe, template).unwrap();
        let expected = indoc! {"
        = My Life Story

        This is a blog post about something.

        It has many paragraphs.

        = Recipe

        Nope, just kidding.

        "};
        assert_eq!(result, expected);
    }

    #[test]
    fn one_section_with_steps() {
        let recipe = indoc! {"
        Put @butter{1%pat} into #frying pan{} on low heat.

        Crack @egg into pan.

        Fry egg on low heat until cooked.

        Enjoy.
        "};

        let template: &str = indoc! {"
            # Steps
            {% for content in sections[0] %}
            {{ content }}
            {%- endfor %}
        "};

        let result = render_template(recipe, template).unwrap();
        println!("{result}");
        let expected = indoc! {"
            # Steps

            1. Put 1 pat butter into frying pan on low heat.
            2. Crack egg into pan.
            3. Fry egg on low heat until cooked.
            4. Enjoy."};
        assert_eq!(result, expected);
    }
}
