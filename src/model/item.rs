//! Model for item.
use super::{Cookware, Ingredient, Timer};
use serde::Serialize;
use std::fmt::Display;

/// Wrapper for [`cooklang::Item`] for reporting.
///
/// # Usage
///
/// Constructed from [`cooklang::Item`] and can be converted into [`minijinja::Value`].
///
/// This enum can only be used directly, and has no fields. Its rendering is handled differently
/// depending on its type.
#[derive(Clone, Debug, Serialize)]
pub enum Item {
    Text(String),
    Ingredient(Ingredient),
    Cookware(Cookware),
    Timer(Timer),
    //InlineQuantity, // TODO; probably won't implement
}

impl From<Item> for minijinja::Value {
    fn from(value: Item) -> Self {
        Self::from_object(value)
    }
}

impl Item {
    pub(super) fn from_recipe_item(recipe: &cooklang::Recipe, item: cooklang::Item) -> Self {
        match item {
            cooklang::Item::Text { value } => Self::Text(value),
            cooklang::Item::Ingredient { index } => {
                Self::Ingredient(Ingredient::from(recipe.ingredients[index].clone()))
            }
            cooklang::Item::Cookware { index } => {
                Self::Cookware(Cookware::from(recipe.cookware[index].clone()))
            }
            cooklang::Item::Timer { index } => {
                Self::Timer(Timer::from(recipe.timers[index].clone()))
            }
            cooklang::Item::InlineQuantity { index: _ } => unimplemented!(),
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Text(text) => write!(f, "{text}"),
            Item::Ingredient(ingredient) => {
                write!(f, "{}", minijinja::Value::from(ingredient.clone()))
            }
            Item::Cookware(cookware) => {
                write!(f, "{}", minijinja::Value::from(cookware.clone()))
            }
            Item::Timer(timer) => {
                write!(f, "{}", minijinja::Value::from(timer.clone()))
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
    #[test_case("Cook for ~{10%minutes}.", "{{ item }}", "Cook for "; "text before timer")]
    fn item(recipe: &str, template: &str, expected: &str) {
        let (recipe, env) = get_recipe_and_env(recipe, template);

        let item = match &recipe.sections[0].content[0] {
            cooklang::Content::Step(step) => Item::from_recipe_item(&recipe, step.items[0].clone()),
            cooklang::Content::Text(_) => unreachable!(),
        };

        // Build context
        let context = context! {
            item => Value::from(item)
        };

        let template = env.get_template("test").unwrap();
        assert_eq!(expected, template.render(context).unwrap());
    }

    #[test]
    fn timer_item() {
        let recipe = "Cook for ~{10%minutes}.";
        let template = "{{ item }}";
        let (recipe, env) = get_recipe_and_env(recipe, template);

        let item = match &recipe.sections[0].content[0] {
            cooklang::Content::Step(step) => Item::from_recipe_item(&recipe, step.items[1].clone()),
            cooklang::Content::Text(_) => unreachable!(),
        };

        // Build context
        let context = context! {
            item => Value::from(item)
        };

        let template = env.get_template("test").unwrap();
        assert_eq!("10 minutes", template.render(context).unwrap());
    }
}
