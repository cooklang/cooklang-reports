use cooklang_reports::{render_template_with_config, Config, ConfigExtension};
use minijinja::Environment;
use std::sync::{Arc, Mutex};

struct CountingExtension {
    counter: Arc<Mutex<u32>>,
}

impl ConfigExtension for CountingExtension {
    fn register(&self, env: &mut Environment<'_>) {
        let c = self.counter.clone();
        env.add_function("bump", move || -> Result<u32, minijinja::Error> {
            let mut n = c.lock().unwrap();
            *n += 1;
            Ok(*n)
        });
    }
}

#[test]
fn extension_function_is_registered() {
    let counter = Arc::new(Mutex::new(0u32));
    let config = Config::builder()
        .build()
        .with_extension(CountingExtension { counter: counter.clone() });

    let recipe = "@flour{1}";
    let template = "{{ bump() }}-{{ bump() }}";

    let out = render_template_with_config(recipe, template, &config).unwrap();
    assert_eq!(out, "1-2");
    assert_eq!(*counter.lock().unwrap(), 2);
}
