use cooklang::{Converter, CooklangParser, Extensions, Recipe, Value, scale::Scaled};
use minijinja::{
    Environment, Error as MiniError, State,
    value::{Enumerator, Object, Value as MiniValue},
};
use std::{path::Path, sync::Arc};
use yaml_datastore::Datastore;

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
            "get" => {
                if args.len() != 2 {
                    return Err(MiniError::new(
                        minijinja::ErrorKind::InvalidOperation,
                        "get method requires exactly 2 arguments: filename and key",
                    ));
                }

                let filename = args[0].as_str().ok_or_else(|| {
                    MiniError::new(
                        minijinja::ErrorKind::InvalidOperation,
                        "first argument must be a string (filename)",
                    )
                })?;

                let key = args[1].as_str().ok_or_else(|| {
                    MiniError::new(
                        minijinja::ErrorKind::InvalidOperation,
                        "second argument must be a string (key)",
                    )
                })?;

                let datastore = self.datastore.as_ref().ok_or_else(|| {
                    MiniError::new(
                        minijinja::ErrorKind::InvalidOperation,
                        "datastore not initialized",
                    )
                })?;

                let value: serde_yaml::Value =
                    datastore.get_with_key(filename, key).map_err(|e| {
                        let error_msg = format!("failed to get value from datastore: {}", e);
                        MiniError::new(minijinja::ErrorKind::InvalidOperation, error_msg)
                    })?;

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

fn get_from_datastore(state: &State, args: &[MiniValue]) -> Result<MiniValue, MiniError> {
    if args.len() != 2 {
        return Err(MiniError::new(
            minijinja::ErrorKind::InvalidOperation,
            "get_from_datastore requires exactly 2 arguments: filename and key",
        ));
    }

    let recipe_template = state.lookup("recipe_template").ok_or_else(|| {
        MiniError::new(
            minijinja::ErrorKind::InvalidOperation,
            "recipe_template not found in context",
        )
    })?;

    recipe_template.call_method(state, "get", args)
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
    env.add_function("get_from_datastore", get_from_datastore);

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
    use std::fs;
    use std::path::PathBuf;

    fn setup_test_datastore() -> PathBuf {
        let temp_dir = tempfile::tempdir().unwrap();
        let datastore_path = temp_dir.path().to_path_buf();

        // Create a test YAML file
        let yaml_content = indoc! {"
            ---
            servings: 4
            difficulty: medium
            preparation_time: 30
        "};

        fs::create_dir_all(&datastore_path).unwrap();
        fs::write(datastore_path.join("recipe_meta.yaml"), yaml_content).unwrap();

        // Keep the tempdir from being dropped
        std::mem::forget(temp_dir);

        datastore_path
    }

    #[test]
    fn test_datastore_access() {
        let datastore_path = setup_test_datastore();

        // Verify the file exists
        assert!(
            datastore_path.join("recipe_meta.yaml").exists(),
            "Test YAML file should exist"
        );

        let recipe = indoc! {"
            Mix @eggs{3%large} with @milk{250%ml}, add @flour{125%g} to make batter.
        "};
        let template = indoc! {"
            # Recipe Info
            Servings: {{ recipe_template.get('recipe_meta.yaml', 'servings') }}
            Difficulty: {{ recipe_template.get('recipe_meta.yaml', 'difficulty') }}
            Preparation Time: {{ recipe_template.get('recipe_meta.yaml', 'preparation_time') }} minutes
        "};

        let result = render_template(recipe, template, None, Some(&datastore_path)).unwrap();
        let expected = indoc! {"
            # Recipe Info
            Servings: 4
            Difficulty: medium
            Preparation Time: 30 minutes"};

        assert_eq!(result, expected);
    }

    #[test]
    fn test_simple_recipe_template() {
        let recipe = indoc! {"
            Mix @eggs{3%large} with @milk{250%ml}, add @flour{125%g} to make batter.
        "};
        let template = indoc! {"
            # Ingredients ({{ scale }}x)
            {%- for ingredient in ingredients %}
            - {{ ingredient.name }}
            {%- endfor %}
        "};

        // Test default scaling (1x)
        let result = render_template(recipe, template, None, None).unwrap();
        let expected = indoc! {"
            # Ingredients (1x)
            - eggs
            - milk
            - flour"};
        assert_eq!(result, expected);

        // Test with 2x scaling
        let result = render_template(recipe, template, Some(2), None).unwrap();
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
        let recipe = indoc! {"
            Mix @eggs{3%large} with @milk{250%ml}, add @flour{125%g} to make batter.
        "};
        let template = indoc! {"
            # Ingredients ({{ scale }}x)
            {%- for ingredient in ingredients %}
            - {{ ingredient.name }}{% if ingredient.quantity %}: {{ ingredient.quantity }}{% if ingredient.unit %} {{ ingredient.unit }}{% endif %}{% endif %}
            {%- endfor %}
        "};

        // Test default scaling (1x)
        let result = render_template(recipe, template, None, None).unwrap();
        let expected = indoc! {"
            # Ingredients (1x)
            - eggs: 3 large
            - milk: 250 ml
            - flour: 125 g"};
        assert_eq!(result, expected);

        // Test with 2x scaling
        let result = render_template(recipe, template, Some(2), None).unwrap();
        let expected = indoc! {"
            # Ingredients (2x)
            - eggs: 6 large
            - milk: 500 ml
            - flour: 250 g"};
        assert_eq!(result, expected);

        // Test with 3x scaling
        let result = render_template(recipe, template, Some(3), None).unwrap();
        let expected = indoc! {"
            # Ingredients (3x)
            - eggs: 9 large
            - milk: 750 ml
            - flour: 375 g"};
        assert_eq!(result, expected);
    }
}
