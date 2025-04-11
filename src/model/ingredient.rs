//! Model for ingredient.
use super::Quantity;
use serde::Serialize;

/// Struct representing an ingredient in a Cooklang recipe.
///
/// This struct is simply a newtype around [`cooklang::Ingredient`] that is compatible with
/// the template engine. It restricts some access, but makes other access more ergonomic.
///
/// Directly rendering an ingredient in a template will print the [display name][`cooklang::Ingredient::display_name`]
/// of the ingredient preceded by the [quantity][`super::Quantity`] if it is present.
///
/// TODO examples
#[derive(Clone, Debug, Serialize)]
pub struct Ingredient(cooklang::Ingredient);

impl From<cooklang::Ingredient> for Ingredient {
    /// Construct an [`Ingredient`] from a [`cooklang::Ingredient`] within a [`cooklang::ScaledRecipe`].
    fn from(ingredient: cooklang::Ingredient) -> Self {
        Self(ingredient)
    }
}

impl minijinja::value::Object for Ingredient {
    fn repr(self: &std::sync::Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Plain
    }

    /// Access a given key within this ingredient in a template.
    ///
    /// # Valid keys
    /// - `name`
    /// - `note`
    /// - `alias`
    /// - `quantity`
    fn get_value(self: &std::sync::Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        match key.as_str()? {
            "name" => Some(minijinja::Value::from(&self.0.name)),
            "note" => self.0.note.as_ref().map(minijinja::Value::from),
            "alias" => self.0.alias.as_ref().map(minijinja::Value::from),
            "quantity" => self
                .0
                .quantity
                .clone()
                .map(Quantity::from)
                .map(minijinja::Value::from),
            _ => None,
        }
    }

    /// Render the ingredient when directly requested in a template.
    ///
    /// Print the [display name][`cooklang::Ingredient::display_name`] of the ingredient preceded
    /// by the [quantity][`super::Quantity`] if it is present.
    fn render(self: &std::sync::Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where
        Self: Sized + 'static,
    {
        match &self.0.quantity {
            Some(quantity) => write!(f, "{quantity} {}", self.0.display_name()),
            None => write!(f, "{}", self.0.display_name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::tests::get_recipe_and_env;
    use minijinja::{Value, context};
    use test_case::test_case;

    #[test_case("Measure @olive oil{} into #frying pan{}.", "{{ ingredient }}", "olive oil"; "just name")]
    #[test_case("Measure @olive oil{1} into #frying pan{}.", "{{ ingredient }}", "1 olive oil"; "name and no unit quantity")]
    #[test_case("Measure @olive oil{1%tbsp} into #frying pan{}.", "{{ ingredient }}", "1 tbsp olive oil"; "name and quantity")]
    #[test_case("Measure @olive oil|oil{1%tbsp} into #frying pan{}.", "{{ ingredient }}", "1 tbsp oil"; "aliased name")]
    #[test_case("Measure @olive oil|oil{1%tbsp} into #frying pan{}.", "{{ ingredient.name }}", "olive oil"; "direct name")]
    #[test_case("Measure @olive oil|oil{1%tbsp} into #frying pan{}.", "{{ ingredient.alias }}", "oil"; "direct alias")]
    #[test_case("Measure @olive oil|oil{1%tbsp}(extra virgin) into #frying pan{}.", "{{ ingredient.note }}", "extra virgin"; "with note")]
    #[test_case("Measure @olive oil|oil{1%tbsp} into #frying pan{}.", "{{ ingredient.quantity }}", "1 tbsp"; "direct quantity")]

    fn ingredient(recipe: &str, template: &str, result: &str) {
        let (recipe, env) = get_recipe_and_env(recipe, template);

        // Build context
        let context = context! {
            ingredient => Value::from_object(Ingredient::from(recipe.ingredients[0].clone()))
        };

        let template = env.get_template("test").unwrap();
        assert_eq!(result, template.render(context).unwrap());
    }
}
