use minijinja::{Error as MiniError, Value as MiniValue};

pub fn quantity_filter(value: MiniValue, _args: &[MiniValue]) -> Result<MiniValue, MiniError> {
    // Extract the quantity value as a string
    let quantity_str = value.to_string();

    // Process the string to improve formatting
    let quantity_str = quantity_str.trim();

    // Normalize spaces between number and unit
    let formatted = if let Some(pos) = quantity_str.find(|c: char| !c.is_numeric() && c != '.' && c != '/') {
        let (number, unit) = quantity_str.split_at(pos);
        format!("{} {}", number.trim(), unit.trim())
    } else {
        quantity_str.to_string()
    };

    Ok(MiniValue::from(formatted))
}
