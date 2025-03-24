use crate::key_path::KeyPath;
use minijinja::{Error as MiniError, ErrorKind, State, Value as MiniValue};
use serde::Deserialize;
use yaml_datastore::Datastore;

fn non_key_error(message: &str) -> MiniError {
    MiniError::new(ErrorKind::NonKey, message.to_owned())
}

pub fn get_from_datastore(state: &State, key_path: &str) -> Result<MiniValue, MiniError> {
    let key_path = KeyPath::try_from(key_path).map_err(|_| non_key_error("invalid key"))?;

    // Lookup datastore. If it exists, convert it from Value to Datastore. Then get the key.
    // This is kinda terse, but the expanded version isn't really any better IMO.
    Ok(state
        .lookup("datastore")
        .ok_or(non_key_error("bad datastore"))
        .and_then(|x| Option::<Datastore>::deserialize(x)?.ok_or(non_key_error("no datastore")))?
        .get_with_key_vec(&*key_path.path(), &key_path.key_vec())
        .map_err(|_| non_key_error("no key found in datastore"))?)
}
