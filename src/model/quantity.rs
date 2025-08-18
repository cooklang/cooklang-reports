use cooklang::quantity::{Quantity as CooklangQuantity, Value as QuantityValue};
use std::fmt::Display;

/// Wrapper for [`cooklang::Quantity`] for reporting, used in [`Ingredient`][`super::Ingredient`].
///
/// # Usage
///
/// Constructed from [`cooklang::Quantity`] and can be converted into [`minijinja::Value`].
///
/// If you have a `quantity`, for example, from an [`Ingredient`][`super::Ingredient`], then
/// the following are valid ways to use that quantity.
///
/// ```text
/// {{ quantity }}
/// {{ quantity.value }}
/// {{ quantity.unit }}
/// ```
///
/// # Limitations
///
/// While the quantity's value can be used in a template and passed through the builtin
/// [`float`][minijinja::filters::float] filter, this only works if the value is a number,
/// and not a range or text.
#[derive(Debug)]
pub struct Quantity(cooklang::Quantity);

impl From<cooklang::Quantity> for Quantity {
    fn from(quantity: cooklang::Quantity) -> Self {
        Self(quantity)
    }
}

impl From<Quantity> for minijinja::Value {
    fn from(value: Quantity) -> Self {
        Self::from_object(value)
    }
}

/// Convert a minijinja Value to a cooklang Quantity
/// The value should be an object with .value and .unit attributes
pub fn quantity_from_value(qty_val: &minijinja::Value) -> Result<CooklangQuantity, String> {
    // Get value and unit from the quantity object
    let value_val = qty_val
        .get_attr("value")
        .map_err(|e| format!("Failed to get quantity value: {e}"))?;
    let value_str = value_val
        .as_str()
        .map_or_else(|| value_val.to_string(), String::from);
    let unit = qty_val
        .get_attr("unit")
        .ok()
        .and_then(|u| u.as_str().map(String::from));

    // Parse the value string
    if let Ok(num) = value_str.parse::<f64>() {
        // Simple number
        Ok(CooklangQuantity::new(
            QuantityValue::Number(num.into()),
            unit,
        ))
    } else if value_str.contains('-') {
        // Handle range like "1-2"
        let parts: Vec<&str> = value_str.split('-').collect();
        if parts.len() == 2 {
            if let (Ok(start), Ok(end)) = (
                parts[0].trim().parse::<f64>(),
                parts[1].trim().parse::<f64>(),
            ) {
                return Ok(CooklangQuantity::new(
                    QuantityValue::Range {
                        start: start.into(),
                        end: end.into(),
                    },
                    unit,
                ));
            }
        }
        // If range parsing fails, treat as text
        Ok(CooklangQuantity::new(QuantityValue::Text(value_str), unit))
    } else {
        // Text value
        Ok(CooklangQuantity::new(QuantityValue::Text(value_str), unit))
    }
}

impl Display for Quantity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl minijinja::value::Object for Quantity {
    fn repr(self: &std::sync::Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Plain
    }

    fn get_value(self: &std::sync::Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        match key.as_str()? {
            "value" => Some(minijinja::Value::from(self.0.value().to_string())),
            "unit" => self.0.unit().map(minijinja::Value::from),
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

    #[test_case("Crack @egg{1} into pan.", "{{ quantity }}", "1"; "number without unit")]
    #[test_case("Pour @flour{100%g} into bowl.", "{{ quantity }}", "100 g"; "number with unit")]
    #[test_case("Crack @eggs{1-2} into pan.", "{{ quantity }}", "1-2"; "range without unit")]
    #[test_case("Pour @olive oil{1-2%tsp} into pan.", "{{ quantity }}", "1-2 tsp"; "range with unit")]
    #[test_case("Peel @garlic{clove}.", "{{ quantity }}", "clove"; "text without unit")]
    #[test_case("Peel @garlic{clove%big}.", "{{ quantity }}", "clove big"; "text with unit")]
    #[test_case("Peel @garlic{1%g}.", "{{ quantity.unit }}", "g"; "unit direct")]
    #[test_case("Peel @garlic{1}.", "{{ quantity.unit }}", ""; "unit direct when empty")]
    #[test_case("Peel @garlic{1%g}.", "{{ quantity.value }}", "1"; "value direct")]
    #[test_case("Peel @garlic{some%g}.", "{{ quantity.value }}", "some"; "value direct when text")]
    #[test_case("Peel @garlic{1-2%g}.", "{{ quantity.value }}", "1-2"; "value direct when range")]
    #[test_case("Peel @garlic{1%g}.", "{{ quantity.value | float }}", "1.0"; "number value as float")]

    fn quantity(recipe: &str, template: &str, result: &str) {
        let (recipe, env) = get_recipe_and_env(recipe, template);
        let first_quantity_in_recipe = recipe.ingredients[0].quantity.as_ref().unwrap().clone();

        // Build context
        let context = context! {
            quantity => Value::from(Quantity(first_quantity_in_recipe))
        };

        let template = env.get_template("test").unwrap();
        assert_eq!(result, template.render(context).unwrap());
    }
}
