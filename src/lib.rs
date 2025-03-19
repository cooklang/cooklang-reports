use cooklang::{CooklangParser, Extensions, Converter, Recipe, scale::Scaled, Value};
use minijinja::{Environment, value::{Value as MiniValue, Object, Enumerator}};
use std::{sync::Arc};

#[derive(Debug)]
pub struct RecipeTemplate {
    recipe: Recipe<Scaled, Value>,
    scale: u32,
}

impl From<Recipe<Scaled, Value>> for RecipeTemplate {
    fn from(recipe: Recipe<Scaled, Value>) -> Self {
        // Default scale is 1
        RecipeTemplate { recipe, scale: 1 }
    }
}

impl Object for RecipeTemplate {
    fn get_value(self: &Arc<Self>, key: &MiniValue) -> Option<MiniValue> {
        match key.as_str()? {
            "ingredients" => {
                let ingredients = self.recipe.ingredients.iter().map(|ingredient| {
                    let mut map = std::collections::HashMap::new();
                    map.insert("name".to_string(), ingredient.name.clone());
                    if let Some(quantity) = &ingredient.quantity {
                        map.insert("quantity".to_string(), quantity.to_string());
                        if let Some(note) = &ingredient.note {
                            map.insert("unit".to_string(), note.clone());
                        }
                    }
                    map
                }).collect::<Vec<_>>();
                Some(MiniValue::from_serialize(&ingredients))
            },
            "scale" => Some(MiniValue::from_serialize(&self.scale)),
            _ => None,
        }
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Str(&["ingredients", "scale"])
    }
}

pub fn render_template(recipe: &str, template: &str, scale: Option<u32>) -> Result<String, Box<dyn std::error::Error>> {
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

    // Setup template environment
    let mut env = Environment::new();
    env.add_template("base", template)?;

    // Render template
    let tmpl = env.get_template("base")?;
    Ok(tmpl.render(MiniValue::from_object(recipe_template))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

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
        let result = render_template(recipe, template, None).unwrap();
        let expected = indoc! {"
            # Ingredients (1x)
            - eggs
            - milk
            - flour"};
        assert_eq!(result, expected);

        // Test with 2x scaling
        let result = render_template(recipe, template, Some(2)).unwrap();
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
        let result = render_template(recipe, template, None).unwrap();
        let expected = indoc! {"
            # Ingredients (1x)
            - eggs: 3 large
            - milk: 250 ml
            - flour: 125 g"};
        assert_eq!(result, expected);

        // Test with 2x scaling
        let result = render_template(recipe, template, Some(2)).unwrap();
        let expected = indoc! {"
            # Ingredients (2x)
            - eggs: 6 large
            - milk: 500 ml
            - flour: 250 g"};
        assert_eq!(result, expected);

        // Test with 3x scaling
        let result = render_template(recipe, template, Some(3)).unwrap();
        let expected = indoc! {"
            # Ingredients (3x)
            - eggs: 9 large
            - milk: 750 ml
            - flour: 375 g"};
        assert_eq!(result, expected);
    }
}
