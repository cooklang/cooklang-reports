mod content;
mod content_list;
mod cookware;
mod ingredient;
mod item;
mod metadata;
mod quantity;
mod section;
mod step;

pub(crate) use content::Content;
pub(crate) use content_list::ContentList;
pub(crate) use cookware::Cookware;
pub(crate) use ingredient::Ingredient;
pub(crate) use item::Item;
pub(crate) use metadata::Metadata;
pub(crate) use quantity::Quantity;
pub(crate) use section::Section;
pub(crate) use step::Step;

#[cfg(test)]
mod tests {
    use cooklang::{Converter, CooklangParser, Extensions, Recipe};
    use minijinja::Environment;

    #[cfg(test)]
    pub(super) fn get_recipe_and_env<'a>(
        recipe: &str,
        template: &'a str,
    ) -> (Recipe, Environment<'a>) {
        let recipe_parser = CooklangParser::new(Extensions::all(), Converter::default());
        let (mut recipe, _warnings) = recipe_parser.parse(recipe).into_result().unwrap();
        recipe.scale(1.into(), &Converter::default());

        let mut env: Environment<'a> = Environment::new();
        env.add_template("test", template).unwrap();

        (recipe, env)
    }
}
