//! Public extension hook for registering additional minijinja functions/filters.
//!
//! Implement [`ConfigExtension`] on a type that owns the state your custom
//! functions need (e.g. an HTTP client). Pass an instance to
//! [`crate::config::Config::with_extension`] to register the functions on
//! the template's minijinja environment.

use minijinja::Environment;

/// Register additional minijinja functions or filters on the template environment.
pub trait ConfigExtension: Send + Sync {
    /// Called during template environment construction. Implementations
    /// register functions and filters via [`Environment::add_function`] etc.
    fn register(&self, env: &mut Environment<'_>);
}
