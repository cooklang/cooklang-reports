use super::Content;
use std::fmt::Display;

#[derive(Clone, Debug)]
pub(crate) struct Section {
    name: Option<String>,
    content: Vec<Content>,
}

impl Section {
    pub(super) fn from_recipe_section(
        recipe: &cooklang::ScaledRecipe,
        step: cooklang::Section,
    ) -> Self {
        Self {
            name: step.name,
            content: step
                .content
                .into_iter()
                .map(|content| Content::from_recipe_content(recipe, content))
                .collect(),
        }
    }
}

impl minijinja::value::Object for Section {
    fn repr(self: &std::sync::Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Seq
    }

    fn get_value(self: &std::sync::Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        self.content
            .get(key.as_usize()?)
            .cloned()
            .map(minijinja::Value::from_object)
    }

    fn enumerate(self: &std::sync::Arc<Self>) -> minijinja::value::Enumerator {
        minijinja::value::Enumerator::Seq(self.content.len())
    }

    fn render(self: &std::sync::Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where
        Self: Sized + 'static,
    {
        self.fmt(f)
    }
}

impl Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            Some(name) => write!(f, "= {name}\n\n"),
            None => write!(f, "= Recipe\n\n"),
        }?;
        for content in &self.content {
            content.fmt(f)?;
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

    const NAMED_TEST: (&str, &str) = (
        "= Intro\n\n> This is some intro.\nIt is not interesting.",
        "= Intro\n\nThis is some intro. It is not interesting.\n\n",
    );
    const UNNAMED_TEST: (&str, &str) = (
        "Crack an @egg.\n\nCook it.",
        "= Recipe\n\n1. Crack an egg.\n\n2. Cook it.\n\n",
    );

    #[test_case(NAMED_TEST.0, "{{ section }}", NAMED_TEST.1; "named")]
    #[test_case(UNNAMED_TEST.0, "{{ section }}", UNNAMED_TEST.1; "unnamed")]
    fn section(recipe: &str, template: &str, expected: &str) {
        let (recipe, env) = get_recipe_and_env(recipe, template);

        // Build context
        let context = context! {
            section => Value::from_object(Section::from_recipe_section(&recipe, recipe.sections[0].clone()))
        };

        let template = env.get_template("test").unwrap();
        assert_eq!(expected, template.render(context).unwrap());
    }
}
