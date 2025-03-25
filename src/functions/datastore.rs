use minijinja::{Error as MiniError, ErrorKind, State, Value as MiniValue};
use serde::Deserialize;
use yaml_datastore::Datastore;

fn non_key_error(message: &str) -> MiniError {
    MiniError::new(ErrorKind::NonKey, message.to_owned())
}

pub fn get_from_datastore(state: &State, keypath: &str) -> Result<MiniValue, MiniError> {
    // Lookup datastore. If it exists, convert it from Value to Datastore. Then get the key.
    // This is kinda terse, but the expanded version isn't really any better IMO.
    state
        .lookup("datastore")
        .ok_or(non_key_error("bad datastore"))
        .and_then(|x| Option::<Datastore>::deserialize(x)?.ok_or(non_key_error("no datastore")))?
        .get(keypath)
        .map_err(|_| non_key_error("no key found in datastore"))
}
