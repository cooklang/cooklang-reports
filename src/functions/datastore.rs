use minijinja::{Error as MiniError, State, Value as MiniValue};
use serde::Deserialize;
use yaml_datastore::Datastore;

pub fn get_from_datastore(state: &State, args: &[MiniValue]) -> Result<MiniValue, MiniError> {
    if args.len() != 1 {
        return Err(MiniError::new(
            minijinja::ErrorKind::InvalidOperation,
            "db requires exactly 1 argument: key-path",
        ));
    }

    // Validate that the argument is a string, but we don't need to store it
    args[0].as_str().ok_or_else(|| {
        MiniError::new(
            minijinja::ErrorKind::InvalidOperation,
            "argument must be a string (key-path)",
        )
    })?;

    // if let Some(datastore) = state.lookup("datastore") {}
    // let datastore = state.lookup("datastore").and_then(f)
    // if let Some(datastore_value) = datastore {
    //     let datastore = Option::<Datastore>::deserialize(datastore_value).unwrap();
    // }

    let recipe_template = state.lookup("recipe_template").ok_or_else(|| {
        MiniError::new(
            minijinja::ErrorKind::InvalidOperation,
            "recipe_template not found in context",
        )
    })?;

    recipe_template.call_method(state, "db", args)
}
