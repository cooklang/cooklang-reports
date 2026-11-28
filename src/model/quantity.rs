use cooklang::quantity::{Quantity as CooklangQuantity, Value as QuantityValue};
use cooklang::{Converter, CooklangParser, Extensions};
use serde::Serialize;
use std::fmt::Display;
use std::sync::OnceLock;

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
#[derive(Clone, Debug, Serialize)]
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

/// The quantity a template value holds.
///
/// A [`Quantity`] object, such as an ingredient's `quantity`, is used exactly
/// as parsed, so fractions, ranges and text keep their kind and numbers their
/// precision. Any other value with `.value` and `.unit` attributes (a map
/// written in a template) is read with the Cooklang quantity grammar:
/// `1/2` is a number, `1-2` and `1/2-1` are ranges, anything else is text.
pub fn quantity_from_value(qty_val: &minijinja::Value) -> Result<CooklangQuantity, String> {
    if let Some(quantity) = qty_val.downcast_object_ref::<Quantity>() {
        return Ok(quantity.0.clone());
    }

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

    Ok(CooklangQuantity::new(parse_value(&value_str), unit))
}

/// Parser for [`parse_value`]. `RANGE_VALUES` makes `1-2` a range with
/// cooklang 0.22; from spec 0011 on, ranges are core syntax and the flag does
/// nothing.
fn value_parser() -> &'static CooklangParser {
    static PARSER: OnceLock<CooklangParser> = OnceLock::new();
    PARSER.get_or_init(|| CooklangParser::new(Extensions::RANGE_VALUES, Converter::empty()))
}

/// Reads `text` as the value between an ingredient's braces, `@x{text}`.
/// Text that would end the braces, start a unit, a comment or a scaling
/// lock, or that the grammar rejects (`1/0`), is kept as text.
fn parse_value(text: &str) -> QuantityValue {
    let as_text = || QuantityValue::Text(text.trim().to_string());
    let breaks_the_braces = text.contains(['{', '}', '%', '\\', '\n', '\r'])
        || text.contains("--")
        || text.contains("[-")
        || text.trim_start().starts_with('=');
    if breaks_the_braces {
        return as_text();
    }
    value_parser()
        .parse(&format!("@x{{{text}}}"))
        .into_result()
        .ok()
        .and_then(|(recipe, _)| recipe.ingredients.into_iter().next()?.quantity)
        .map_or_else(as_text, |quantity| quantity.value().clone())
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

    const HALF: cooklang::quantity::Number = cooklang::quantity::Number::Fraction {
        whole: 0,
        num: 1,
        den: 2,
        err: 0.0,
    };

    #[test]
    fn from_value_keeps_a_parsed_quantity_as_it_is() {
        for q in [
            CooklangQuantity::new(
                QuantityValue::Range {
                    start: HALF,
                    end: 1.0.into(),
                },
                Some("tsp".into()),
            ),
            CooklangQuantity::new(QuantityValue::Number(HALF), Some("cup".into())),
            CooklangQuantity::new(QuantityValue::Number(0.333_35.into()), None),
            // text that looks like a range stays text
            CooklangQuantity::new(QuantityValue::Text("2-3".into()), None),
        ] {
            let value = Value::from(Quantity(q.clone()));
            assert_eq!(quantity_from_value(&value).unwrap(), q);
        }
    }

    #[test_case("1/2", Some("cup"), QuantityValue::Number(HALF); "fraction")]
    #[test_case("2", None, QuantityValue::Number(2.0.into()); "number")]
    #[test_case("1-2", Some("g"), QuantityValue::Range { start: 1.0.into(), end: 2.0.into() }; "range")]
    #[test_case("1/2-1", Some("tsp"), QuantityValue::Range { start: HALF, end: 1.0.into() }; "range with a fraction")]
    #[test_case("some", None, QuantityValue::Text("some".into()); "text")]
    #[test_case("1-2-3", None, QuantityValue::Text("1-2-3".into()); "not a range")]
    #[test_case("a {b}", None, QuantityValue::Text("a {b}".into()); "text with braces")]
    fn from_value_reads_a_plain_map_with_the_cooklang_grammar(
        value: &str,
        unit: Option<&str>,
        expected: QuantityValue,
    ) {
        let map = context! { value => value, unit => unit };
        assert_eq!(
            quantity_from_value(&map).unwrap(),
            CooklangQuantity::new(expected, unit.map(String::from))
        );
    }
}
