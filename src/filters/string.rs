pub fn camelize_filter(value: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in value.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            // Safe: chars always have at least one uppercase variant
            if let Some(upper_c) = c.to_uppercase().next() {
                result.push(upper_c);
            }
            capitalize_next = false;
        } else {
            // Safe: chars always have at least one lowercase variant
            if let Some(lower_c) = c.to_lowercase().next() {
                result.push(lower_c);
            }
        }
    }

    result
}

pub fn underscore_filter(value: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;

    for (i, c) in value.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('_');
            }
            // Safe: chars always have at least one lowercase variant
            if let Some(lower_c) = c.to_lowercase().next() {
                result.push(lower_c);
            }
            prev_is_upper = true;
        } else if c == '-' || c == ' ' {
            result.push('_');
            prev_is_upper = false;
        } else {
            result.push(c);
            prev_is_upper = false;
        }
    }

    result
}

pub fn dasherize_filter(value: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;

    for (i, c) in value.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('-');
            }
            // Safe: chars always have at least one lowercase variant
            if let Some(lower_c) = c.to_lowercase().next() {
                result.push(lower_c);
            }
            prev_is_upper = true;
        } else if c == '_' || c == ' ' {
            result.push('-');
            prev_is_upper = false;
        } else {
            result.push(c);
            prev_is_upper = false;
        }
    }

    result
}

pub fn humanize_filter(value: &str) -> String {
    let mut result = String::new();
    let mut first = true;

    for c in value.chars() {
        if c == '_' || c == '-' {
            result.push(' ');
        } else if first {
            // Safe: chars always have at least one uppercase variant
            if let Some(upper_c) = c.to_uppercase().next() {
                result.push(upper_c);
            }
            first = false;
        } else {
            result.push(c);
        }
    }

    result
}

pub fn titleize_filter(value: &str) -> String {
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
            // Safe: chars always have at least one uppercase variant
            if let Some(upper_c) = c.to_uppercase().next() {
                result.push(upper_c);
            }
            capitalize_next = false;
        } else {
            // Safe: chars always have at least one lowercase variant
            if let Some(lower_c) = c.to_lowercase().next() {
                result.push(lower_c);
            }
        }
    }

    result
}

pub fn upcase_first_filter(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => {
            let mut result = String::new();
            // Safe: chars always have at least one uppercase variant
            if let Some(upper_c) = c.to_uppercase().next() {
                result.push(upper_c);
            }
            result.push_str(chars.as_str());
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camelize() {
        assert_eq!(camelize_filter("hello_world"), "HelloWorld");
        assert_eq!(camelize_filter("hello-world"), "HelloWorld");
        assert_eq!(camelize_filter("hello world"), "HelloWorld");
        assert_eq!(camelize_filter("hello_world_foo"), "HelloWorldFoo");
        assert_eq!(camelize_filter(""), "");
    }

    #[test]
    fn test_underscore() {
        assert_eq!(underscore_filter("HelloWorld"), "hello_world");
        assert_eq!(underscore_filter("hello-world"), "hello_world");
        assert_eq!(underscore_filter("hello world"), "hello_world");
        assert_eq!(underscore_filter("HelloWorldFoo"), "hello_world_foo");
        assert_eq!(underscore_filter(""), "");
    }

    #[test]
    fn test_dasherize() {
        assert_eq!(dasherize_filter("HelloWorld"), "hello-world");
        assert_eq!(dasherize_filter("hello_world"), "hello-world");
        assert_eq!(dasherize_filter("hello world"), "hello-world");
        assert_eq!(dasherize_filter("HelloWorldFoo"), "hello-world-foo");
        assert_eq!(dasherize_filter(""), "");
    }

    #[test]
    fn test_humanize() {
        assert_eq!(humanize_filter("hello_world"), "Hello world");
        assert_eq!(humanize_filter("hello-world"), "Hello world");
        assert_eq!(humanize_filter("hello_world_foo"), "Hello world foo");
        assert_eq!(humanize_filter(""), "");
    }

    #[test]
    fn test_titleize() {
        assert_eq!(titleize_filter("hello_world"), "Hello World");
        assert_eq!(titleize_filter("hello-world"), "Hello World");
        assert_eq!(titleize_filter("hello world"), "Hello World");
        assert_eq!(titleize_filter("hello_world_foo"), "Hello World Foo");
        assert_eq!(titleize_filter(""), "");
    }

    #[test]
    fn test_upcase_first() {
        assert_eq!(upcase_first_filter("hello"), "Hello");
        assert_eq!(upcase_first_filter("Hello"), "Hello");
        assert_eq!(upcase_first_filter("hello world"), "Hello world");
        assert_eq!(upcase_first_filter(""), "");
    }
}
