use super::Item;
use serde::Serialize;
use std::fmt::Display;

/// Wrapper for [`cooklang::Step`] for reporting.
///
/// # Usage
///
/// Constructed from [`cooklang::Step`] and can be converted into [`minijinja::Value`].
///
/// If you have a `step`, then the following are valid ways to use it.
///
/// ```text
/// {{ step }}
/// {{ step.number }}
/// ```
///
/// For the above:
///
/// - `step` formats according to its `Display` implementation, which prints the step number and step text.
/// - `step.number` renders the step's number.
///
/// The step may also be iterated over in a template, which will enumerate all its parts. This is for more
/// advanced processing, for when the `{{ part }}` will need special rendering.
///
/// ```text
/// {% for part in step %}
/// {{ part }}
/// {% endfor %}
/// ```
#[derive(Clone, Debug, Serialize)]
pub struct Step {
    items: Vec<Item>,
    number: u32,
}

impl From<Step> for minijinja::Value {
    fn from(value: Step) -> Self {
        Self::from_object(value)
    }
}

impl Step {
    pub(super) fn from_recipe_step(recipe: &cooklang::ScaledRecipe, step: cooklang::Step) -> Self {
        Self {
            items: step
                .items
                .into_iter()
                .map(|item| Item::from_recipe_item(recipe, item))
                .collect(),
            number: step.number,
        }
    }
}

impl minijinja::value::Object for Step {
    fn repr(self: &std::sync::Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Seq
    }

    fn get_value(self: &std::sync::Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        // If it's an index, fetch it.
        if let Some(index) = key.as_usize() {
            return self.items.get(index).cloned().map(minijinja::Value::from);
        }

        match key.as_str()? {
            "number" => Some(minijinja::Value::from(self.number)),
            _ => None,
        }
    }

    fn enumerate(self: &std::sync::Arc<Self>) -> minijinja::value::Enumerator {
        minijinja::value::Enumerator::Seq(self.items.len())
    }

    fn render(self: &std::sync::Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where
        Self: Sized + 'static,
    {
        self.fmt(f)
    }
}

impl Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}. ", self.number)?;
        for item in &self.items {
            item.fmt(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::tests::get_recipe_and_env;
    use minijinja::{Value, context};
    use test_case::test_case;

    #[test_case("Wash your hands.\n\nGet ready.", "{{ step }}", "1. Wash your hands."; "text-only step")]
    #[test_case("Pour @olive oil{} into #frying pan{}.\n\nDon't burn yourself.", "{{ step }}", "1. Pour olive oil into frying pan."; "complex step")]
    #[test_case("Pour @olive oil{}\ninto #frying pan{}.\n\nDon't burn yourself.", "{{ step }}", "1. Pour olive oil into frying pan."; "multiline step")]
    #[test_case("Pour @olive oil{}\ninto #frying pan{}", "{{ step.number }}", "1"; "step number")]
    #[test_case("Pour @olive oil{} into #frying pan{}", "{% for part in step %}{{ part }},{% endfor %}",
        "Pour ,olive oil, into ,frying pan,"; "step parts")]

    fn step(recipe: &str, template: &str, expected: &str) {
        let (recipe, env) = get_recipe_and_env(recipe, template);

        let step = match &recipe.sections[0].content[0] {
            cooklang::Content::Step(step) => Step::from_recipe_step(&recipe, step.clone()),
            cooklang::Content::Text(_) => unreachable!(),
        };

        // Build context
        let context = context! {
            step => Value::from(step)
        };

        let template = env.get_template("test").unwrap();
        assert_eq!(expected, template.render(context).unwrap());
    }
}
