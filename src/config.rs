use std::path::PathBuf;

pub struct Config {
    scale: u32,
    datastore_path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scale: 1,
            datastore_path: None,
        }
    }
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

pub struct ConfigBuilder {
    scale: u32,
    datastore_path: Option<PathBuf>,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self {
            scale: 1,
            datastore_path: None,
        }
    }
}

impl ConfigBuilder {
    fn scale(&mut self, scale: u32) -> &mut Self {
        self.scale = scale;
        self
    }

    fn datastore_path<P: Into<PathBuf>>(&mut self, datstore_path: P) -> &mut Self {
        self.datastore_path = Some(datstore_path.into());
        self
    }
}
