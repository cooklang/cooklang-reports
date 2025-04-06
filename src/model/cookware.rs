//! Model for cookware.
//!
//! Standard display behavior is just to use the display name, which is either the
//! name, or alias, if present. Quantity and notes are not included, but are accessible.
#[derive(Debug, Clone)]
pub(crate) struct Cookware(cooklang::Cookware);

impl From<cooklang::Cookware> for Cookware {
    fn from(cookware: cooklang::Cookware) -> Self {
        Self(cookware)
    }
}

impl minijinja::value::Object for Cookware {
    fn repr(self: &std::sync::Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Plain
    }

    fn get_value(self: &std::sync::Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        match key.as_str()? {
            "note" => self.0.note.as_ref().map(minijinja::Value::from),
            "name" => Some(minijinja::Value::from(&self.0.name)),
            "alias" => self.0.alias.as_ref().map(minijinja::Value::from),
            "quantity" => self
                .0
                .quantity
                .as_ref()
                .map(ToString::to_string)
                .map(minijinja::Value::from),
            _ => None,
        }
    }

    fn render(self: &std::sync::Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where
        Self: Sized + 'static,
    {
        write!(f, "{}", self.0.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cooklang::{Converter, CooklangParser, Extensions, ScaledRecipe};
    use minijinja::{Environment, Value, context};
    use test_case::test_case;

    fn get_recipe_and_env<'a>(recipe: &str, template: &'a str) -> (ScaledRecipe, Environment<'a>) {
        let recipe_parser = CooklangParser::new(Extensions::all(), Converter::default());
        let (unscaled_recipe, _warnings) = recipe_parser.parse(recipe).into_result().unwrap();
        let recipe = unscaled_recipe.scale(1.into(), &Converter::default());

        let mut env: Environment<'a> = Environment::new();
        env.add_template("test", template).unwrap();

        (recipe, env)
    }

    #[test_case("Crack @egg{1} into #frying pan{}.", "{{ cookware }}", "frying pan"; "just name")]
    #[test_case("Crack @egg{1} into #frying pan{1}.", "{{ cookware }}", "frying pan"; "name and quantity")]
    #[test_case("Crack @egg{1} into #frying pan|pan{}.", "{{ cookware }}", "pan"; "aliased name")]
    #[test_case("Crack @egg{1} into #frying pan|pan{}(greased).", "{{ cookware.name }}", "frying pan"; "direct name")]
    #[test_case("Crack @egg{1} into #frying pan|pan{}(greased).", "{{ cookware.alias }}", "pan"; "direct alias")]
    #[test_case("Crack @egg{1} into #frying pan|pan{}(greased).", "{{ cookware.note }}", "greased"; "with note")]
    #[test_case("Crack @egg{1} into #frying pan|pan{1}(greased).", "{{ cookware.quantity }}", "1"; "direct quantity")]

    fn cookware(recipe: &str, template: &str, result: &str) {
        let (recipe, env) = get_recipe_and_env(recipe, template);

        // Build context
        let context = context! {
            cookware => Value::from_object(Cookware::from(recipe.cookware[0].clone()))
        };

        let template = env.get_template("test").unwrap();
        assert_eq!(result, template.render(context).unwrap());
    }
}
