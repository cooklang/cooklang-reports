use crate::key_path::KeyPath;
use minijinja::{Error as MiniError, ErrorKind, State, Value as MiniValue};
use serde::Deserialize;
use yaml_datastore::Datastore;

pub fn get_from_datastore(state: &State, key_path: &str) -> Result<MiniValue, MiniError> {
    let key_path = KeyPath::try_from(key_path).unwrap();

    // Lookup datastore. If it exists, convert it from Value to Datastore.
    // This is kinda terse, but the expanded version isn't really any better.
    let datastore = state
        .lookup("datastore")
        .ok_or(MiniError::new(ErrorKind::NonKey, "bad datastore"))
        .and_then(|x| {
            Option::<Datastore>::deserialize(x)?
                .ok_or(MiniError::new(ErrorKind::NonKey, "no datastore"))
        })?;

    Ok(datastore
        .get_with_key_vec(
            &*key_path.path(),
            key_path
                .key_vec()
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
        )
        .unwrap())
}
