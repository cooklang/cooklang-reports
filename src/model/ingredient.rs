use super::Quantity;
use serde::Serialize;
use std::fmt::Display;

/// Wrapper for [`cooklang::Ingredient`] for reporting.
///
/// # Usage
///
/// Constructed from [`cooklang::Ingredient`] and can be converted into [`minijinja::Value`].
///
/// If you have an `ingredient`, then the following are valid ways to use that ingredient.
///
/// ```text
/// {{ ingredient }}
/// {{ ingredient.name }}
/// {{ ingredient.alias }}
/// {{ ingredient.note }}
/// {{ ingredient.quantity }}
/// ```
///
/// For the above:
///
/// - `ingredient` formats according to its `Display` implementation, which uses its display name and quantity.
/// - `ingredient.name` renders the ingredient's name field.
/// - `ingredient.alias` renders the ingredient's alias.
/// - `ingredient.note` renders the note attached to the ingredient.
/// - `ingredient.quantity` provides access to a [`Quantity`][`super::Quantity`].
#[derive(Clone, Debug, Serialize)]
pub struct Ingredient(cooklang::Ingredient);

impl From<cooklang::Ingredient> for Ingredient {
    /// Construct an [`Ingredient`] from a [`cooklang::Ingredient`] within a [`cooklang::ScaledRecipe`].
    fn from(ingredient: cooklang::Ingredient) -> Self {
        Self(ingredient)
    }
}

impl Display for Ingredient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0.quantity {
            Some(quantity) => write!(f, "{quantity} {}", self.0.display_name()),
            None => write!(f, "{}", self.0.display_name()),
        }
    }
}

impl minijinja::value::Object for Ingredient {
    fn repr(self: &std::sync::Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Plain
    }

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

    fn render(self: &std::sync::Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where
        Self: Sized + 'static,
    {
        self.fmt(f)
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
    #[test_case("Measure @olive oil|oil{1%tbsp} into #frying pan{}.", "{{ ingredient.quantity.value }}", "1"; "direct quantity value")]
    #[test_case("Measure @olive oil|oil{1%tbsp} into #frying pan{}.", "{{ ingredient.quantity.unit }}", "tbsp"; "direct quantity unit")]

    fn ingredient(recipe: &str, template: &str, result: &str) {
        let (recipe, env) = get_recipe_and_env(recipe, template);

        // Build context
        let context = context! {
            ingredient => Value::from_object(Ingredient(recipe.ingredients[0].clone()))
        };

        let template = env.get_template("test").unwrap();
        assert_eq!(result, template.render(context).unwrap());
    }
}
