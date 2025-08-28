use minijinja::Value;
use serde::Serialize;
use std::fmt::{self, Display};

/// A wrapper around cooklang's `IngredientList` that can be used in templates
#[derive(Debug, Clone, Serialize)]
pub struct IngredientList {
    items: Vec<IngredientListItem>,
}

/// An individual item in the ingredient list
#[derive(Debug, Clone, Serialize)]
pub struct IngredientListItem {
    pub name: String,
    pub quantities: GroupedQuantity,
}

/// Wrapper for grouped quantities that provides template-friendly display and iteration
#[derive(Clone, Debug, Serialize)]
pub struct GroupedQuantity {
    quantities: Vec<Quantity>,
}

/// Represents a single quantity with an optional unit
#[derive(Clone, Debug, Serialize)]
pub struct Quantity {
    pub value: String,
    pub unit: Option<String>,
}

// GroupedQuantity implementations
impl GroupedQuantity {
    /// Create from a list of quantities
    pub fn from_quantities(quantities: Vec<Quantity>) -> Self {
        Self { quantities }
    }

    /// Check if there are no quantities
    pub fn is_empty(&self) -> bool {
        self.quantities.is_empty()
    }
}

impl Display for GroupedQuantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted: Vec<String> = self
            .quantities
            .iter()
            .map(|q| {
                if let Some(unit) = &q.unit {
                    format!("{} {}", q.value, unit)
                } else {
                    q.value.clone()
                }
            })
            .collect();

        write!(f, "{}", formatted.join(", "))
    }
}

impl minijinja::value::Object for GroupedQuantity {
    fn repr(self: &std::sync::Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Seq
    }

    fn render(self: &std::sync::Arc<Self>, f: &mut fmt::Formatter<'_>) -> fmt::Result
    where
        Self: Sized + 'static,
    {
        self.fmt(f)
    }

    fn get_value(self: &std::sync::Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        // Check if it's a numeric index for iteration
        if let Some(idx) = key.as_usize() {
            return self
                .quantities
                .get(idx)
                .map(minijinja::Value::from_serialize);
        }

        // Otherwise check for named properties
        match key.as_str()? {
            // Allow accessing the raw array if needed
            "list" => Some(minijinja::Value::from_serialize(&self.quantities)),
            _ => None,
        }
    }

    fn enumerate(self: &std::sync::Arc<Self>) -> minijinja::value::Enumerator {
        minijinja::value::Enumerator::Seq(self.quantities.len())
    }
}

impl From<GroupedQuantity> for minijinja::Value {
    fn from(value: GroupedQuantity) -> Self {
        Self::from_object(value)
    }
}

// GroupedIngredient/IngredientListItem implementations
impl IngredientListItem {
    /// Create a new grouped ingredient with merged quantities
    pub fn new(name: String, quantities: GroupedQuantity) -> Self {
        Self { name, quantities }
    }

    /// Get a formatted string of all quantities
    pub fn quantities_str(&self) -> String {
        self.quantities.to_string()
    }
}

impl Display for IngredientListItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.quantities.to_string().is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}: {}", self.name, self.quantities)
        }
    }
}

impl minijinja::value::Object for IngredientListItem {
    fn repr(self: &std::sync::Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Plain
    }

    fn render(self: &std::sync::Arc<Self>, f: &mut fmt::Formatter<'_>) -> fmt::Result
    where
        Self: Sized + 'static,
    {
        self.fmt(f)
    }

    fn get_value(self: &std::sync::Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        match key.as_str()? {
            "name" => Some(minijinja::Value::from(&self.name)),
            "quantities" => Some(minijinja::Value::from(self.quantities.clone())),
            _ => None,
        }
    }
}

impl From<IngredientListItem> for minijinja::Value {
    fn from(value: IngredientListItem) -> Self {
        Self::from_object(value)
    }
}

// IngredientList implementations
impl IngredientList {
    /// Create a new `IngredientList` from cooklang's `IngredientList`
    pub fn from_cooklang(list: cooklang::ingredient_list::IngredientList) -> Self {
        let mut items = Vec::new();

        for (name, grouped_qty) in list {
            let quantities: Vec<Quantity> = grouped_qty
                .into_vec()
                .into_iter()
                .map(|qty| Quantity {
                    value: qty.value().to_string(),
                    unit: qty.unit().map(String::from),
                })
                .collect();

            items.push(IngredientListItem {
                name,
                quantities: GroupedQuantity::from_quantities(quantities),
            });
        }

        Self { items }
    }

    /// Get the items as a slice
    pub fn items(&self) -> &[IngredientListItem] {
        &self.items
    }
}

impl fmt::Display for IngredientList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.items {
            write!(f, "{}: {}", item.name, item.quantities)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

impl From<IngredientList> for Value {
    fn from(list: IngredientList) -> Self {
        // Convert each item to a Value using from_object to preserve the Object trait implementation
        let values: Vec<Value> = list.items.into_iter().map(Value::from_object).collect();
        Value::from(values)
    }
}
