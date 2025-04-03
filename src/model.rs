use cooklang::Quantity;
use serde::{Serialize, Serializer};

// Because this is getting invoked automatically, I don't think I can fix this lint.
#[allow(clippy::ref_option)]
pub(crate) fn quantity_to_string<S>(
    value: &Option<Quantity>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(
        match value.as_ref() {
            Some(quantity) => quantity.to_string(),
            None => String::new(),
        }
        .as_str(),
    )
}

/// An Ingredient that's used here instead of the parser's one, for template access.
#[derive(Debug, Serialize)]
pub(crate) struct Ingredient<'a> {
    pub(crate) name: &'a str,
    #[serde(serialize_with = "quantity_to_string")]
    pub(crate) quantity: &'a Option<Quantity>,
}

impl<'a> From<&'a cooklang::Ingredient> for Ingredient<'a> {
    fn from(value: &'a cooklang::Ingredient) -> Self {
        Ingredient {
            name: &value.name,
            quantity: &value.quantity,
        }
    }
}
