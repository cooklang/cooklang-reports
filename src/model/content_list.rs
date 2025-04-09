use std::fmt::Display;

use super::Content;

#[derive(Clone, Debug)]
pub(crate) struct ContentList(Vec<Content>);

impl ContentList {
    pub(crate) fn from_recipe_contents(
        recipe: &cooklang::ScaledRecipe,
        contents: Vec<cooklang::Content>,
    ) -> Self {
        Self(
            contents
                .into_iter()
                .map(|content| Content::from_recipe_content(recipe, content))
                .collect(),
        )
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn get(&self, index: usize) -> Option<&Content> {
        self.0.get(index)
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, Content> {
        self.0.iter()
    }
}

impl minijinja::value::Object for ContentList {
    fn repr(self: &std::sync::Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Seq
    }

    fn get_value(self: &std::sync::Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        self.0
            .get(key.as_usize()?)
            .cloned()
            .map(minijinja::Value::from_object)
    }

    fn enumerate(self: &std::sync::Arc<Self>) -> minijinja::value::Enumerator {
        minijinja::value::Enumerator::Seq(self.0.len())
    }

    fn render(self: &std::sync::Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where
        Self: Sized + 'static,
    {
        self.fmt(f)
    }
}

impl Display for ContentList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for content in &self.0 {
            writeln!(f, "{content}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::tests::get_recipe_and_env;
    use minijinja::{Value, context};
    use test_case::test_case;

    const ITERATE_TEMPLATE: &str = "{% for content in content_list %}{{ content }}\n{% endfor %}";

    #[test_case("> First text.\n\n> Second text.", "{{ content_list }}", "First text.\nSecond text.\n"; "two texts")]
    #[test_case("First step.\n\nSecond step.", "{{ content_list }}", "1. First step.\n2. Second step.\n"; "two steps")]
    #[test_case("> First text.\n\nFirst step.", "{{ content_list }}", "First text.\n1. First step.\n"; "text step")]
    #[test_case("First step.\n\n> First text.", "{{ content_list }}", "1. First step.\nFirst text.\n"; "step text")]
    #[test_case("> First text.\n\n> Second text.", ITERATE_TEMPLATE, "First text.\nSecond text.\n"; "iterate two texts")]
    #[test_case("First step.\n\nSecond step.", ITERATE_TEMPLATE, "1. First step.\n2. Second step.\n"; "iterate two steps")]
    #[test_case("> First text.\n\nFirst step.", ITERATE_TEMPLATE, "First text.\n1. First step.\n"; "iterate text step")]
    #[test_case("First step.\n\n> First text.", ITERATE_TEMPLATE, "1. First step.\nFirst text.\n"; "iterate step text")]
    fn content_list(recipe: &str, template: &str, expected: &str) {
        let (recipe, env) = get_recipe_and_env(recipe, template);
        let context = context! {
            content_list => Value::from_object(ContentList::from_recipe_contents(&recipe, recipe.sections[0].content.clone()))
        };

        let template = env.get_template("test").unwrap();
        assert_eq!(expected, template.render(context).unwrap());
    }
}
