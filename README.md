# Cooklang Reports

A PoC of a Rust library for generating reports from Cooklang recipes using Jinja2-style templates.

## Features

- Parse Cooklang recipes
- Scale recipe quantities
- Access recipe metadata and ingredients
- Template-based report generation
- Custom filters for formatting
- YAML datastore integration for recipe metadata
- Support for nested data access

## Installation (soon)

Add this to your `Cargo.toml`:

```toml
[dependencies]
cooklang-reports = "0.1.0"
```

## Usage

### Basic Template Rendering

```rust
use indoc::indoc;
use cooklang_reports::render_template;

let recipe = r#"
    Mix @eggs{3%large} with @milk{250%ml}, add @flour{125%g} to make batter.
    Add @sugar{1.5%tbsp} and @salt{1/4%tsp} for flavor.
"#;

let template = indoc! {"
    # Ingredients ({{ scale }}x)
    {%- for ingredient in ingredients %}
    - {{ ingredient.name }}
    {%- endfor %}
"};

// Test default scaling (1x)
let result = render_template(&recipe, template).unwrap();
let expected = indoc! {"
    # Ingredients (1.0x)
    - eggs
    - milk
    - flour
    - sugar
    - salt"};
assert_eq!(result, expected);
```

### Using Datastore

```rust
use indoc::indoc;
use std::path::Path;
use cooklang_reports::render_template_with_config;
use cooklang_reports::config::Config;

let recipe = r#"
    Mix @eggs{3%large} with @milk{250%ml}, add @flour{125%g} to make batter.
    Add @sugar{1.5%tbsp} and @salt{1/4%tsp} for flavor.
"#;

let datastore_path = Path::new("test/data/db");
let template = indoc! {"
    # Eggs Info

    Density: {{ db('eggs.meta.density') }}
    Shelf Life: {{ db('eggs.meta.storage.shelf life') }} days
    Fridge Life: {{ db('eggs.meta.storage.fridge life') }} days
"};

let config = Config::builder().datastore_path(datastore_path).build();
let result = render_template_with_config(&recipe, template, &config).unwrap();
let expected = indoc! {"
    # Eggs Info

    Density: 1.03
    Shelf Life: 30 days
    Fridge Life: 60 days"};

assert_eq!(result, expected);
```

## Template Features

### Available Variables

- `ingredients`: List of recipe ingredients with their quantities and units
- `scale`: Current recipe scale factor
- `recipe_template`: Full recipe template object with additional methods

### Built-in Functions

- `db(key_path)`: Access data from the YAML datastore
  - Format: `directory.file.key.subkey`
  - Example: `db('eggs.meta.storage.shelf life')`

### Built-in Filters

- `quantity`: Format ingredient quantities with proper spacing
  - Example: `{{ ingredient.quantity | quantity }}`

## Project Structure

```text
src/
├── lib.rs           # Main library code
├── filters/         # Template filters
│   ├── mod.rs
│   └── quantity.rs
└── functions/       # Template functions
    ├── mod.rs
    └── datastore.rs
```

## Datastore Format

The datastore is a directory of YAML files organized by ingredient:

```text
datastore/
├── eggs/
│   ├── meta.yml
│   └── shopping.yml
├── milk/
│   ├── meta.yml
│   └── shopping.yml
└── flour/
    ├── meta.yml
    └── shopping.yml
```

Example YAML file (`eggs/meta.yml`):
```yaml
density: 1.03
storage:
  shelf life: 30
  fridge life: 60
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MPL 2.0 License - see the LICENSE file for details.
