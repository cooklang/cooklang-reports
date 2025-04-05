use cooklang::Quantity;
use serde::{Serialize, Serializer};

// Because this is getting invoked automatically, I don't think I can fix this lint.
#[allow(clippy::ref_option)]
pub(crate) fn quantity_to_string<S>(
    value: &Option<Quantity>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(
        match value.as_ref() {
            Some(quantity) => quantity.to_string(),
            None => String::new(),
        }
        .as_str(),
    )
}

/// An Ingredient that's used here instead of the parser's one, for template access.
#[derive(Debug, Serialize)]
pub(crate) struct Ingredient<'a> {
    pub(crate) name: &'a str,

    #[serde(serialize_with = "quantity_to_string")]
    pub(crate) quantity: &'a Option<Quantity>,
}

impl<'a> From<&'a cooklang::Ingredient> for Ingredient<'a> {
    fn from(value: &'a cooklang::Ingredient) -> Self {
        Ingredient {
            name: &value.name,
            quantity: &value.quantity,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Section<'a> {
    name: Option<&'a str>,
    content: Vec<Content<'a>>,
}

impl<'a> Section<'a> {
    pub(crate) fn from_recipe_section(
        recipe: &'a cooklang::ScaledRecipe,
        section: &'a cooklang::Section,
    ) -> Self {
        Self {
            name: section.name.as_deref(),
            content: section
                .content
                .iter()
                .map(|x| Content::from_recipe_content(recipe, x))
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
enum Content<'a> {
    Step(Step<'a>),
    Text(&'a str),
}

impl<'a> Content<'a> {
    fn from_recipe_content(
        recipe: &'a cooklang::ScaledRecipe,
        content: &'a cooklang::Content,
    ) -> Self {
        match content {
            cooklang::Content::Step(step) => Content::Step(Step::from_recipe_step(recipe, step)),
            cooklang::Content::Text(text) => Content::Text(text),
        }
    }
}

/// A step in a recipe
#[derive(Debug, Serialize)]
struct Step<'a> {
    items: Vec<Item<'a>>,
    number: u32,
}

impl<'a> Step<'a> {
    fn from_recipe_step(recipe: &'a cooklang::ScaledRecipe, step: &'a cooklang::Step) -> Self {
        Self {
            items: step
                .items
                .iter()
                .map(|item| Item::from_recipe_item(recipe, item))
                .collect(),
            number: step.number,
        }
    }
}

/// A cooklang step item.
///
/// Cooklang provides these as indices, but we want them as actual references.
#[derive(Debug, Serialize)]
enum Item<'a> {
    Text(&'a str),
    Ingredient(Ingredient<'a>),
    Cookware(&'a cooklang::Cookware),
    //Timer,          // TODO
    //InlineQuantity, // TODO; probably won't implement
}

// I hate it but the Ingredients are duplicated here.
// The only way I could think to avoid it would be to initialize the vector of ingredients
// prior to trying to construct my recipe thing.
// Even if I wanted to use Arc or something to have this refer to that, I'd then still have to
// have it available...
// For what it's worth I'm only copying my shim
impl<'a> Item<'a> {
    fn from_recipe_item(recipe: &'a cooklang::ScaledRecipe, item: &'a cooklang::Item) -> Self {
        match item {
            cooklang::Item::Text { value } => Self::Text(value),
            cooklang::Item::Ingredient { index } => {
                Self::Ingredient(Ingredient::from(&recipe.ingredients[*index]))
            }
            cooklang::Item::Cookware { index } => Self::Cookware(&recipe.cookware[*index]),
            cooklang::Item::Timer { index: _ } => todo!(),
            cooklang::Item::InlineQuantity { index: _ } => todo!(),
        }
    }
}
