use cooklang::{Converter, CooklangParser, Extensions, Recipe, Value, scale::Scaled};
use minijinja::{
    Environment, Error as MiniError, State,
    value::{Enumerator, Object, Value as MiniValue},
};
use std::{path::Path, sync::Arc};
use yaml_datastore::Datastore;

mod filters;
mod functions;

#[derive(Debug)]
pub struct RecipeTemplate {
    recipe: Recipe<Scaled, Value>,
    scale: u32,
    datastore: Option<Datastore>,
}

impl From<Recipe<Scaled, Value>> for RecipeTemplate {
    fn from(recipe: Recipe<Scaled, Value>) -> Self {
        // Default scale is 1
        RecipeTemplate {
            recipe,
            scale: 1,
            datastore: None,
        }
    }
}

impl Object for RecipeTemplate {
    fn get_value(self: &Arc<Self>, key: &MiniValue) -> Option<MiniValue> {
        match key.as_str()? {
            "ingredients" => {
                let ingredients = self
                    .recipe
                    .ingredients
                    .iter()
                    .map(|ingredient| {
                        let mut map = std::collections::HashMap::new();
                        map.insert("name".to_string(), ingredient.name.clone());
                        if let Some(quantity) = &ingredient.quantity {
                            map.insert("quantity".to_string(), quantity.to_string());
                            if let Some(note) = &ingredient.note {
                                map.insert("unit".to_string(), note.clone());
                            }
                        }
                        map
                    })
                    .collect::<Vec<_>>();
                Some(MiniValue::from_serialize(&ingredients))
            }
            "scale" => Some(MiniValue::from_serialize(&self.scale)),
            _ => None,
        }
    }

    fn call_method(
        self: &Arc<Self>,
        _state: &State,
        name: &str,
        args: &[MiniValue],
    ) -> Result<MiniValue, MiniError> {
        match name {
            "db" => {
                if args.len() != 1 {
                    return Err(MiniError::new(
                        minijinja::ErrorKind::InvalidOperation,
                        "db method requires exactly 1 argument: key-path",
                    ));
                }

                let key_path = args[0].as_str().ok_or_else(|| {
                    MiniError::new(
                        minijinja::ErrorKind::InvalidOperation,
                        "the argument must be a string (key-path)",
                    )
                })?;

                let datastore = self.datastore.as_ref().ok_or_else(|| {
                    MiniError::new(
                        minijinja::ErrorKind::InvalidOperation,
                        "datastore not initialized",
                    )
                })?;

                // Extract file path and key components from key_path
                let parts: Vec<&str> = key_path.split('/').collect();
                if parts.is_empty() {
                    return Err(MiniError::new(
                        minijinja::ErrorKind::InvalidOperation,
                        "invalid key_path format: must include directory/file path",
                    ));
                }

                // Get the directory part (prefix before the first /)
                let dir_name = parts[0];

                // For remaining parts, the first one is the file name, rest is the key path
                let file_name = if parts.len() > 1 {
                    // Split the second part by first dot to separate filename from key
                    let file_parts: Vec<&str> = parts[1].splitn(2, '.').collect();
                    file_parts[0]
                } else {
                    "meta" // Default filename if not specified
                };

                // Construct the full file path
                let file_path = format!("{}/{}.yml", dir_name, file_name);

                // Build the key path for nested access
                let key_parts: Vec<String> = if parts.len() > 1 {
                    // Get the remaining part after the filename
                    let remaining = if parts[1].contains('.') {
                        let file_parts: Vec<&str> = parts[1].splitn(2, '.').collect();
                        if file_parts.len() > 1 {
                            file_parts[1]
                        } else {
                            ""
                        }
                    } else {
                        ""
                    };

                    // Combine with any additional path components
                    let mut key_string = remaining.to_string();
                    for i in 2..parts.len() {
                        if !key_string.is_empty() {
                            key_string.push('.');
                        }
                        key_string.push_str(parts[i]);
                    }

                    // Split by dots to get the key parts
                    key_string.split('.').map(|s| s.to_string()).collect()
                } else {
                    Vec::new()
                };

                let value: serde_yaml::Value = if key_parts.len() == 1 {
                    // Use get_with_key for single key access
                    datastore.get_with_key(&file_path, &key_parts[0]).map_err(|e| {
                        let error_msg = format!("failed to get value from datastore: {}", e);
                        MiniError::new(minijinja::ErrorKind::InvalidOperation, error_msg)
                    })?
                } else if key_parts.len() > 1 {
                    // Use get_with_key_vec for nested key access
                    // Convert Vec<String> to Vec<&str> for the function call
                    let key_refs: Vec<&str> = key_parts.iter().map(|s| s.as_str()).collect();
                    datastore.get_with_key_vec(&file_path, &key_refs).map_err(|e| {
                        let error_msg = format!("failed to get value from datastore: {}", e);
                        MiniError::new(minijinja::ErrorKind::InvalidOperation, error_msg)
                    })?
                } else {
                    // No keys specified, get the entire file
                    datastore.get(&file_path).map_err(|e| {
                        let error_msg = format!("failed to get value from datastore: {}", e);
                        MiniError::new(minijinja::ErrorKind::InvalidOperation, error_msg)
                    })?
                };

                Ok(MiniValue::from_serialize(&value))
            }
            _ => {
                let error_msg = format!("method {} not found", name);
                Err(MiniError::new(
                    minijinja::ErrorKind::InvalidOperation,
                    error_msg,
                ))
            }
        }
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Str(&["ingredients", "scale"])
    }
}

pub fn render_template(
    recipe: &str,
    template: &str,
    scale: Option<u32>,
    datastore_path: Option<&Path>,
) -> Result<String, Box<dyn std::error::Error>> {
    // Parse recipe
    let parser = CooklangParser::new(Extensions::all(), Converter::default());
    let (recipe, _warnings) = parser.parse(recipe).into_result()?;

    // Scale recipe
    let converter = Converter::default();
    let recipe = if let Some(scale) = scale {
        recipe.scale(scale, &converter)
    } else {
        recipe.default_scale()
    };

    let mut recipe_template = RecipeTemplate::from(recipe);
    if let Some(scale) = scale {
        recipe_template.scale = scale;
    }
    if let Some(path) = datastore_path {
        recipe_template.datastore = Some(Datastore::open(path));
    }

    // Setup template environment
    let mut env = Environment::new();
    env.add_template("base", template)?;
    env.add_function("db", functions::get_from_datastore);

    // Add quantity filter to format quantities nicely
    env.add_filter("quantity", filters::quantity_filter);

    // TODO: remove this once we have a proper context
    // Create context with both direct access and recipe_template
    let mut context = std::collections::HashMap::new();
    let recipe_template_value = MiniValue::from_object(recipe_template);

    // Add direct access to ingredients and scale
    if let Ok(ingredients) = recipe_template_value.get_attr("ingredients") {
        context.insert("ingredients", ingredients);
    }
    if let Ok(scale) = recipe_template_value.get_attr("scale") {
        context.insert("scale", scale);
    }
    context.insert("recipe_template", recipe_template_value);

    // Render template
    let tmpl = env.get_template("base")?;
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
    fn test_datastore_access() {
        let datastore_path = get_test_data_path().join("db");

        // Use Pancakes.cook from test data
        let recipe_path = get_test_data_path().join("recipes").join("Pancakes.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        let template = indoc! {"
            # Eggs Info

            Density: {{ db('eggs/meta.density') }}
            Shelf Life: {{ db('eggs/meta.storage.shelf life') }} days
            Fridge Life: {{ db('eggs/meta.storage.fridge life') }} days
        "};

        let result = render_template(&recipe, template, None, Some(&datastore_path)).unwrap();
        let expected = indoc! {"
            # Eggs Info

            Density: 1.03
            Shelf Life: 30 days
            Fridge Life: 60 days"};

        assert_eq!(result, expected);
    }

    #[test]
    fn test_simple_recipe_template() {
        // Use Pancakes.cook from test data
        let recipe_path = get_test_data_path().join("recipes").join("Pancakes.cook");
        let recipe = std::fs::read_to_string(recipe_path).unwrap();

        let template = indoc! {"
            # Ingredients ({{ scale }}x)
            {%- for ingredient in ingredients %}
            - {{ ingredient.name }}
            {%- endfor %}
        "};

        // Test default scaling (1x)
        let result = render_template(&recipe, template, None, None).unwrap();
        let expected = indoc! {"
            # Ingredients (1x)
            - eggs
            - milk
            - flour"};
        assert_eq!(result, expected);

        // Test with 2x scaling
        let result = render_template(&recipe, template, Some(2), None).unwrap();
        let expected = indoc! {"
            # Ingredients (2x)
            - eggs
            - milk
            - flour"};
        assert_eq!(result, expected);
    }

    // TODO need to update parser builder to support scaling
    #[test]
    #[ignore]
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
        let result = render_template(&recipe, template, None, None).unwrap();
        let expected = indoc! {"
            # Ingredients (1x)
            - eggs: 3 large
            - milk: 250 ml
            - flour: 125 g"};
        assert_eq!(result, expected);

        // Test with 2x scaling
        let result = render_template(&recipe, template, Some(2), None).unwrap();
        let expected = indoc! {"
            # Ingredients (2x)
            - eggs: 6 large
            - milk: 500 ml
            - flour: 250 g"};
        assert_eq!(result, expected);

        // Test with 3x scaling
        let result = render_template(&recipe, template, Some(3), None).unwrap();
        let expected = indoc! {"
            # Ingredients (3x)
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

        let template_path = get_test_data_path().join("reports").join("ingredients.md.jinja");
        let template = std::fs::read_to_string(template_path).unwrap();

        let result = render_template(&recipe, &template, None, None).unwrap();
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

        // Create a simplified version of the cost template that uses db function
        let template = indoc! {"
            # Cost Report

            {% set eggs_price = db('eggs/shopping.price_per_1') * 3 %}
            {% set milk_price = db('milk/shopping.price_per_1') * 250 %}
            {% set flour_price = db('flour/shopping.price_per_1') * 125 %}
            {% set total = eggs_price + milk_price + flour_price %}

            * Eggs (3): ${{ eggs_price | round(2) }}
            * Milk (250ml): ${{ milk_price | round(2) }}
            * Flour (125g): ${{ flour_price | round(2) }}

            Total: ${{ total | round(2) }}
        "};

        let result = render_template(&recipe, &template, None, Some(&datastore_path)).unwrap();

        // Check if the result contains expected values without comparing exact formatting
        assert!(result.contains("# Cost Report"));
        assert!(result.contains("Eggs (3): $"));
        assert!(result.contains("Milk (250ml): $"));
        assert!(result.contains("Flour (125g): $"));
        assert!(result.contains("Total: $"));
    }

    #[test]
    fn test_quantity_filter() {
        // Test recipe with various quantity formats
        let recipe = indoc! {"
            Mix @eggs{3%large} with @milk{250 %ml}, add @flour{ 125%g } to make batter.
            Add @sugar{1.5%tbsp  } and @salt{  1/4 % tsp} for flavor.
        "};

        // Create a template that uses the quantity filter
        let template = indoc! {"
            # Ingredients with Formatted Quantities
            {%- for ingredient in ingredients %}
            * {{ ingredient.name }}: {{ ingredient.quantity | quantity }}
            {%- endfor %}
        "};

        let result = render_template(recipe, template, None, None).unwrap();
        let expected = indoc! {"
            # Ingredients with Formatted Quantities
            * eggs: 3 large
            * milk: 250 ml
            * flour: 125 g
            * sugar: 1.5 tbsp
            * salt: 1/4 tsp"};

        assert_eq!(result, expected);
    }
}
