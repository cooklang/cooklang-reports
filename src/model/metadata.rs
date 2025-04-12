use minijinja::value::ObjectExt;
use serde::Serialize;

/// Wrapper for [`cooklang::Metadata`] for reporting.
///
/// # Usage
///
/// Constructed from [`cooklang::Metadata`] and can be converted into [`minijinja::Value`].
///
/// If you have a `metadata`, then the following are valid ways to use it. Below, `<key>` refers
/// to a valid key within the metadata YAML.
///
/// ```text
/// {{ metadata }}
/// {{ metadata.<key>> }}
/// ```
///
/// The usage `{{ metadata }}` will render the entire frontmatter block, with `---` before and after it.
///
/// The metadata may also be iterated over in a template, which will enumerate all its keys. This
/// can be passed to `items`, which will split it into keys and values:
///
/// ```text
/// {% for (key, value) in metadata | items %}
/// {{ key }}: {{ value }}
/// {% endfor %}
/// ```
#[derive(Clone, Debug, Serialize)]
pub struct Metadata(cooklang::Metadata);

impl From<cooklang::Metadata> for Metadata {
    fn from(metadata: cooklang::Metadata) -> Self {
        Self(metadata)
    }
}

impl From<Metadata> for minijinja::Value {
    fn from(val: Metadata) -> Self {
        minijinja::Value::from_object(val)
    }
}

impl minijinja::value::Object for Metadata {
    fn repr(self: &std::sync::Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Map
    }

    fn get_value(self: &std::sync::Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        self.0
            .get(key.as_str()?)
            .map(minijinja::Value::from_serialize)
    }

    fn enumerate(self: &std::sync::Arc<Self>) -> minijinja::value::Enumerator {
        // let keys = ;
        self.mapped_enumerator(|this| {
            Box::new(
                this.0
                    .map
                    .keys()
                    .map(|x| x.as_str())
                    .map(minijinja::Value::from),
            )
        })
    }

    /// Render this YAML metadata. The entire block is omitted if there is no metadata.
    fn render(self: &std::sync::Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where
        Self: Sized + 'static,
    {
        if !self.0.map.is_empty() {
            let yaml_string = serde_yaml::to_string(&self.0.map).map_err(|_| std::fmt::Error)?;
            writeln!(f, "---")?;
            write!(f, "{yaml_string}")?;
            writeln!(f, "---")?;
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

    const RECIPE: &str = "---\ntitle: Food\nauthor: Chef\n---\n\n";
    const LOOP_TEMPLATE: &str =
        "{% for (key, value) in metadata | items %}{{ key }}: {{ value }}\n{% endfor %}";

    #[test_case(RECIPE, "{{ metadata }}", "---\ntitle: Food\nauthor: Chef\n---\n"; "as-is")]
    #[test_case(RECIPE, LOOP_TEMPLATE, "title: Food\nauthor: Chef\n"; "enumerated")]
    #[test_case(RECIPE, "{{ metadata.title }}", "Food"; "get title key by name")]
    #[test_case(RECIPE, "{{ metadata.author }}", "Chef"; "get author key by name")]
    #[test_case(RECIPE, "{{ metadata.nothing }}", ""; "get invalid key by name")]
    fn metadata(recipe: &str, template: &str, expected: &str) {
        let (recipe, env) = get_recipe_and_env(recipe, template);
        let context = context! {
            metadata => Value::from(Metadata(recipe.metadata)),
        };

        let template = env.get_template("test").unwrap();
        assert_eq!(expected, template.render(context).unwrap());
    }
}
