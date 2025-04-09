use super::ContentList;
use std::fmt::Display;

#[derive(Clone, Debug)]
pub(crate) struct Section {
    name: Option<String>,
    content: ContentList,
}

impl Section {
    pub(super) fn from_recipe_section(
        recipe: &cooklang::ScaledRecipe,
        section: &cooklang::Section,
    ) -> Self {
        Self {
            name: section.name.clone(),
            content: ContentList::from_recipe_contents(recipe, section.content.clone()),
        }
    }

    pub(super) fn from_recipe_sections(recipe: &cooklang::ScaledRecipe) -> Vec<Self> {
        recipe
            .sections
            .iter()
            .map(|section| Self::from_recipe_section(recipe, section))
            .collect()
    }
}

impl minijinja::value::Object for Section {
    fn repr(self: &std::sync::Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Seq
    }

    fn get_value(self: &std::sync::Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        match key.as_str()? {
            "name" if self.name.is_some() => Some(minijinja::Value::from(self.name.clone())),
            "name" if self.name.is_none() => Some(minijinja::Value::from("")),
            _ => self
                .content
                .get(key.as_usize()?)
                .cloned()
                .map(minijinja::Value::from_object),
        }
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
        for content in self.content.iter() {
            writeln!(f, "{content}\n")?;
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
    #[test_case("= Intro\n\n> This is something", "{{ section.name }}", "Intro"; "named name")]
    #[test_case("> This is something", "{{ section.name }}", ""; "unnamed name")]
    fn section(recipe: &str, template: &str, expected: &str) {
        let (recipe, env) = get_recipe_and_env(recipe, template);
        let context = context! {
            section => Value::from_object(Section::from_recipe_section(&recipe, &recipe.sections[0]))
        };

        let template = env.get_template("test").unwrap();
        assert_eq!(expected, template.render(context).unwrap());
    }
}
