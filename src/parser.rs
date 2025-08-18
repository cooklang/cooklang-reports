//! Global parser instance for efficient recipe parsing.
//!
//! This module provides a singleton `CooklangParser` instance that is initialized once
//! and reused throughout the application, improving performance by avoiding repeated
//! parser initialization.

use cooklang::{Converter, CooklangParser};
use std::sync::OnceLock;

/// Global `CooklangParser` instance that is initialized once and reused throughout the application.
/// This improves performance by avoiding repeated parser initialization.
static PARSER: OnceLock<CooklangParser> = OnceLock::new();

/// Get the global `CooklangParser` instance.
///
/// The parser is initialized with all extensions enabled and an empty converter
/// (no unit conversions). This allows parsing all recipe features while keeping
/// units as-is without conversions.
/// This function is thread-safe and will only initialize the parser once.
///
/// # Example
/// ```no_run
/// use cooklang_reports::parser::get_parser;
///
/// let parser = get_parser();
/// let (recipe, warnings) = parser.parse("@eggs{2}").into_result().unwrap();
/// ```
pub fn get_parser() -> &'static CooklangParser {
    PARSER.get_or_init(CooklangParser::canonical)
}

/// Get the converter from the global parser.
///
/// This is a convenience function that returns the converter from the global parser instance.
#[must_use]
pub fn get_converter() -> &'static Converter {
    get_parser().converter()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_parser_singleton() {
        // Get parser multiple times and ensure it's the same instance
        let parser1 = get_parser();
        let parser2 = get_parser();

        // Both should be the same instance (same memory address)
        assert!(std::ptr::eq(parser1, parser2));
    }

    #[test]
    fn test_parser_works() {
        let parser = get_parser();
        let recipe = "@eggs{2} and @milk{250%ml}";
        let (parsed, _warnings) = parser.parse(recipe).into_result().unwrap();

        assert_eq!(parsed.ingredients.len(), 2);
        assert_eq!(parsed.ingredients[0].name, "eggs");
        assert_eq!(parsed.ingredients[1].name, "milk");
    }

    #[test]
    fn test_converter_access() {
        let converter = get_converter();
        // Just ensure we can get the converter without panic
        // The converter should be accessible and functional
        // Try to iterate over units to make sure it works
        let _units: Vec<_> = converter.all_units().collect();
        // If we get here without panic, the test passes
    }
}
