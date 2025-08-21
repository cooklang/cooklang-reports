use minijinja::{Error, State, Value};

/// Filter ingredients to exclude items that are already in the pantry.
///
/// This function takes a list of ingredients and returns only those that are NOT
/// in the pantry configuration, i.e., items that need to be purchased.
///
/// # Arguments
/// * `ingredients` - The list of ingredients to filter
///
/// # Returns
/// A list of ingredients that are not in the pantry.
/// If no pantry configuration is available, returns all ingredients.
///
/// # Template Usage
/// ```jinja
/// # Need to buy
/// {% for ingredient in excluding_pantry(ingredients) %}
/// - {{ ingredient.name }}: {{ ingredient.quantity }}
/// {% endfor %}
/// ```
#[allow(clippy::needless_pass_by_value)]
pub fn excluding_pantry(state: &State, ingredients: Value) -> Result<Value, Error> {
    // Try to get pantry content from state
    let pantry_content = state
        .lookup("pantry_content")
        .and_then(|v| v.as_str().map(String::from));

    if let Some(content) = pantry_content {
        // Parse the pantry configuration
        let parse_result = cooklang::pantry::parse_lenient(&content);

        if let Some(pantry_conf) = parse_result.output() {
            // Filter ingredients - keep only those NOT in pantry
            let mut filtered = Vec::new();

            if let Ok(iter) = ingredients.try_iter() {
                for item in iter {
                    // Get ingredient name
                    if let Ok(name) = item.get_attr("name") {
                        if let Some(name_str) = name.as_str() {
                            // Check if this ingredient is NOT in the pantry
                            let in_pantry = pantry_conf.has_ingredient(name_str);

                            if !in_pantry {
                                filtered.push(item);
                            }
                        }
                    }
                }
            }

            Ok(Value::from(filtered))
        } else {
            // Failed to parse pantry configuration
            eprintln!("Warning: Failed to parse pantry configuration. Returning all ingredients.");
            Ok(ingredients)
        }
    } else {
        // No pantry configuration provided - return all ingredients
        Ok(ingredients)
    }
}

/// Filter ingredients to include only items that are in the pantry.
///
/// This is the opposite of `excluding_pantry` - it returns only items that ARE
/// in the pantry configuration.
///
/// # Arguments
/// * `ingredients` - The list of ingredients to filter
///
/// # Returns
/// A list of ingredients that are in the pantry.
/// If no pantry configuration is available, returns an empty list.
///
/// # Template Usage
/// ```jinja
/// # Already have in pantry
/// {% for ingredient in from_pantry(ingredients) %}
/// - {{ ingredient.name }}: {{ ingredient.quantity }}
/// {% endfor %}
/// ```
#[allow(clippy::needless_pass_by_value)]
pub fn from_pantry(state: &State, ingredients: Value) -> Result<Value, Error> {
    // Try to get pantry content from state
    let pantry_content = state
        .lookup("pantry_content")
        .and_then(|v| v.as_str().map(String::from));

    if let Some(content) = pantry_content {
        // Parse the pantry configuration
        let parse_result = cooklang::pantry::parse_lenient(&content);

        if let Some(pantry_conf) = parse_result.output() {
            // Filter ingredients - keep only those IN pantry
            let mut filtered = Vec::new();

            if let Ok(iter) = ingredients.try_iter() {
                for item in iter {
                    // Get ingredient name
                    if let Ok(name) = item.get_attr("name") {
                        if let Some(name_str) = name.as_str() {
                            // Check if this ingredient IS in the pantry
                            let in_pantry = pantry_conf.has_ingredient(name_str);

                            if in_pantry {
                                filtered.push(item);
                            }
                        }
                    }
                }
            }

            Ok(Value::from(filtered))
        } else {
            // Failed to parse pantry configuration
            eprintln!("Warning: Failed to parse pantry configuration. Returning empty list.");
            Ok(Value::from(Vec::<Value>::new()))
        }
    } else {
        // No pantry configuration provided - return empty list
        Ok(Value::from(Vec::<Value>::new()))
    }
}
