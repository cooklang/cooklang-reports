use super::Quantity;
use serde::Serialize;
use std::fmt::Display;

/// Wrapper for [`cooklang::Timer`] for reporting.
///
/// # Usage
///
/// Constructed from [`cooklang::Timer`] and can be converted into [`minijinja::Value`].
///
/// If you have a `timer`, the following are valid ways to use it:
///
/// ```text
/// {{ timer }}
/// {{ timer.name }}
/// {{ timer.quantity }}
/// {{ timer.quantity.value }}
/// {{ timer.quantity.unit }}
/// ```
#[derive(Clone, Debug, Serialize)]
pub struct Timer {
    pub name: Option<String>,
    pub quantity: Option<Quantity>,
}

impl From<cooklang::Timer> for Timer {
    fn from(timer: cooklang::Timer) -> Self {
        Self {
            name: timer.name,
            quantity: timer.quantity.map(Quantity::from),
        }
    }
}

impl From<Timer> for minijinja::Value {
    fn from(value: Timer) -> Self {
        Self::from_object(value)
    }
}

impl Display for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.name, &self.quantity) {
            (Some(name), Some(quantity)) => write!(f, "{name} for {quantity}"),
            (Some(name), None) => write!(f, "{name}"),
            (None, Some(quantity)) => write!(f, "{quantity}"),
            (None, None) => write!(f, "timer"),
        }
    }
}

impl minijinja::value::Object for Timer {
    fn repr(self: &std::sync::Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Plain
    }

    fn get_value(self: &std::sync::Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        match key.as_str()? {
            "name" => self
                .name
                .as_ref()
                .map(|n| minijinja::Value::from(n.clone())),
            "quantity" => self
                .quantity
                .as_ref()
                .map(|q| minijinja::Value::from(q.clone())),
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

    #[test_case("Cook for ~{10%minutes}.", "{{ timer }}", "10 minutes"; "timer with quantity")]
    #[test_case("~{Timer}.", "{{ timer }}", "Timer"; "timer with name only")]
    #[test_case("Cook for ~oven timer{10%minutes}.", "{{ timer }}", "oven timer for 10 minutes"; "timer with name and quantity")]
    #[test_case("Cook for ~{10%min}.", "{{ timer.quantity }}", "10 min"; "timer quantity")]
    #[test_case("Cook for ~{10%min}.", "{{ timer.quantity.value }}", "10"; "timer quantity value")]
    #[test_case("Cook for ~{10%min}.", "{{ timer.quantity.unit }}", "min"; "timer quantity unit")]
    #[test_case("~oven timer{10%min}.", "{{ timer.name }}", "oven timer"; "timer name")]
    fn timer(recipe: &str, template: &str, expected: &str) {
        let (recipe, env) = get_recipe_and_env(recipe, template);

        let timer = Timer::from(recipe.timers[0].clone());

        // Build context
        let context = context! {
            timer => Value::from(timer)
        };

        let template = env.get_template("test").unwrap();
        assert_eq!(expected, template.render(context).unwrap());
    }
}
