// Integration test: the crate's `missing_docs` and `unwrap_used` lints are meant
// for the library surface, not for test scaffolding.
#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

use cooklang_reports::render_template;

#[test]
fn tojson_serializes_template_values() {
    let out =
        render_template("@flour{200%g}", r#"{{ {"a": [1, 2], "b": "x"} | tojson }}"#).unwrap();
    assert_eq!(out, r#"{"a":[1,2],"b":"x"}"#);
}

#[test]
fn tojson_serializes_recipe_data() {
    let out = render_template(
        "@flour{200%g} and @eggs{2}",
        "{{ ingredients | map(attribute='name') | list | tojson }}",
    )
    .unwrap();
    assert_eq!(out, r#"["flour","eggs"]"#);
}

#[test]
fn tojson_escapes_html_significant_characters() {
    let out = render_template("@flour{1}", r#"{{ "</script> & 'x'" | tojson }}"#).unwrap();
    assert!(
        !out.contains('<') && !out.contains('>') && !out.contains('&') && !out.contains('\''),
        "{out}"
    );
    let back: String = serde_json::from_str(&out).unwrap();
    assert_eq!(back, "</script> & 'x'");
}
