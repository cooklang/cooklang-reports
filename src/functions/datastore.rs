use minijinja::{Error as MiniError, State, Value as MiniValue};

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

    let recipe_template = state.lookup("recipe_template").ok_or_else(|| {
        MiniError::new(
            minijinja::ErrorKind::InvalidOperation,
            "recipe_template not found in context",
        )
    })?;

    recipe_template.call_method(state, "db", args)
}
