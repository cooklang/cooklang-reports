//! Configuration struct for report generation.
use std::path::PathBuf;

/// Struct for template configuration.
///
/// At present, configuration contains a scale and an optional path to a [datastore][`yaml_datastore`].
///
/// Construct via [`ConfigBuilder`] or [`default()`][`Self::default`].
///
/// # Examples
///
/// Use [`Config::builder()`][`Config::builder`] to get a [`ConfigBuilder`] and then chain calls to set the desired configuration.
/// Call [`build()`][`ConfigBuilder::build`] to get a `Config`.
///
/// ```
/// use cooklang_reports::config::Config;
/// let config = Config::builder().scale(2).datastore_path("db").build();
/// ```
pub struct Config {
    pub(crate) scale: u32,
    pub(crate) datastore_path: Option<PathBuf>,
}

impl Default for Config {
    /// Return a default [`Config`] with a scale of 1 and no datastore path.
    fn default() -> Self {
        Self {
            scale: 1,
            datastore_path: None,
        }
    }
}

impl Config {
    /// Return a [`ConfigBuilder`] for building a `Config`.
    #[must_use]
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

/// Builder for building a [`Config`].
pub struct ConfigBuilder {
    scale: u32,
    datastore_path: Option<PathBuf>,
}

impl Default for ConfigBuilder {
    /// Return a default [`ConfigBuilder`] with a scale of 1 and no datastore path.
    fn default() -> Self {
        Self {
            scale: 1,
            datastore_path: None,
        }
    }
}

impl ConfigBuilder {
    /// Set the scale property. This is used in recipe scaling and is itself passed to the template.
    pub fn scale(&mut self, scale: u32) -> &mut Self {
        self.scale = scale;
        self
    }

    /// Set a path to a [datastore][`yaml_datastore`].
    pub fn datastore_path<P: Into<PathBuf>>(&mut self, datstore_path: P) -> &mut Self {
        self.datastore_path = Some(datstore_path.into());
        self
    }

    /// Return a new [`Config`] based on the builder's properties.
    pub fn build(&mut self) -> Config {
        Config {
            scale: self.scale,
            datastore_path: self.datastore_path.clone(),
        }
    }
}
