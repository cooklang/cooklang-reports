//! Error types for the cooklang-reports library.

use thiserror::Error;

/// Error type for this crate.
#[derive(Error, Debug)]
pub enum Error {
    /// An error occurred when parsing the recipe.
    #[error("error parsing recipe")]
    RecipeParseError(#[from] cooklang::error::SourceReport),

    /// An error occurred when generating a report from a template.
    #[error("template error")]
    TemplateError(#[from] minijinja::Error),
}

impl Error {
    /// Format the error with full context including source chain and helpful hints
    ///
    /// This method provides comprehensive error formatting that includes:
    /// - The main error message
    /// - The complete chain of error causes
    /// - Template-specific context for common errors
    /// - Helpful suggestions for fixing the error
    ///
    /// # Returns
    /// A formatted string suitable for display to end users with detailed error information.
    ///
    /// # Example
    /// ```no_run
    /// use cooklang_reports::render_template;
    ///
    /// let recipe = "@eggs{2}";
    /// let template = "{% for item in ingredients %}{{ item.name }}{% endfor"; // Missing %}
    ///
    /// match render_template(recipe, template) {
    ///     Ok(result) => println!("{}", result),
    ///     Err(err) => {
    ///         // This will print detailed error information including:
    ///         // - The syntax error
    ///         // - Line and column information
    ///         // - Suggestions for fixing missing closing tags
    ///         eprintln!("{}", err.format_with_source());
    ///     }
    /// }
    /// ```
    ///
    /// # Output Format
    /// The output includes:
    /// - Primary error message with debug info (line numbers, source context)
    /// - Caused by chain (if any)
    /// - Additional details from minijinja
    /// - Context-specific help for common template errors
    #[must_use]
    pub fn format_with_source(&self) -> String {
        use std::fmt::Write;

        let mut output = String::new();

        // Add template-specific context if it's a template error
        if let Error::TemplateError(minijinja_err) = self {
            // First show the actual error message
            let error_detail = minijinja_err.detail().unwrap_or_default();
            if !error_detail.is_empty() {
                let _ = writeln!(output, "Error: {error_detail}");
            }

            // Then show the debug info with source location
            let _ = write!(output, "{}", minijinja_err.display_debug_info());

            // Add helpful hints based on error type
            match minijinja_err.kind() {
                minijinja::ErrorKind::SyntaxError => {
                    output.push_str("\n\nHint: This is a syntax error. Check for:");
                    output.push_str("\n  • Missing closing tags ({% endfor %}, {% endif %}, etc.)");
                    output.push_str("\n  • Invalid Jinja2 syntax");
                    output.push_str("\n  • Unclosed strings or brackets");
                }
                minijinja::ErrorKind::UndefinedError => {
                    output.push_str("\n\nHint: A variable or attribute is undefined. Check that:");
                    output
                        .push_str("\n  • All variables used in the template exist in the context");
                    output.push_str("\n  • Property names are spelled correctly");
                    output.push_str("\n  • You're not trying to access properties on null values");
                }
                minijinja::ErrorKind::InvalidOperation => {
                    // Check if the error message contains specific keywords for better hints
                    let error_str = minijinja_err.to_string();
                    if error_str.contains("Failed to scale recipe") {
                        output.push_str("\n\nHint: Recipe scaling failed. Check that:");
                        output.push_str("\n  • The referenced recipe has the required metadata (servings or yield)");
                        output.push_str(
                            "\n  • The units in the reference match the recipe's metadata",
                        );
                        output.push_str("\n  • The recipe file exists and is valid");
                    } else {
                        output.push_str("\n\nHint: Invalid operation. Check that:");
                        output.push_str("\n  • You're using the correct types for operations");
                        output.push_str("\n  • Functions are called with correct arguments");
                        output.push_str("\n  • Filters are applied to compatible values");
                    }
                }
                minijinja::ErrorKind::NonKey => {
                    output.push_str("\n\nHint: Key not found. Check that:");
                    output.push_str("\n  • The key exists in your datastore");
                    output.push_str("\n  • The key path is spelled correctly");
                    output.push_str("\n  • String transformations are producing the expected keys");
                }
                _ => {}
            }
            // Don't traverse the error chain for template errors since display_debug_info already shows it
            return output;
        }

        // For non-template errors, use the standard display
        let _ = write!(output, "Error: {self:#}");

        // Traverse the error chain for non-template errors
        let mut current_error: &dyn std::error::Error = self;
        while let Some(source) = current_error.source() {
            let _ = write!(output, "\n\nCaused by:\n    {source:#}");
            current_error = source;
        }

        output
    }
}
