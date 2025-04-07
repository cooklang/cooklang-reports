pub(crate) mod cookware;
pub(crate) mod ingredient;
pub(crate) mod item;
pub(crate) mod quantity;
pub(crate) mod step;

pub(crate) use cookware::Cookware;
pub(crate) use ingredient::Ingredient;
pub(crate) use item::Item;
pub(crate) use quantity::Quantity;
pub(crate) use step::Step;

#[cfg(test)]
mod tests {
    use cooklang::{Converter, CooklangParser, Extensions, ScaledRecipe};
    use minijinja::Environment;

    #[cfg(test)]
    pub(super) fn get_recipe_and_env<'a>(
        recipe: &str,
        template: &'a str,
    ) -> (ScaledRecipe, Environment<'a>) {
        let recipe_parser = CooklangParser::new(Extensions::all(), Converter::default());
        let (unscaled_recipe, _warnings) = recipe_parser.parse(recipe).into_result().unwrap();
        let recipe = unscaled_recipe.scale(1.into(), &Converter::default());

        let mut env: Environment<'a> = Environment::new();
        env.add_template("test", template).unwrap();

        (recipe, env)
    }
}
