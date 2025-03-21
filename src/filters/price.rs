use minijinja::{Error as MiniError, Value as MiniValue};

pub fn format_price_filter(value: MiniValue, args: &[MiniValue]) -> Result<MiniValue, MiniError> {
    // Get the number of decimal places from args, default to 2
    let decimals = if let Some(arg) = args.first() {
        match arg.kind() {
            minijinja::value::ValueKind::Number => {
                let num_str = arg.to_string();
                if let Ok(num) = num_str.parse::<f64>() {
                    num as i64
                } else {
                    return Err(MiniError::new(
                        minijinja::ErrorKind::InvalidOperation,
                        "format_price requires an integer argument for decimal places",
                    ));
                }
            }
            _ => {
                return Err(MiniError::new(
                    minijinja::ErrorKind::InvalidOperation,
                    "format_price requires an integer argument for decimal places",
                ));
            }
        }
    } else {
        2
    };

    // Get the numeric value
    match value.kind() {
        minijinja::value::ValueKind::Number => {
            let num_str = value.to_string();
            if let Ok(num) = num_str.parse::<f64>() {
                // Format the number with the specified decimal places
                let formatted = format!("{:.1$}", num, decimals as usize);
                Ok(MiniValue::from(formatted))
            } else {
                Err(MiniError::new(
                    minijinja::ErrorKind::InvalidOperation,
                    "format_price requires a numeric value",
                ))
            }
        }
        _ => Err(MiniError::new(
            minijinja::ErrorKind::InvalidOperation,
            "format_price requires a numeric value",
        )),
    }
}
