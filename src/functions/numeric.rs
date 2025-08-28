use minijinja::value::Kwargs;
use minijinja::{Error, ErrorKind, State, Value};

fn parse_number(value: &Value) -> Result<f64, Error> {
    // Try to parse as integer first
    if let Some(int) = value.as_i64() {
        #[allow(clippy::cast_precision_loss)]
        return Ok(int as f64);
    }

    // Try to parse from string representation
    let s = value.to_string();
    s.parse::<f64>()
        .map_err(|_| Error::new(ErrorKind::InvalidOperation, format!("Invalid number: {s}")))
}

#[allow(clippy::needless_pass_by_value)]
pub fn number_to_currency(_state: &State, value: Value, kwargs: Kwargs) -> Result<String, Error> {
    let number = parse_number(&value)?;

    let precision: usize = kwargs.get("precision").unwrap_or(2);
    let unit: &str = kwargs.get("unit").unwrap_or("$");
    let delimiter: &str = kwargs.get("delimiter").unwrap_or(",");
    let separator: &str = kwargs.get("separator").unwrap_or(".");
    let format_str: &str = kwargs.get("format").unwrap_or("%u%n");
    let negative_format: &str = kwargs.get("negative_format").unwrap_or("-%u%n");

    let abs_number = number.abs();
    let is_negative = number < 0.0;

    let formatted_number = format_number(abs_number, precision, delimiter, separator);

    let template = if is_negative {
        negative_format
    } else {
        format_str
    };

    Ok(template
        .replace("%u", unit)
        .replace("%n", &formatted_number))
}

#[allow(clippy::needless_pass_by_value)]
pub fn number_to_human(_state: &State, value: Value, kwargs: Kwargs) -> Result<String, Error> {
    let number = parse_number(&value)?;

    let precision: usize = kwargs.get("precision").unwrap_or(3);
    let separator: &str = kwargs.get("separator").unwrap_or(".");
    let delimiter: &str = kwargs.get("delimiter").unwrap_or("");

    let abs_number = number.abs();
    let is_negative = number < 0.0;

    let (formatted, unit) = if abs_number < 1_000.0 {
        (
            format_number(abs_number, precision, delimiter, separator),
            "",
        )
    } else if abs_number < 1_000_000.0 {
        (
            format_number(abs_number / 1_000.0, precision, delimiter, separator),
            " Thousand",
        )
    } else if abs_number < 1_000_000_000.0 {
        (
            format_number(abs_number / 1_000_000.0, precision, delimiter, separator),
            " Million",
        )
    } else if abs_number < 1_000_000_000_000.0 {
        (
            format_number(
                abs_number / 1_000_000_000.0,
                precision,
                delimiter,
                separator,
            ),
            " Billion",
        )
    } else if abs_number < 1_000_000_000_000_000.0 {
        (
            format_number(
                abs_number / 1_000_000_000_000.0,
                precision,
                delimiter,
                separator,
            ),
            " Trillion",
        )
    } else {
        (
            format_number(
                abs_number / 1_000_000_000_000_000.0,
                precision,
                delimiter,
                separator,
            ),
            " Quadrillion",
        )
    };

    Ok(format!(
        "{}{}{}",
        if is_negative { "-" } else { "" },
        formatted,
        unit
    ))
}

#[allow(clippy::needless_pass_by_value)]
pub fn number_to_human_size(_state: &State, value: Value, kwargs: Kwargs) -> Result<String, Error> {
    let number = parse_number(&value)?;

    let precision: usize = kwargs.get("precision").unwrap_or(3);
    let separator: &str = kwargs.get("separator").unwrap_or(".");
    let delimiter: &str = kwargs.get("delimiter").unwrap_or("");

    if number < 0.0 {
        return Err(Error::new(
            ErrorKind::InvalidOperation,
            "Size cannot be negative",
        ));
    }

    let (formatted, unit) = if number < 1024.0 {
        (
            format_number(number, precision, delimiter, separator),
            " Bytes",
        )
    } else if number < 1024.0 * 1024.0 {
        (
            format_number(number / 1024.0, precision, delimiter, separator),
            " KB",
        )
    } else if number < 1024.0 * 1024.0 * 1024.0 {
        (
            format_number(number / (1024.0 * 1024.0), precision, delimiter, separator),
            " MB",
        )
    } else if number < 1024.0 * 1024.0 * 1024.0 * 1024.0 {
        (
            format_number(
                number / (1024.0 * 1024.0 * 1024.0),
                precision,
                delimiter,
                separator,
            ),
            " GB",
        )
    } else if number < 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0 {
        (
            format_number(
                number / (1024.0 * 1024.0 * 1024.0 * 1024.0),
                precision,
                delimiter,
                separator,
            ),
            " TB",
        )
    } else {
        (
            format_number(
                number / (1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0),
                precision,
                delimiter,
                separator,
            ),
            " PB",
        )
    };

    // Only trim ".0" for whole numbers (like 123.0 -> 123)
    let trimmed = if formatted.ends_with(".000") {
        formatted[..formatted.len() - 4].to_string()
    } else {
        formatted
    };
    Ok(format!("{trimmed}{unit}"))
}

#[allow(clippy::needless_pass_by_value)]
pub fn number_to_percentage(_state: &State, value: Value, kwargs: Kwargs) -> Result<String, Error> {
    let number = parse_number(&value)?;

    let precision: usize = kwargs.get("precision").unwrap_or(3);
    let separator: &str = kwargs.get("separator").unwrap_or(".");
    let delimiter: &str = kwargs.get("delimiter").unwrap_or("");
    let format_str: &str = kwargs.get("format").unwrap_or("%n%");

    let formatted = format_number(number, precision, delimiter, separator);
    Ok(format_str.replace("%n", &formatted))
}

#[allow(clippy::needless_pass_by_value)]
pub fn number_with_delimiter(
    _state: &State,
    value: Value,
    kwargs: Kwargs,
) -> Result<String, Error> {
    let number = parse_number(&value)?;

    let separator: &str = kwargs.get("separator").unwrap_or(".");
    let delimiter: &str = kwargs.get("delimiter").unwrap_or(",");

    // Determine precision based on the original number
    let precision = if number.fract() == 0.0 {
        0
    } else {
        let s = number.to_string();
        if let Some(dot_pos) = s.find('.') {
            s.len() - dot_pos - 1
        } else {
            0
        }
    };

    Ok(format_number(number, precision, delimiter, separator))
}

#[allow(clippy::needless_pass_by_value)]
pub fn number_with_precision(
    _state: &State,
    value: Value,
    kwargs: Kwargs,
) -> Result<String, Error> {
    let number = parse_number(&value)?;

    let precision: usize = kwargs.get("precision").unwrap_or(3);
    let separator: &str = kwargs.get("separator").unwrap_or(".");
    let delimiter: &str = kwargs.get("delimiter").unwrap_or("");
    let strip_insignificant_zeros: bool = kwargs.get("strip_insignificant_zeros").unwrap_or(false);

    let mut result = format_number(number, precision, delimiter, separator);

    if strip_insignificant_zeros && result.contains(separator) {
        result = result.trim_end_matches('0').to_string();
        if result.ends_with(separator) {
            result = result[..result.len() - separator.len()].to_string();
        }
    }

    Ok(result)
}

fn format_number(number: f64, precision: usize, delimiter: &str, separator: &str) -> String {
    let is_negative = number < 0.0;
    let abs_number = number.abs();

    let formatted = format!("{abs_number:.precision$}");
    let parts: Vec<&str> = formatted.split('.').collect();
    let integer_part = parts[0];
    let decimal_part = parts.get(1);

    let mut integer_with_delimiters = String::new();
    let chars: Vec<char> = integer_part.chars().collect();
    let len = chars.len();

    for (i, ch) in chars.iter().enumerate() {
        integer_with_delimiters.push(*ch);
        let remaining = len - i - 1;
        if remaining > 0 && remaining % 3 == 0 && !delimiter.is_empty() {
            integer_with_delimiters.push_str(delimiter);
        }
    }

    let mut result = String::new();
    if is_negative {
        result.push('-');
    }
    result.push_str(&integer_with_delimiters);

    if let Some(dec) = decimal_part {
        if precision > 0 {
            result.push_str(separator);
            result.push_str(dec);
        }
    } else if precision > 0 {
        result.push_str(separator);
        result.push_str(&"0".repeat(precision));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_to_currency() {
        // Create a minimal environment for testing
        let mut env = minijinja::Environment::new();
        env.add_function("number_to_currency", number_to_currency);

        // Test basic formatting
        let tmpl = env
            .template_from_str("{{ number_to_currency(1234.567) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "$1,234.57");

        // Test with precision
        let tmpl = env
            .template_from_str("{{ number_to_currency(1234.567, precision=1) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "$1,234.6");

        // Test with different unit
        let tmpl = env
            .template_from_str("{{ number_to_currency(1234.567, unit='£') }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "£1,234.57");

        // Test negative format
        let tmpl = env
            .template_from_str("{{ number_to_currency(-1234.567, negative_format='(%u%n)') }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "($1,234.57)");
    }

    #[test]
    fn test_number_to_human() {
        let mut env = minijinja::Environment::new();
        env.add_function("number_to_human", number_to_human);

        let tmpl = env.template_from_str("{{ number_to_human(123) }}").unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "123.000");

        let tmpl = env
            .template_from_str("{{ number_to_human(1234) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "1.234 Thousand");

        let tmpl = env
            .template_from_str("{{ number_to_human(1234567) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "1.235 Million");

        let tmpl = env
            .template_from_str("{{ number_to_human(1234567890) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "1.235 Billion");

        let tmpl = env
            .template_from_str("{{ number_to_human(1234567890, precision=2) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "1.23 Billion");
    }

    #[test]
    fn test_number_to_human_size() {
        let mut env = minijinja::Environment::new();
        env.add_function("number_to_human_size", number_to_human_size);

        let tmpl = env
            .template_from_str("{{ number_to_human_size(123) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "123 Bytes");

        let tmpl = env
            .template_from_str("{{ number_to_human_size(1234) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "1.205 KB");

        let tmpl = env
            .template_from_str("{{ number_to_human_size(1234567) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "1.177 MB");

        let tmpl = env
            .template_from_str("{{ number_to_human_size(1234567890) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "1.150 GB");

        let tmpl = env
            .template_from_str("{{ number_to_human_size(1234567890, precision=2) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "1.15 GB");
    }

    #[test]
    fn test_number_to_percentage() {
        let mut env = minijinja::Environment::new();
        env.add_function("number_to_percentage", number_to_percentage);

        let tmpl = env
            .template_from_str("{{ number_to_percentage(100) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "100.000%");

        let tmpl = env
            .template_from_str("{{ number_to_percentage(100, precision=0) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "100%");

        let tmpl = env
            .template_from_str("{{ number_to_percentage(302.24398923423, precision=2) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "302.24%");
    }

    #[test]
    fn test_number_with_delimiter() {
        let mut env = minijinja::Environment::new();
        env.add_function("number_with_delimiter", number_with_delimiter);

        let tmpl = env
            .template_from_str("{{ number_with_delimiter(12345678) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "12,345,678");

        let tmpl = env
            .template_from_str("{{ number_with_delimiter(12345678.05) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "12,345,678.05");

        let tmpl = env
            .template_from_str("{{ number_with_delimiter(12345678, delimiter='_') }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "12_345_678");
    }

    #[test]
    fn test_number_with_precision() {
        let mut env = minijinja::Environment::new();
        env.add_function("number_with_precision", number_with_precision);

        let tmpl = env
            .template_from_str("{{ number_with_precision(111.2345) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "111.234");

        let tmpl = env
            .template_from_str("{{ number_with_precision(111.2345, precision=2) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "111.23");

        let tmpl = env
            .template_from_str("{{ number_with_precision(13, precision=5) }}")
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "13.00000");

        let tmpl = env
            .template_from_str(
                "{{ number_with_precision(13, precision=5, strip_insignificant_zeros=true) }}",
            )
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "13");

        let tmpl = env
            .template_from_str(
                "{{ number_with_precision(13.5, precision=5, strip_insignificant_zeros=true) }}",
            )
            .unwrap();
        assert_eq!(tmpl.render(()).unwrap(), "13.5");
    }
}
