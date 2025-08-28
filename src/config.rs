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
/// let config = Config::builder().scale(2.0).datastore_path("db").build();
/// ```
pub struct Config {
    pub(crate) scale: f64,
    pub(crate) datastore_path: Option<PathBuf>,
    pub(crate) base_path: Option<PathBuf>,
    pub(crate) aisle_path: Option<PathBuf>,
    pub(crate) pantry_path: Option<PathBuf>,
}

impl Default for Config {
    /// Return a default [`Config`] with a scale of 1, no datastore path, aisle path, pantry path, and base path set to the current working directory.
    fn default() -> Self {
        Self {
            scale: 1.0,
            datastore_path: None,
            base_path: std::env::current_dir().ok(),
            aisle_path: None,
            pantry_path: None,
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
    scale: f64,
    datastore_path: Option<PathBuf>,
    base_path: Option<PathBuf>,
    aisle_path: Option<PathBuf>,
    pantry_path: Option<PathBuf>,
}

impl Default for ConfigBuilder {
    /// Return a default [`ConfigBuilder`] with a scale of 1, no datastore path, aisle path, pantry path, and base path set to the current working directory.
    fn default() -> Self {
        Self {
            scale: 1.0,
            datastore_path: None,
            base_path: std::env::current_dir().ok(),
            aisle_path: None,
            pantry_path: None,
        }
    }
}

impl ConfigBuilder {
    /// Set the scale property. This is used in recipe scaling and is itself passed to the template.
    pub fn scale(&mut self, scale: f64) -> &mut Self {
        self.scale = scale;
        self
    }

    /// Set a path to a [datastore][`yaml_datastore`].
    pub fn datastore_path<P: Into<PathBuf>>(&mut self, datstore_path: P) -> &mut Self {
        self.datastore_path = Some(datstore_path.into());
        self
    }

    /// Set a base path for recipe lookups.
    pub fn base_path<P: Into<PathBuf>>(&mut self, base_path: P) -> &mut Self {
        self.base_path = Some(base_path.into());
        self
    }

    /// Set a path to an aisle configuration file for ingredient categorization.
    pub fn aisle_path<P: Into<PathBuf>>(&mut self, aisle_path: P) -> &mut Self {
        self.aisle_path = Some(aisle_path.into());
        self
    }

    /// Set a path to a pantry configuration file for filtering out pantry items.
    pub fn pantry_path<P: Into<PathBuf>>(&mut self, pantry_path: P) -> &mut Self {
        self.pantry_path = Some(pantry_path.into());
        self
    }

    /// Return a new [`Config`] based on the builder's properties.
    pub fn build(&mut self) -> Config {
        Config {
            scale: self.scale,
            datastore_path: self.datastore_path.clone(),
            base_path: self.base_path.clone(),
            aisle_path: self.aisle_path.clone(),
            pantry_path: self.pantry_path.clone(),
        }
    }
}
