use std::fmt::Display;

#[derive(Debug)]
pub(crate) struct Quantity(cooklang::Quantity);

impl From<cooklang::Quantity> for Quantity {
    fn from(quantity: cooklang::Quantity) -> Self {
        Self(quantity)
    }
}

impl Display for Quantity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl minijinja::value::Object for Quantity {
    fn repr(self: &std::sync::Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Plain
    }

    fn render(self: &std::sync::Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where
        Self: Sized + 'static,
    {
        self.fmt(f)
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

    #[test_case("Crack @egg{1} into pan.", "{{ quantity }}", "1"; "number without unit")]
    #[test_case("Pour @flour{100%g} into bowl.", "{{ quantity }}", "100 g"; "number with unit")]
    #[test_case("Crack @eggs{1-2} into pan.", "{{ quantity }}", "1-2"; "range without unit")]
    #[test_case("Pour @olive oil{1-2%tsp} into pan.", "{{ quantity }}", "1-2 tsp"; "range with unit")]
    #[test_case("Peel @garlic{clove}.", "{{ quantity }}", "clove"; "text without unit")]
    #[test_case("Peel @garlic{clove%big}.", "{{ quantity }}", "clove big"; "text with unit")]

    fn quantity(recipe: &str, template: &str, result: &str) {
        let (recipe, env) = get_recipe_and_env(recipe, template);

        // Build context
        let context = context! {
            quantity => Value::from_object(Quantity::from(recipe.ingredients[0].quantity.as_ref().unwrap().clone()))
        };

        let template = env.get_template("test").unwrap();
        assert_eq!(result, template.render(context).unwrap());
    }
}
