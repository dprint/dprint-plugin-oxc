extern crate dprint_development;
extern crate dprint_plugin_oxc;

use std::path::PathBuf;
use std::sync::Arc;

use dprint_core::configuration::*;
use dprint_development::*;
use dprint_plugin_oxc::configuration::Configuration;
use dprint_plugin_oxc::configuration::resolve_config;
use dprint_plugin_oxc::*;

#[test]
fn test_specs() {
  let global_config = GlobalConfiguration::default();

  run_specs(
    &PathBuf::from("./tests/specs"),
    &ParseSpecOptions {
      default_file_name: "file.ts",
    },
    &RunSpecsOptions {
      fix_failures: false,
      format_twice: true,
    },
    {
      let global_config = global_config.clone();
      Arc::new(move |file_path, file_text, _range, spec_config| {
        let spec_config: ConfigKeyMap = serde_json::from_value(spec_config.clone().into()).unwrap();
        let config_result = resolve_config(spec_config, &global_config);
        ensure_no_diagnostics(&config_result.diagnostics);

        format_text(file_path, file_text, &config_result.config)
      })
    },
    Arc::new(move |_file_path, _file_text, _spec_config| panic!("Plugin does not support dprint-core tracing.")),
  )
}

#[test]
fn should_fail_on_parse_error_js() {
  let config = Configuration::default();
  let err = format_text(&PathBuf::from("./file.ts"), "const t string = 5;", &config).unwrap_err();
  // Just verify that it returns an error for invalid syntax
  assert!(!err.to_string().is_empty());
}

#[test]
fn should_fail_on_parse_error_other_languages() {
  let config = Configuration::default();
  for (file_name, text) in [
    ("file.json", "{ \"a\": }"),
    ("package.json", "{ \"a\": }"),
    ("file.graphql", "query {"),
    ("file.css", "a { color: red"),
    ("file.scss", "a { b: { }"),
    ("file.yaml", "a: [1, 2"),
    (".prettierrc", "a: [1, 2"),
  ] {
    let result = format_text(&PathBuf::from(file_name), text, &config);
    assert!(result.is_err(), "expected an error for {file_name}");
  }
}

#[test]
fn should_not_format_unknown_or_excluded_files() {
  let config = Configuration::default();
  for file_name in [
    "file.txt",
    "file.html",
    "package-lock.json",
    "pnpm-lock.yaml",
    "Cargo.lock",
  ] {
    let result = format_text(&PathBuf::from(file_name), "a:   1", &config).unwrap();
    assert_eq!(result, None, "{file_name}");
  }
}

#[test]
fn should_only_format_markdown_when_enabled() {
  let path = PathBuf::from("file.md");
  let text = "#   Title\n";
  for config in [
    Configuration::default(),
    resolve(serde_json::json!({ "experimentalMarkdown": false })).config,
  ] {
    assert_eq!(format_text(&path, text, &config).unwrap(), None);
    // not even a whitespace only file is changed
    assert_eq!(format_text(&path, "  \n", &config).unwrap(), None);
  }

  let config = resolve(serde_json::json!({ "experimentalMarkdown": true })).config;
  assert_eq!(format_text(&path, text, &config).unwrap().as_deref(), Some("# Title\n"));
}

#[test]
fn should_format_whitespace_only_file_as_empty() {
  let config = resolve(serde_json::json!({ "experimentalMarkdown": true })).config;
  for file_name in [
    "file.ts",
    "file.json",
    "file.css",
    "file.yaml",
    "file.md",
    "file.toml",
    "file.graphql",
  ] {
    let path = PathBuf::from(file_name);
    assert_eq!(format_text(&path, "", &config).unwrap(), None, "{file_name}");
    assert_eq!(
      format_text(&path, "  \n\n", &config).unwrap(),
      Some(String::new()),
      "{file_name}"
    );
  }
}

#[test]
fn should_use_line_ending() {
  let config = resolve(serde_json::json!({ "lineEnding": "crlf", "experimentalMarkdown": true })).config;
  for (file_name, text, expected) in [
    ("file.ts", "a;\nb;\n", "a;\r\nb;\r\n"),
    ("file.json", "[\n1,\n2]", "[1, 2]\r\n"),
    ("file.css", "a{b:c}", "a {\r\n  b: c;\r\n}\r\n"),
    ("file.yaml", "a: 1\nb: 2\n", "a: 1\r\nb: 2\r\n"),
    ("file.md", "a\n\nb\n", "a\r\n\r\nb\r\n"),
    ("file.toml", "a=1\nb=2\n", "a = 1\r\nb = 2\r\n"),
    ("file.graphql", "{a}", "{\r\n  a\r\n}\r\n"),
  ] {
    let result = format_text(&PathBuf::from(file_name), text, &config).unwrap();
    assert_eq!(result.as_deref(), Some(expected), "{file_name}");
  }
}

#[test]
fn should_resolve_toggle_options() {
  let result = resolve(serde_json::json!({
    "sortImports": true,
    "sortTailwindcss": true,
    "jsdoc": true,
    "sortPackageJson": true,
  }));
  assert!(result.diagnostics.is_empty());
  assert!(result.config.experimental_sort_imports.is_some());
  assert!(result.config.experimental_tailwindcss.is_some());
  assert!(result.config.jsdoc.is_some());
  assert!(result.config.sort_package_json.is_some_and(|options| options.enabled));

  let result = resolve(serde_json::json!({
    "experimentalSortImports": false,
    "experimentalTailwindcss": false,
    "jsdoc": false,
    "sortPackageJson": false,
  }));
  assert!(result.diagnostics.is_empty());
  assert!(result.config.experimental_sort_imports.is_none());
  assert!(result.config.experimental_tailwindcss.is_none());
  assert!(result.config.jsdoc.is_none());
  assert!(result.config.sort_package_json.is_some_and(|options| !options.enabled));
}

#[test]
fn should_have_diagnostics_for_invalid_config() {
  for (config, property_name) in [
    (serde_json::json!({ "sortImports": "yes" }), "sortImports"),
    (serde_json::json!({ "sortPackageJson": "yes" }), "sortPackageJson"),
    (
      serde_json::json!({ "sortPackageJson": { "unknown": true } }),
      "sortPackageJson.unknown",
    ),
    (serde_json::json!({ "proseWrap": "sometimes" }), "proseWrap"),
    (
      serde_json::json!({ "embeddedLanguageFormatting": "on" }),
      "embeddedLanguageFormatting",
    ),
    // these are the combinations that oxc itself rejects
    (
      serde_json::json!({ "sortImports": { "partitionByNewline": true } }),
      "sortImports",
    ),
    (
      serde_json::json!({ "sortImports": { "groups": ["builtin", "not-a-group"] } }),
      "sortImports",
    ),
    (
      serde_json::json!({ "sortImports": { "groups": [{ "newlinesBetween": true }, "builtin"] } }),
      "sortImports.groups",
    ),
    (
      serde_json::json!({ "sortImports": { "groups": ["builtin", { "newlinesBetween": true }] } }),
      "sortImports.groups",
    ),
  ] {
    let diagnostics = resolve(config.clone()).diagnostics;
    assert_eq!(
      diagnostics.iter().map(|d| d.property_name.as_str()).collect::<Vec<_>>(),
      vec![property_name],
      "{config}"
    );
  }
}

fn resolve(config: serde_json::Value) -> ResolveConfigurationResult<Configuration> {
  let config: ConfigKeyMap = serde_json::from_value(config).unwrap();
  resolve_config(config, &GlobalConfiguration::default())
}
