use crate::model::{IngredientList as ModelIngredientList, quantity_from_value};
use crate::parser::{get_converter, get_parser};
use anyhow::{Context, Result, anyhow};
use cooklang::{
    ingredient_list::IngredientList,
    quantity::{GroupedQuantity, Quantity, Value as QuantityValue},
};
use minijinja::{Error, ErrorKind, State, Value};
use std::collections::BTreeMap;

/// Recursively extract and merge ingredients from a recipe, including referenced sub-recipes
///
/// This function processes a list of ingredients and optionally expands any recipe references
/// (e.g., `@./Pancakes.cook{2}`) into their actual ingredients. It merges duplicate
/// ingredients by combining their quantities.
///
/// # Arguments
/// * `ingredients` - The list of ingredients to process
/// * `expand_references` - Optional boolean to control whether to expand recipe references.
///   Defaults to `true` (current behavior). When `false`, recipe
///   references are kept as-is without expansion.
// TODO unessary reimplementation, need to convert ingredients to Cooklang::Ingredients and then
// reuse parser functions.
#[allow(clippy::needless_pass_by_value)]
pub fn get_ingredient_list(
    state: &State,
    ingredients: &Value,
    expand_references: Option<Value>,
) -> Result<Value, Error> {
    let base_path = extract_base_path(state);

    // Default to true if not provided
    let should_expand = expand_references
        .as_ref()
        .is_none_or(minijinja::Value::is_true);

    let mut list = IngredientList::new();
    let mut seen = BTreeMap::new();

    // Process all ingredients directly
    process_ingredients(
        ingredients,
        &mut list,
        &mut seen,
        &base_path,
        1.0,
        should_expand,
    )
    .map_err(|e| {
        // Preserve the original error message for better debugging
        Error::new(ErrorKind::InvalidOperation, format!("{e:#}"))
    })?;

    // Convert to model IngredientList and return as Value
    let model_list = ModelIngredientList::from_cooklang(list);
    Ok(Value::from(model_list))
}

/// Extract the base path from the state, defaulting to current directory
fn extract_base_path(state: &State) -> String {
    state
        .lookup("base_path")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| {
            std::env::current_dir()
                .map_or_else(|_| ".".to_string(), |p| p.to_string_lossy().to_string())
        })
}

/// Process ingredients from minijinja Values
fn process_ingredients(
    ingredients: &Value,
    list: &mut IngredientList,
    seen: &mut BTreeMap<String, usize>,
    base_path: &str,
    parent_scaling: f64,
    expand_references: bool,
) -> Result<()> {
    let iter = ingredients
        .try_iter()
        .map_err(|e| anyhow!("ingredients must be an array: {}", e))?;

    for item in iter {
        // Check if this is a recipe reference
        let is_reference = item
            .get_attr("reference")
            .map(|v| v.is_true())
            .unwrap_or(false);

        if is_reference && expand_references {
            // Handle recipe reference only if expansion is enabled
            process_recipe_reference(
                &item,
                list,
                seen,
                base_path,
                parent_scaling,
                expand_references,
            )?;
        } else {
            // Handle regular ingredient (or reference when expansion is disabled)
            process_regular_ingredient(&item, list, parent_scaling)?;
        }
    }

    Ok(())
}

/// Process a regular ingredient
fn process_regular_ingredient(
    item: &Value,
    list: &mut IngredientList,
    parent_scaling: f64,
) -> Result<()> {
    let name = item
        .get_attr("name")
        .map_err(|e| anyhow!("Failed to get ingredient name: {}", e))?
        .as_str()
        .ok_or_else(|| anyhow!("Ingredient name must be a string"))?
        .to_string();

    // Get the display name (use alias if present)
    let display_name = item
        .get_attr("alias")
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or(name);

    let mut grouped = GroupedQuantity::empty();

    // Parse and add quantity if present
    if let Ok(qty_val) = item.get_attr("quantity") {
        if let Ok(qty) = quantity_from_value(&qty_val) {
            // Apply parent scaling if needed
            let final_qty = if (parent_scaling - 1.0).abs() > f64::EPSILON {
                match qty.value() {
                    QuantityValue::Number(n) => Quantity::new(
                        QuantityValue::Number((n.value() * parent_scaling).into()),
                        qty.unit().map(String::from),
                    ),
                    _ => qty,
                }
            } else {
                qty
            };
            grouped.add(&final_qty, get_converter());
        }
    }

    // Add the ingredient to the list using the parser's methods
    list.add_ingredient(display_name, &grouped, get_converter());
    Ok(())
}

/// Process a recipe reference
fn process_recipe_reference(
    item: &Value,
    list: &mut IngredientList,
    seen: &mut BTreeMap<String, usize>,
    base_path: &str,
    parent_scaling: f64,
    expand_references: bool,
) -> Result<()> {
    let name = item
        .get_attr("name")
        .map_err(|e| anyhow!("Failed to get ingredient name: {}", e))?
        .as_str()
        .ok_or_else(|| anyhow!("Ingredient name must be a string"))?
        .to_string();

    // Get the reference path
    let reference_path = item
        .get_attr("reference_path")
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .map_or_else(|| name, |s| normalize_path(&s));

    // Check for circular dependency
    if seen.contains_key(&reference_path) {
        return Err(anyhow!(
            "Circular dependency found: {} -> {}",
            seen.keys().cloned().collect::<Vec<_>>().join(" -> "),
            reference_path
        ));
    }

    seen.insert(reference_path.clone(), seen.len());

    // Load and parse the referenced recipe
    let recipe_entry = get_recipe(base_path, &reference_path)?;
    let content = recipe_entry
        .content()
        .context("Failed to read recipe content")?;

    let parse_result = get_parser().parse(&content);

    // Check if there are parse errors to include in error message
    if parse_result.report().has_errors() {
        let mut error_msg = format!("Failed to parse recipe '{reference_path}':");
        for error in parse_result.report().errors() {
            use std::fmt::Write;
            let _ = write!(error_msg, "\n  - {error}");
        }
        return Err(anyhow!(error_msg));
    }

    // Include warnings if present
    if parse_result.report().has_warnings() {
        for warning in parse_result.report().warnings() {
            eprintln!("Warning in '{reference_path}': {warning}");
        }
    }

    let mut recipe = parse_result
        .output()
        .ok_or_else(|| anyhow!("Failed to get recipe output for '{}'", reference_path))?
        .clone();

    // Apply scaling based on quantity if present
    if let Ok(qty_val) = item.get_attr("quantity") {
        if let Ok(qty) = quantity_from_value(&qty_val) {
            if let Some(unit) = qty.unit() {
                // Extract numeric value from quantity
                let target_value = match qty.value() {
                    QuantityValue::Number(n) => n.value(),
                    _ => 1.0,
                };

                recipe
                    .scale_to_target(target_value, Some(unit), get_converter())
                    .with_context(|| {
                        format!(
                            "Failed to scale recipe '{reference_path}' with target {target_value} {unit}"
                        )
                    })?;
            } else if let QuantityValue::Number(n) = qty.value() {
                // Just a number, use as scaling factor
                recipe.scale(n.value(), get_converter());
            }
        }
    }

    // Apply parent scaling if needed
    if (parent_scaling - 1.0).abs() > f64::EPSILON {
        recipe.scale(parent_scaling, get_converter());
    }

    // Add recipe ingredients to list, get back indices of recipe references
    let ref_indices = list.add_recipe(&recipe, get_converter(), false);

    // Process nested recipe references recursively
    for ref_index in ref_indices {
        let nested_ingredient = &recipe.ingredients[ref_index];

        // Create a minijinja Value representing the nested reference
        let mut map = std::collections::HashMap::new();
        map.insert("name", Value::from(nested_ingredient.name.clone()));
        map.insert("reference", Value::from(true));

        if let Some(ref_) = &nested_ingredient.reference {
            map.insert(
                "reference_path",
                Value::from(ref_.path(std::path::MAIN_SEPARATOR_STR)),
            );
        }

        if let Some(qty) = &nested_ingredient.quantity {
            // Create a quantity object with value and unit preserved
            let mut qty_map = std::collections::HashMap::new();
            qty_map.insert("value", Value::from(qty.value().to_string()));
            if let Some(unit) = qty.unit() {
                qty_map.insert("unit", Value::from(unit));
            }
            map.insert("quantity", Value::from_iter(qty_map));
        }

        let nested_value = Value::from_iter(map);
        let nested_ingredients = Value::from(vec![nested_value]);
        process_ingredients(
            &nested_ingredients,
            list,
            seen,
            base_path,
            parent_scaling,
            expand_references,
        )?;
    }

    seen.remove(&reference_path);
    Ok(())
}

/// Normalize a recipe path by removing leading slashes
// TODO remove, it wrongly builds path
fn normalize_path(path: &str) -> String {
    path.strip_prefix('/').unwrap_or(path).to_string()
}

/// Load a recipe by name from the given base path
fn get_recipe(base_path: &str, name: &str) -> Result<cooklang_find::RecipeEntry> {
    Ok(cooklang_find::get_recipe_str(vec![base_path], name)?)
}
