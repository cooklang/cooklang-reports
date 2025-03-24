//! Parsing for key paths, which are of the form:
//!
//!     dir/file/outer.middle.inner
//!
//! This would read the key outer -> middle -> inner from the file `dir/file`.
//!
//! This means that at least a single slash is required.
//!
//! How do we deal with slashes in YAML keys? We should just disallow them.
//!
//! Also just for a note, because we're using dots for the keys, we also technically disallow those.
use serde::Deserialize;
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};
use thiserror::Error;

const YAML_EXTENSION: &str = "yml";
const KEYPATH_DELIMITER: &str = "/";
const KEY_DELIMITER: &str = ".";

#[derive(Error, Debug)]
pub enum KeyPathParseError {
    #[error("no delimiter found. missing delimiter ({}).", KEYPATH_DELIMITER)]
    NoDelimiter,

    #[error("no data before delimiter ({})", KEYPATH_DELIMITER)]
    NoPath,
}

#[derive(Deserialize, Debug)]
pub struct KeyPath {
    path: PathBuf,
    key: Vec<String>,
}

impl KeyPath {
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn key_vec(&self) -> Vec<String> {
        self.key.to_owned()
    }
}

impl TryFrom<&str> for KeyPath {
    type Error = KeyPathParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (path, key) = value
            .rsplit_once(KEYPATH_DELIMITER)
            .ok_or(KeyPathParseError::NoDelimiter)?;

        if path.is_empty() {
            return Err(KeyPathParseError::NoPath);
        }

        Ok(Self {
            path: PathBuf::from(path).with_extension(YAML_EXTENSION),
            key: key
                .split_terminator(KEY_DELIMITER)
                .map(String::from)
                .collect(),
        })
    }
}

impl Display for KeyPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}",
            self.path.with_extension("").display(),
            self.key.join(".")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_and_simple_key() {
        let input = "path/key";
        let result = KeyPath::try_from(input).unwrap();
        assert_eq!(
            result.path,
            PathBuf::from("path").with_extension(YAML_EXTENSION)
        );
        assert_eq!(result.key, vec!["key"]);
        assert_eq!(input, result.to_string());
    }

    #[test]
    fn path_and_simple_key() {
        let input = "dir/path/key";
        let result = KeyPath::try_from(input).unwrap();
        assert_eq!(
            result.path,
            PathBuf::from("dir/path").with_extension(YAML_EXTENSION)
        );
        assert_eq!(result.key, vec!["key"]);
        assert_eq!(input, result.to_string());
    }

    #[test]
    fn file_and_nested_key() {
        let input = "path/key.nested.value";
        let result = KeyPath::try_from(input).unwrap();
        assert_eq!(
            result.path,
            PathBuf::from("path").with_extension(YAML_EXTENSION)
        );
        assert_eq!(result.key, vec!["key", "nested", "value"]);
        assert_eq!(input, result.to_string());
    }

    #[test]
    fn path_and_nested_key() {
        let input = "dir/path/key.nested.value";
        let result = KeyPath::try_from(input).unwrap();
        assert_eq!(
            result.path,
            PathBuf::from("dir/path").with_extension(YAML_EXTENSION)
        );
        assert_eq!(result.key, vec!["key", "nested", "value"]);
        assert_eq!(input, result.to_string());
    }

    #[test]
    fn file_and_empty_key() {
        let input = "path/";
        let result = KeyPath::try_from(input).unwrap();
        assert_eq!(
            result.path,
            PathBuf::from("path").with_extension(YAML_EXTENSION)
        );
        assert_eq!(result.key, Vec::<&str>::new());
        assert_eq!(input, result.to_string());
    }

    #[test]
    fn err_no_path_some_key() {
        let result = KeyPath::try_from("/key").unwrap_err();
        assert!(matches!(result, KeyPathParseError::NoPath));
    }

    #[test]
    fn err_no_delimiter() {
        let values = vec!["inner", "middle.inner", "outer.middle.inner"];
        for value in values {
            let result = KeyPath::try_from(value).unwrap_err();
            assert!(matches!(result, KeyPathParseError::NoDelimiter));
        }
    }

    #[test]
    fn err_no_path_or_key() {
        let result = KeyPath::try_from("/").unwrap_err();
        assert!(matches!(result, KeyPathParseError::NoPath));
    }
}
