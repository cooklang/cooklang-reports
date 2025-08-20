#![allow(clippy::unnecessary_wraps)] // minijinja requires Result return type
#![allow(clippy::unwrap_used)] // Safe for char case conversions

use minijinja::Error;

pub fn camelize_filter(value: &str) -> Result<String, Error> {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in value.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            // Safe unwrap: chars always have at least one uppercase variant
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            // Safe unwrap: chars always have at least one lowercase variant
            result.push(c.to_lowercase().next().unwrap());
        }
    }

    Ok(result)
}

pub fn underscore_filter(value: &str) -> Result<String, Error> {
    let mut result = String::new();
    let mut prev_is_upper = false;

    for (i, c) in value.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('_');
            }
            // Safe unwrap: chars always have at least one lowercase variant
            result.push(c.to_lowercase().next().unwrap());
            prev_is_upper = true;
        } else if c == '-' || c == ' ' {
            result.push('_');
            prev_is_upper = false;
        } else {
            result.push(c);
            prev_is_upper = false;
        }
    }

    Ok(result)
}

pub fn dasherize_filter(value: &str) -> Result<String, Error> {
    let mut result = String::new();
    let mut prev_is_upper = false;

    for (i, c) in value.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('-');
            }
            // Safe unwrap: chars always have at least one lowercase variant
            result.push(c.to_lowercase().next().unwrap());
            prev_is_upper = true;
        } else if c == '_' || c == ' ' {
            result.push('-');
            prev_is_upper = false;
        } else {
            result.push(c);
            prev_is_upper = false;
        }
    }

    Ok(result)
}

pub fn humanize_filter(value: &str) -> Result<String, Error> {
    let mut result = String::new();
    let mut first = true;

    for c in value.chars() {
        if c == '_' || c == '-' {
            result.push(' ');
        } else if first {
            // Safe unwrap: chars always have at least one uppercase variant
            result.push(c.to_uppercase().next().unwrap());
            first = false;
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

pub fn titleize_filter(value: &str) -> Result<String, Error> {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in value.chars() {
        if c == '_' || c == '-' {
            result.push(' ');
            capitalize_next = true;
        } else if c == ' ' {
            result.push(c);
            capitalize_next = true;
        } else if capitalize_next {
            // Safe unwrap: chars always have at least one uppercase variant
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            // Safe unwrap: chars always have at least one lowercase variant
            result.push(c.to_lowercase().next().unwrap());
        }
    }

    Ok(result)
}

pub fn upcase_first_filter(value: &str) -> Result<String, Error> {
    let mut chars = value.chars();
    match chars.next() {
        None => Ok(String::new()),
        Some(c) => {
            let mut result = String::new();
            // Safe unwrap: chars always have at least one uppercase variant
            result.push(c.to_uppercase().next().unwrap());
            result.push_str(chars.as_str());
            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camelize() {
        assert_eq!(camelize_filter("hello_world").unwrap(), "HelloWorld");
        assert_eq!(camelize_filter("hello-world").unwrap(), "HelloWorld");
        assert_eq!(camelize_filter("hello world").unwrap(), "HelloWorld");
        assert_eq!(camelize_filter("hello_world_foo").unwrap(), "HelloWorldFoo");
        assert_eq!(camelize_filter("").unwrap(), "");
    }

    #[test]
    fn test_underscore() {
        assert_eq!(underscore_filter("HelloWorld").unwrap(), "hello_world");
        assert_eq!(underscore_filter("hello-world").unwrap(), "hello_world");
        assert_eq!(underscore_filter("hello world").unwrap(), "hello_world");
        assert_eq!(
            underscore_filter("HelloWorldFoo").unwrap(),
            "hello_world_foo"
        );
        assert_eq!(underscore_filter("").unwrap(), "");
    }

    #[test]
    fn test_dasherize() {
        assert_eq!(dasherize_filter("HelloWorld").unwrap(), "hello-world");
        assert_eq!(dasherize_filter("hello_world").unwrap(), "hello-world");
        assert_eq!(dasherize_filter("hello world").unwrap(), "hello-world");
        assert_eq!(
            dasherize_filter("HelloWorldFoo").unwrap(),
            "hello-world-foo"
        );
        assert_eq!(dasherize_filter("").unwrap(), "");
    }

    #[test]
    fn test_humanize() {
        assert_eq!(humanize_filter("hello_world").unwrap(), "Hello world");
        assert_eq!(humanize_filter("hello-world").unwrap(), "Hello world");
        assert_eq!(
            humanize_filter("hello_world_foo").unwrap(),
            "Hello world foo"
        );
        assert_eq!(humanize_filter("").unwrap(), "");
    }

    #[test]
    fn test_titleize() {
        assert_eq!(titleize_filter("hello_world").unwrap(), "Hello World");
        assert_eq!(titleize_filter("hello-world").unwrap(), "Hello World");
        assert_eq!(titleize_filter("hello world").unwrap(), "Hello World");
        assert_eq!(
            titleize_filter("hello_world_foo").unwrap(),
            "Hello World Foo"
        );
        assert_eq!(titleize_filter("").unwrap(), "");
    }

    #[test]
    fn test_upcase_first() {
        assert_eq!(upcase_first_filter("hello").unwrap(), "Hello");
        assert_eq!(upcase_first_filter("Hello").unwrap(), "Hello");
        assert_eq!(upcase_first_filter("hello world").unwrap(), "Hello world");
        assert_eq!(upcase_first_filter("").unwrap(), "");
    }
}
