use std::borrow::Cow;
use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_core::FormatSession;
use oxc_formatter_core::InputKind;
use oxc_formatter_css::CssVariant;
use oxc_formatter_json::JsonVariant;
use oxc_span::SourceType;

use crate::configuration::Configuration;
use crate::embed::build_session_services;
use crate::file_kind::FileKind;
use crate::options::build_css_options;
use crate::options::build_graphql_options;
use crate::options::build_js_options;
use crate::options::build_json_options;
use crate::options::build_markdown_options;
use crate::options::build_toml_options;
use crate::options::build_yaml_options;

type FormatError = Box<dyn std::error::Error + Send + Sync>;

pub fn format_text(file_path: &Path, input_text: &str, config: &Configuration) -> Result<Option<String>, FormatError> {
  let Some(file_kind) = FileKind::from_path(file_path) else {
    return Ok(None);
  };

  // a final newline is not inserted into an empty file
  if input_text.trim().is_empty() {
    return Ok(if input_text.is_empty() {
      None
    } else {
      Some(String::new())
    });
  }

  let mut output = match file_kind {
    FileKind::Js(source_type) => format_js(input_text, source_type, config)?,
    FileKind::Json(variant) => format_json(input_text, variant, config)?,
    FileKind::PackageJson => format_package_json(input_text, config)?,
    FileKind::Graphql => format_graphql(input_text, config)?,
    FileKind::Css(variant) => format_css(input_text, variant, config)?,
    FileKind::Yaml => format_yaml(input_text, config)?,
    // these are formatted as JSON when they are JSON
    FileKind::YamlRc => match format_json(input_text, JsonVariant::Json, config) {
      Ok(output) => output,
      Err(_) => format_yaml(input_text, config)?,
    },
    FileKind::Markdown => format_markdown(input_text, config)?,
    FileKind::Toml => oxc_toml::format(input_text, build_toml_options(config)),
  };

  // every formatter ends its output with a newline
  if config.insert_final_newline == Some(false) {
    output.truncate(output.trim_end().len());
  }

  if output == input_text {
    Ok(None)
  } else {
    Ok(Some(output))
  }
}

fn format_js(input_text: &str, source_type: SourceType, config: &Configuration) -> Result<String, FormatError> {
  let allocator = Allocator::default();
  let session = FormatSession::with_services(&allocator, InputKind::PhysicalFile, build_session_services(config));
  let formatted = oxc_formatter::format_with_session(&session, input_text, source_type, build_js_options(config))
    .map_err(|e| e.to_string())?;
  Ok(formatted.print().map_err(|e| e.to_string())?.into_code())
}

fn format_json(input_text: &str, variant: JsonVariant, config: &Configuration) -> Result<String, FormatError> {
  let allocator = Allocator::default();
  let formatted = oxc_formatter_json::format(&allocator, input_text, build_json_options(config, variant))
    .map_err(|e| e.to_string())?;
  Ok(formatted.print().map_err(|e| e.to_string())?.into_code())
}

fn format_package_json(input_text: &str, config: &Configuration) -> Result<String, FormatError> {
  let sort_options = match &config.sort_package_json {
    Some(options) if !options.enabled => None,
    options => Some(
      sort_package_json::SortOptions::new()
        .with_sort_scripts(options.as_ref().is_some_and(|options| options.sort_scripts))
        // the text is formatted after
        .with_pretty(false),
    ),
  };
  // the sorter only handles strictly valid JSON, but the formatter is more
  // permissive, so format without sorting instead of failing when it errors
  let text = match sort_options.map(|options| sort_package_json::sort_package_json_with_options(input_text, &options)) {
    Some(Ok(sorted_text)) => Cow::Owned(sorted_text),
    Some(Err(_)) | None => Cow::Borrowed(input_text),
  };
  format_json(&text, JsonVariant::JsonStringify, config)
}

fn format_graphql(input_text: &str, config: &Configuration) -> Result<String, FormatError> {
  let allocator = Allocator::default();
  let formatted =
    oxc_formatter_graphql::format(&allocator, input_text, build_graphql_options(config)).map_err(|e| e.to_string())?;
  Ok(formatted.print().map_err(|e| e.to_string())?.into_code())
}

fn format_css(input_text: &str, variant: CssVariant, config: &Configuration) -> Result<String, FormatError> {
  let allocator = Allocator::default();
  // the services are what format the front matter
  let session = FormatSession::with_services(&allocator, InputKind::PhysicalFile, build_session_services(config));
  let formatted = oxc_formatter_css::format_with_session(&session, input_text, build_css_options(config, variant))
    .map_err(|e| e.to_string())?;
  Ok(formatted.print().map_err(|e| e.to_string())?.into_code())
}

fn format_yaml(input_text: &str, config: &Configuration) -> Result<String, FormatError> {
  let allocator = Allocator::default();
  let formatted =
    oxc_formatter_yaml::format(&allocator, input_text, build_yaml_options(config)).map_err(|e| e.to_string())?;
  Ok(formatted.print().map_err(|e| e.to_string())?.into_code())
}

fn format_markdown(input_text: &str, config: &Configuration) -> Result<String, FormatError> {
  let allocator = Allocator::default();
  let formatted = oxc_formatter_markdown::format(&allocator, input_text, build_markdown_options(config))
    .map_err(|e| e.to_string())?;
  Ok(formatted.print().map_err(|e| e.to_string())?.into_code())
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn formats_basic_js() {
    let input = "const x=1";
    let config = crate::configuration::Configuration::default();
    let result = format_text(std::path::Path::new("test.js"), input, &config)
      .unwrap()
      .unwrap();
    assert_eq!(result, "const x = 1;\n");
  }
}
