use minijinja::{Error as MiniError, Value as MiniValue};

pub fn numeric_filter(value: MiniValue, _args: &[MiniValue]) -> Result<MiniValue, MiniError> {
    // Extract the quantity value as a string
    let quantity_str = value.to_string();

    // Find the first non-numeric character (excluding decimal point and fraction slash)
    let numeric_part = quantity_str
        .chars()
        .take_while(|c| c.is_numeric() || *c == '.' || *c == '/')
        .collect::<String>();

    // Parse the numeric part
    if numeric_part.contains('/') {
        // Handle fractions (e.g., "1/4")
        let parts: Vec<&str> = numeric_part.split('/').collect();
        if parts.len() == 2 {
            if let (Ok(num), Ok(denom)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                return Ok(MiniValue::from(num / denom));
            }
        }
    } else if let Ok(num) = numeric_part.parse::<f64>() {
        return Ok(MiniValue::from(num));
    }

    Err(MiniError::new(
        minijinja::ErrorKind::InvalidOperation,
        "could not extract numeric value from quantity",
    ))
}
