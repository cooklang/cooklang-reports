use crate::parser::get_converter;
use minijinja::{Error, State, Value};
use std::collections::BTreeMap;

/// Group ingredients by aisle category using an aisle configuration file.
///
/// This function takes a list of ingredients and groups them by their aisle categories
/// as defined in the aisle configuration. Ingredients without a category are placed
/// under "other".
///
/// # Arguments
/// * `ingredients` - The list of ingredients to categorize
///
/// # Returns
/// A map where keys are aisle categories and values are lists of ingredients.
/// If no aisle configuration is available, returns all ingredients under "other" category
/// and logs a warning.
///
/// # Template Usage
/// ```jinja
/// {% for aisle, items in aisled(ingredients) | items %}
/// ## {{ aisle }}
/// {% for ingredient in items %}
/// - {{ ingredient.name }}: {{ ingredient.quantities }}
/// {% endfor %}
/// {% endfor %}
/// ```
#[allow(clippy::needless_pass_by_value)]
pub fn aisled(state: &State, ingredients: Value) -> Result<Value, Error> {
    // Try to get aisle content from state
    let aisle_content = state
        .lookup("aisle_content")
        .and_then(|v| v.as_str().map(String::from));

    let mut result = BTreeMap::new();

    if let Some(content) = aisle_content {
        // Parse the aisle configuration
        let parse_result = cooklang::aisle::parse_lenient(&content);

        if let Some(aisle_conf) = parse_result.output() {
            // Build an IngredientList from the ingredients value
            let mut ingredient_list = cooklang::ingredient_list::IngredientList::new();

            // Extract ingredients from the minijinja Value
            if let Ok(iter) = ingredients.try_iter() {
                for item in iter {
                    // Get ingredient name
                    let name = item
                        .get_attr("name")
                        .ok()
                        .and_then(|v| v.as_str().map(String::from))
                        .unwrap_or_default();

                    // For categorization, we don't need quantities
                    // Just add the ingredient with empty quantity
                    ingredient_list.add_ingredient(name, &Default::default(), get_converter());
                }
            }

            // Categorize the ingredients
            let categorized = ingredient_list.categorize(aisle_conf);

            // Convert categorized ingredients back to template values
            // Process categories
            for (category, list) in categorized.categories {
                let mut category_items = Vec::new();

                // Find the original ingredient data from the input
                for (ingredient_name, _) in list {
                    if let Ok(iter) = ingredients.try_iter() {
                        for item in iter {
                            if let Ok(name) = item.get_attr("name") {
                                if name.as_str() == Some(&ingredient_name) {
                                    category_items.push(item);
                                    break;
                                }
                            }
                        }
                    }
                }

                if !category_items.is_empty() {
                    result.insert(category, Value::from(category_items));
                }
            }

            // Process "other" category
            if !categorized.other.is_empty() {
                let mut other_items = Vec::new();

                for (ingredient_name, _) in categorized.other {
                    if let Ok(iter) = ingredients.try_iter() {
                        for item in iter {
                            if let Ok(name) = item.get_attr("name") {
                                if name.as_str() == Some(&ingredient_name) {
                                    other_items.push(item);
                                    break;
                                }
                            }
                        }
                    }
                }

                if !other_items.is_empty() {
                    result.insert("other".to_string(), Value::from(other_items));
                }
            }
        } else {
            // Failed to parse aisle configuration
            eprintln!(
                "Warning: Failed to parse aisle configuration. All ingredients will be placed under 'other' category."
            );
            result.insert("other".to_string(), ingredients);
        }
    } else {
        // No aisle configuration provided
        eprintln!(
            "Warning: No aisle configuration provided. All ingredients will be placed under 'other' category. To configure aisles, use Config::builder().aisle_path(path)"
        );
        result.insert("other".to_string(), ingredients);
    }

    Ok(Value::from_iter(result))
}
