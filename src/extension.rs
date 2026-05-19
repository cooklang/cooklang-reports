//! Public extension hook for registering additional minijinja functions/filters.
//!
//! Implement [`ConfigExtension`] on a type that owns the state your custom
//! functions need (e.g. an HTTP client). Pass an instance to
//! [`crate::config::Config::with_extension`] to register the functions on
//! the template's minijinja environment.

use minijinja::Environment;

/// Register additional minijinja functions or filters on the template environment.
///
/// # Ordering
///
/// Extensions are invoked **after** all built-in functions and filters have been
/// registered, in the order they were added to the `Config` via
/// [`crate::Config::with_extension`]. Because minijinja silently overwrites on
/// duplicate names, a later extension can intentionally shadow an earlier
/// extension or a built-in by registering a function with the same name.
///
/// # Errors
///
/// `register` returns `()`. If your setup work can fail, pre-validate the
/// state owned by your extension type before calling
/// [`crate::Config::with_extension`], or panic from inside `register`. A
/// fallible variant may be added in a future release.
pub trait ConfigExtension: Send + Sync {
    /// Called once during template environment construction.
    ///
    /// Implementations register functions and filters via
    /// [`Environment::add_function`], [`Environment::add_filter`], etc.
    fn register(&self, env: &mut Environment<'_>);
}
