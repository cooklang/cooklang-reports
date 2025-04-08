//! Model for item.
use super::{Cookware, Ingredient};
use std::fmt::Display;

/// A cooklang step item.
///
/// Cooklang provides these as indices, but we want them as actual references.
#[derive(Debug, Clone)]
pub(crate) enum Item {
    Text(String),
    Ingredient(Ingredient),
    Cookware(Cookware),
    //Timer,          // TODO
    //InlineQuantity, // TODO; probably won't implement
}

impl Item {
    pub(super) fn from_recipe_item(recipe: &cooklang::ScaledRecipe, item: cooklang::Item) -> Self {
        match item {
            cooklang::Item::Text { value } => Self::Text(value),
            cooklang::Item::Ingredient { index } => {
                Self::Ingredient(Ingredient::from(recipe.ingredients[index].clone()))
            }
            cooklang::Item::Cookware { index } => {
                Self::Cookware(Cookware::from(recipe.cookware[index].clone()))
            }
            cooklang::Item::Timer { index } => todo!(),
            cooklang::Item::InlineQuantity { index } => todo!(),
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Text(text) => write!(f, "{text}"),
            Item::Ingredient(ingredient) => {
                write!(f, "{}", minijinja::Value::from_object(ingredient.clone()))
            }
            Item::Cookware(cookware) => {
                write!(f, "{}", minijinja::Value::from_object(cookware.clone()))
            }
        }
    }
}

impl minijinja::value::Object for Item {
    fn repr(self: &std::sync::Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Plain
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

    #[test_case("Measure @olive oil{} into #frying pan{}.", "{{ item }}", "Measure "; "initial text")]
    #[test_case("@olive oil{} into #frying pan{}.", "{{ item }}", "olive oil"; "ingredient")]
    #[test_case("#frying pan{}.", "{{ item }}", "frying pan"; "cookware")]
    fn item(recipe: &str, template: &str, expected: &str) {
        let (recipe, env) = get_recipe_and_env(recipe, template);

        let item = match &recipe.sections[0].content[0] {
            cooklang::Content::Step(step) => Item::from_recipe_item(&recipe, step.items[0].clone()),
            cooklang::Content::Text(_) => unreachable!(),
        };

        // Build context
        let context = context! {
            item => Value::from_object(item)
        };

        let template = env.get_template("test").unwrap();
        assert_eq!(expected, template.render(context).unwrap());
    }
}
