use serde::Serialize;

/// Wrapper for [`cooklang::Cookware`] for reporting.
///
/// # Usage
///
/// Constructed from [`cooklang::Cookware`] and can be converted into [`minijinja::Value`].
///
/// If you have a `cookware`, then the following are valid ways to use it.
///
/// ```text
/// {{ cookware }}
/// {{ cookware.name }}
/// {{ cookware.alias }}
/// {{ cookware.note }}
/// {{ cookware.quantity }}
/// ```
///
/// For the above:
///
/// - `cookware` formats according to its `Display` implementation, which uses its display name.
/// - `cookware.name` renders the cookware's name field.
/// - `cookware.alias` renders the cookware's alias.
/// - `cookware.note` renders the note attached to the cookware.
/// - `cookware.quantity` provides access to a [`Value`][`cookware::Value`] as a string.
#[derive(Clone, Debug, Serialize)]
pub struct Cookware(cooklang::Cookware);

impl From<cooklang::Cookware> for Cookware {
    fn from(cookware: cooklang::Cookware) -> Self {
        Self(cookware)
    }
}

impl From<Cookware> for minijinja::Value {
    fn from(value: Cookware) -> Self {
        Self::from_object(value)
    }
}

impl minijinja::value::Object for Cookware {
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
                .as_ref()
                .map(ToString::to_string)
                .map(minijinja::Value::from),
            _ => None,
        }
    }

    fn render(self: &std::sync::Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where
        Self: Sized + 'static,
    {
        write!(f, "{}", self.0.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::tests::get_recipe_and_env;
    use minijinja::{Value, context};
    use test_case::test_case;

    #[test_case("Crack @egg{1} into #frying pan{}.", "{{ cookware }}", "frying pan"; "just name")]
    #[test_case("Crack @egg{1} into #frying pan{1}.", "{{ cookware }}", "frying pan"; "name and quantity")]
    #[test_case("Crack @egg{1} into #frying pan|pan{}.", "{{ cookware }}", "pan"; "aliased name")]
    #[test_case("Crack @egg{1} into #frying pan|pan{}(greased).", "{{ cookware.name }}", "frying pan"; "direct name")]
    #[test_case("Crack @egg{1} into #frying pan|pan{}(greased).", "{{ cookware.alias }}", "pan"; "direct alias")]
    #[test_case("Crack @egg{1} into #frying pan|pan{}(greased).", "{{ cookware.note }}", "greased"; "with note")]
    #[test_case("Crack @egg{1} into #frying pan|pan{1}(greased).", "{{ cookware.quantity }}", "1"; "direct quantity")]

    fn cookware(recipe: &str, template: &str, result: &str) {
        let (recipe, env) = get_recipe_and_env(recipe, template);

        // Build context
        let context = context! {
            cookware => Value::from(Cookware(recipe.cookware[0].clone()))
        };

        let template = env.get_template("test").unwrap();
        assert_eq!(result, template.render(context).unwrap());
    }
}
