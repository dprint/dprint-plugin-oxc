extern crate dprint_development;
extern crate dprint_plugin_oxc;

use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use dprint_core::configuration::*;
use dprint_development::*;
use dprint_plugin_oxc::configuration::Configuration;
use dprint_plugin_oxc::configuration::resolve_config;
use dprint_plugin_oxc::*;
use oxc_formatter::ExternalCallbacks;

fn make_external_callbacks() -> ExternalCallbacks {
  let css_config = malva::config::FormatOptions::default();
  ExternalCallbacks::new().with_embedded_formatter(Some(Arc::new(move |language, code| {
    match language {
      "tagged-css" | "styled-jsx" | "angular-styles" => {
        malva::format_text(code, malva::Syntax::Scss, &css_config).map_err(|e| format!("{e}"))
      }
      "tagged-html" | "angular-template" => {
        let html_opts = markup_fmt::config::FormatOptions::default();
        markup_fmt::format_text(code, markup_fmt::Language::Html, &html_opts, |code, _| {
          Ok::<_, std::convert::Infallible>(Cow::Borrowed(code))
        })
        .map_err(|e| format!("{e}"))
      }
      "tagged-markdown" => {
        let md_config =
          dprint_plugin_markdown::configuration::ConfigurationBuilder::new().build();
        dprint_plugin_markdown::format_text(code, &md_config, |_, _, _| Ok(None))
          .map(|r| r.unwrap_or_else(|| code.to_string()))
          .map_err(|e| format!("{e}"))
      }
      _ => Ok(code.to_string()),
    }
  })))
}

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
      Arc::new(move |file_path, file_text, spec_config| {
        let spec_config: ConfigKeyMap = serde_json::from_value(spec_config.clone().into()).unwrap();
        let config_result = resolve_config(spec_config, &global_config);
        ensure_no_diagnostics(&config_result.diagnostics);

        format_text(
          file_path,
          &file_text,
          &config_result.config,
          Some(make_external_callbacks()),
        )
      })
    },
    Arc::new(move |_file_path, _file_text, _spec_config| panic!("Plugin does not support dprint-core tracing.")),
  )
}

#[test]
fn should_fail_on_parse_error_js() {
  let config = Configuration::default();
  let err = format_text(&PathBuf::from("./file.ts"), "const t string = 5;", &config, None).unwrap_err();
  // Just verify that it returns an error for invalid syntax
  assert!(!err.to_string().is_empty());
}

#[test]
fn format_with_json_plugin() {
  // Verify dprint-plugin-json can format JSON (oxc doesn't handle .json files)
  let input = r#"{"hello":   "world","num":42}"#;
  let json_config = dprint_plugin_json::configuration::ConfigurationBuilder::new().build();
  let result = dprint_plugin_json::format_text(Path::new("test.json"), input, &json_config)
    .unwrap()
    .unwrap();
  assert!(result.contains("\"hello\": \"world\""));
  let _: serde_json::Value =
    serde_json::from_str(&result).expect("dprint-plugin-json output should be valid JSON");
}
