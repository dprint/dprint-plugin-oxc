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
    ("file.jsonc", "{ \"a\": }"),
    ("file.json5", "{ a: }"),
    ("file.css", "a { color: red"),
    ("file.scss", "a { b: { }"),
    ("file.less", "a { b: { }"),
    ("file.toml", "a = = 1"),
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
fn should_format_markdown() {
  let path = PathBuf::from("file.md");
  let text = "#   Title\n";
  let config = Configuration::default();
  assert_eq!(format_text(&path, text, &config).unwrap().as_deref(), Some("# Title\n"));
  assert_eq!(format_text(&path, "  \n", &config).unwrap(), Some(String::new()));
}

#[test]
fn should_format_whitespace_only_file_as_empty() {
  let config = Configuration::default();
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
  let config = resolve(serde_json::json!({ "lineEnding": "crlf" })).config;
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
    (
      serde_json::json!({ "sortPackageJson": true, "experimentalSortPackageJson": false }),
      "experimentalSortPackageJson",
    ),
    (
      serde_json::json!({ "sortImports": true, "experimentalSortImports": { "order": "desc" } }),
      "experimentalSortImports",
    ),
    (
      serde_json::json!({ "sortTailwindcss": true, "experimentalTailwindcss": {} }),
      "experimentalTailwindcss",
    ),
    (
      serde_json::json!({
        "sortImports": {
          "groups": ["builtin", { "newlinesBetween": false }, "external"],
          "newlineBoundaryOverrides": [true],
        },
      }),
      "sortImports.newlineBoundaryOverrides",
    ),
    // these are the combinations that oxc itself rejects
    (
      serde_json::json!({ "sortImports": { "partitionByNewline": true, "newlinesBetween": true } }),
      "sortImports",
    ),
    (
      serde_json::json!({ "experimentalSortImports": { "newlineBoundaryOverrides": [false] } }),
      "experimentalSortImports",
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

#[test]
fn should_resolve_aliases() {
  // a null value is the same as the option not being set
  let result = resolve(serde_json::json!({
    "sortImports": null,
    "experimentalSortImports": { "order": "desc" },
    "sortPackageJson": null,
    "experimentalSortPackageJson": false,
  }));
  assert!(result.diagnostics.is_empty());
  assert!(result.config.experimental_sort_imports.is_some());
  assert!(result.config.sort_package_json.is_some_and(|options| !options.enabled));
}

#[test]
fn should_not_have_newlines_between_by_default_when_partitioning_by_newline() {
  let result = resolve(serde_json::json!({ "experimentalSortImports": { "partitionByNewline": true } }));
  assert!(result.diagnostics.is_empty());
  assert_eq!(
    result.config.experimental_sort_imports.unwrap().newlines_between,
    Some(false)
  );
}

#[test]
fn should_have_position_in_parse_errors() {
  let config = Configuration::default();
  for (file_name, text, expected) in [
    ("file.ts", "const a = 1;\nconst = ;\n", "(line 2, column 7)"),
    ("file.json", "{\n  \"a\": 1,\n  \"b\": }\n", "(line 3, column 8)"),
    ("file.yaml", "a: 1\nb: [1, 2\n", "(line 3, column 1)"),
    ("file.toml", "a = 1\nb = = 2\n", "(line 2, column 5)"),
  ] {
    let err = format_text(&PathBuf::from(file_name), text, &config).unwrap_err();
    assert!(err.to_string().ends_with(expected), "{file_name}: {err}");
  }
}

#[test]
fn should_format_files_matched_case_insensitively() {
  let config = Configuration::default();
  for (file_name, text, expected) in [
    ("FILE.TS", "a", "a;\n"),
    ("File.JSON", "[1,2]", "[1, 2]\n"),
    ("STYLE.CSS", "a{b:c}", "a {\n  b: c;\n}\n"),
    ("CONFIG.YML", "a:   1", "a: 1\n"),
    ("pipfile", "a=1", "a = 1\n"),
  ] {
    let result = format_text(&PathBuf::from(file_name), text, &config).unwrap();
    assert_eq!(result.as_deref(), Some(expected), "{file_name}");
  }
}

#[test]
fn should_format_package_json_without_sorting_when_the_sorter_fails() {
  // the sorter only accepts strictly valid json, but the formatter accepts more
  let config = Configuration::default();
  let result = format_text(
    &PathBuf::from("package.json"),
    "{version:\"1\",\"name\":\"a\",}",
    &config,
  )
  .unwrap();
  assert_eq!(
    result.as_deref(),
    Some("{\n  \"version\": \"1\",\n  \"name\": \"a\"\n}\n")
  );
}

fn resolve(config: serde_json::Value) -> ResolveConfigurationResult<Configuration> {
  let config: ConfigKeyMap = serde_json::from_value(config).unwrap();
  resolve_config(config, &GlobalConfiguration::default())
}
