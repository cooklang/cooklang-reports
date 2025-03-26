use minijinja::{Error, ErrorKind::InvalidOperation};

pub fn numeric_filter(value: &str) -> Result<f64, Error> {
    // Find the first non-numeric character (excluding decimal point and fraction slash)
    let numeric_part = value
        .chars()
        .take_while(|c| c.is_numeric() || *c == '.' || *c == '/')
        .collect::<String>();

    // Parse the numeric part
    if numeric_part.contains('/') {
        // Handle fractions (e.g., "1/4")
        let parts: Vec<&str> = numeric_part.split('/').collect();
        if parts.len() == 2 {
            if let (Ok(num), Ok(denom)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                let fraction = num / denom;
                if fraction.is_finite() {
                    return Ok(fraction);
                }
            }
        }
    } else if let Ok(num) = numeric_part.parse::<f64>() {
        return Ok(num);
    }
    Err(Error::new(InvalidOperation, "could not parse numeric"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::assert_approx_eq;

    fn test<F: Into<f64>>(expected: F, actual: &str) {
        assert_approx_eq!(f64, expected.into(), numeric_filter(actual).unwrap());
    }

    #[test]
    fn integer() {
        test(42, "42");
    }

    #[test]
    fn float() {
        test(42.54, "42.54");
    }

    #[test]
    fn fraction() {
        test(0.5, "1/2");
    }

    #[test]
    fn integer_with_text() {
        test(42, "42oz");
        test(42, "42 ounces");
    }

    #[test]
    fn float_with_text() {
        test(42.54, "42.54c");
        test(42.54, "42.54 cups");
        test(42.54, "42.54 cup.units");
    }

    #[test]
    fn fraction_with_text() {
        test(0.5, "1/2tsp");
        test(0.5, "1/2 teaspoon");
        test(0.5, "1/2 tea/spoons");
    }

    #[test]
    fn err_non_numeric() {
        numeric_filter("pinch, single").unwrap_err();
        numeric_filter("pi2nch, single").unwrap_err();
        numeric_filter("pinch, 3single").unwrap_err();
    }

    #[test]
    fn divide_by_zero() {
        numeric_filter("1/0").unwrap_err();
    }
}
