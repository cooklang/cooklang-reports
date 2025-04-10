mod content;
mod content_list;
mod cookware;
mod ingredient;
mod item;
mod quantity;
mod section;
mod step;

pub use content::Content;
pub use content_list::ContentList;
pub use cookware::Cookware;
pub use ingredient::Ingredient;
pub use item::Item;
pub use quantity::Quantity;
pub use section::Section;
pub use step::Step;

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
