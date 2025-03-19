use cooklang::{CooklangParser, Extensions, Converter, Recipe, scale::Scaled, Value};
use minijinja::{Environment, value::{Value as MiniValue, Object, Enumerator}};
use std::{sync::Arc};

#[derive(Debug)]
pub struct RecipeTemplate(Recipe<Scaled, Value>);

impl From<Recipe<Scaled, Value>> for RecipeTemplate {
    fn from(recipe: Recipe<Scaled, Value>) -> Self {
        RecipeTemplate(recipe)
    }
}

impl Object for RecipeTemplate {
    fn get_value(self: &Arc<Self>, key: &MiniValue) -> Option<MiniValue> {
        match key.as_str()? {
            "ingredients" => Some(MiniValue::from_serialize(&self.0.ingredients)),
            _ => None,
        }
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Str(&["ingredients"])
    }
}

pub fn render_template(recipe: &str, template: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Parse recipe
    let parser = CooklangParser::new(Extensions::all(), Converter::default());
    let (recipe, _warnings) = parser.parse(recipe).into_result()?;
    let recipe = recipe.default_scale();
    let recipe_template = RecipeTemplate::from(recipe);

    // Setup template environment
    let mut env = Environment::new();
    env.add_template("recipe", template)?;

    // Render template
    let tmpl = env.get_template("recipe")?;
    Ok(tmpl.render(MiniValue::from_object(recipe_template))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_template() {
        let recipe = "Mix @eggs{3%large} with @milk{250%ml}, add @flour{125%g} to make batter.";
        let template = "# Ingredients\n\
            {%- for ingredient in ingredients %}\n\
            - {{ ingredient.name }}\n\
            {%- endfor %}";

        let result = render_template(recipe, template).unwrap();
        let expected = "# Ingredients\n\
            - eggs\n\
            - milk\n\
            - flour";

        assert_eq!(result, expected);
    }
}
