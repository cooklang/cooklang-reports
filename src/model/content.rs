use super::Step;
use serde::Serialize;
use std::fmt::Display;

/// Wrapper for [`cooklang::Content`] for reporting.
///
/// # Usage
///
/// Constructed from [`cooklang::Content`] and can be converted into [`minijinja::Value`].
///
/// If you have a `content`, then the following are valid ways to use it.
///
/// ```text
/// {{ content }}
/// ```
///
/// For the above, it is formatted according to its `Display` implementation, which renders its [`Step`][`super::Step`] or string.
#[derive(Clone, Debug, Serialize)]
pub enum Content {
    Step(Step),
    Text(String),
}

impl From<Content> for minijinja::Value {
    fn from(value: Content) -> Self {
        Self::from_object(value)
    }
}

impl Content {
    pub(super) fn from_recipe_content(
        recipe: &cooklang::ScaledRecipe,
        content: cooklang::Content,
    ) -> Self {
        match content {
            cooklang::Content::Step(step) => Self::Step(Step::from_recipe_step(recipe, step)),
            cooklang::Content::Text(value) => Self::Text(value),
        }
    }
}

impl Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::Step(step) => write!(f, "{step}"),
            Content::Text(string) => write!(f, "{string}"),
        }
    }
}

impl minijinja::value::Object for Content {
    fn repr(self: &std::sync::Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Plain
    }

    fn render(self: &std::sync::Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::tests::get_recipe_and_env;
    use minijinja::{Value, context};
    use test_case::test_case;

    #[test_case("> This recipe is great!\n\nI am an actual step.", "{{ content }}", "This recipe is great!"; "initial text")]
    #[test_case("I am an actual step.\n\n> This recipe is great!", "{{ content }}", "1. I am an actual step."; "initial basic step")]
    #[test_case("Rinse @potatoes{1%kg} with @water.\n\n> This recipe is great!", "{{ content }}", "1. Rinse 1 kg potatoes with water."; "interesting step")]
    fn content(recipe: &str, template: &str, expected: &str) {
        let (recipe, env) = get_recipe_and_env(recipe, template);
        let first_content = recipe.sections[0].content[0].clone();

        // Build context
        let context = context! {
            content => Value::from(Content::from_recipe_content(&recipe, first_content))
        };

        let template = env.get_template("test").unwrap();
        assert_eq!(expected, template.render(context).unwrap());
    }
}
