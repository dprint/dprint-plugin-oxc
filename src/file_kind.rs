use std::path::Path;

use oxc_formatter_css::CssVariant;
use oxc_formatter_json::JsonVariant;
use oxc_span::SourceType;

use crate::configuration::Configuration;

/// The kind of file to format, which decides the formatter that handles it.
///
/// This mirrors how oxfmt classifies files so that a file is formatted the
/// same way by this plugin as it is by oxfmt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
  Js(SourceType),
  Json(JsonVariant),
  /// `package.json` is optionally sorted, then formatted `JSON.stringify` style.
  PackageJson,
  Graphql,
  Css(CssVariant),
  Yaml,
  /// Files like `.prettierrc` that are formatted as JSON, falling back to YAML.
  YamlRc,
  Markdown,
  Toml,
}

impl FileKind {
  /// Gets the kind of the file at the provided path or `None` when the
  /// file is not one that should be formatted.
  pub fn from_path(path: &Path) -> Option<FileKind> {
    if let Ok(source_type) = SourceType::from_path(path) {
      return Some(FileKind::Js(source_type));
    }

    let file_name = path.file_name().and_then(|f| f.to_str())?;

    // machine generated files that should never be reformatted
    if EXCLUDE_FILE_NAMES.contains(&file_name) {
      return None;
    }

    let extension = path.extension().and_then(|ext| ext.to_str());
    let has_extension = |extensions: &[&str]| extension.is_some_and(|ext| extensions.contains(&ext));

    if is_extra_js_file(file_name, extension) {
      return Some(FileKind::Js(SourceType::default()));
    }
    if TOML_FILE_NAMES.contains(&file_name) || extension == Some("toml") || file_name.ends_with(".toml.example") {
      return Some(FileKind::Toml);
    }
    if file_name == "package.json" {
      return Some(FileKind::PackageJson);
    }
    if file_name == "composer.json" || extension == Some("importmap") {
      return Some(FileKind::Json(JsonVariant::JsonStringify));
    }
    if JSON_FILE_NAMES.contains(&file_name)
      || has_extension(JSON_EXTENSIONS)
      || file_name.ends_with(".json.example")
      || file_name.ends_with(".tfstate.backup")
    {
      return Some(FileKind::Json(JsonVariant::Json));
    }
    if has_extension(JSONC_EXTENSIONS) {
      return Some(FileKind::Json(JsonVariant::Jsonc));
    }
    if extension == Some("json5") {
      return Some(FileKind::Json(JsonVariant::Json5));
    }
    if has_extension(GRAPHQL_EXTENSIONS) {
      return Some(FileKind::Graphql);
    }
    if has_extension(CSS_EXTENSIONS) {
      return Some(FileKind::Css(CssVariant::Css));
    }
    match extension {
      Some("scss") => return Some(FileKind::Css(CssVariant::Scss)),
      Some("less") => return Some(FileKind::Css(CssVariant::Less)),
      _ => {}
    }
    // check these before the generic YAML check because they're formatted as JSON first
    if YAML_RC_FILE_NAMES.contains(&file_name) {
      return Some(FileKind::YamlRc);
    }
    if YAML_FILE_NAMES.contains(&file_name) || has_extension(YAML_EXTENSIONS) {
      return Some(FileKind::Yaml);
    }
    if MARKDOWN_FILE_NAMES.contains(&file_name) || has_extension(MARKDOWN_EXTENSIONS) {
      return Some(FileKind::Markdown);
    }
    None
  }
}

/// The file extensions the plugin formats with the provided configuration.
pub fn file_extensions(config: &Configuration) -> Vec<String> {
  [
    JS_EXTENSIONS,
    ADDITIONAL_JS_EXTENSIONS,
    &["toml", "importmap", "json5", "scss", "less"],
    JSON_EXTENSIONS,
    JSONC_EXTENSIONS,
    GRAPHQL_EXTENSIONS,
    CSS_EXTENSIONS,
    YAML_EXTENSIONS,
    markdown_only(config, MARKDOWN_EXTENSIONS),
  ]
  .into_iter()
  .flatten()
  .map(|ext| ext.to_string())
  .collect()
}

/// The names of the files the plugin formats regardless of their extension
/// with the provided configuration.
pub fn file_names(config: &Configuration) -> Vec<String> {
  [
    JS_FILE_NAMES,
    TOML_FILE_NAMES,
    JSON_FILE_NAMES,
    YAML_RC_FILE_NAMES,
    YAML_FILE_NAMES,
    markdown_only(config, MARKDOWN_FILE_NAMES),
  ]
  .into_iter()
  .flatten()
  .map(|name| name.to_string())
  .collect()
}

fn markdown_only(config: &Configuration, items: &'static [&'static str]) -> &'static [&'static str] {
  if config.experimental_markdown == Some(true) {
    items
  } else {
    &[]
  }
}

fn is_extra_js_file(file_name: &str, extension: Option<&str>) -> bool {
  if JS_FILE_NAMES.contains(&file_name) {
    return true;
  }
  let Some(extension) = extension else {
    return false;
  };
  if ADDITIONAL_JS_EXTENSIONS.contains(&extension) {
    return true;
  }
  // only `*.start.frag` and `*.end.frag` are JS
  if extension == "frag" {
    let stem = &file_name[..file_name.len() - ".frag".len()];
    return stem.ends_with(".start") || stem.ends_with(".end");
  }
  false
}

// The lists below are kept in sync with oxfmt's (see `apps/oxfmt/src/core/support.rs`
// in the oxc repo), which are in turn derived from what Prettier supports.

const EXCLUDE_FILE_NAMES: &[&str] = &[
  // JSON, YAML lock files
  "package-lock.json",
  "pnpm-lock.yaml",
  "yarn.lock",
  "MODULE.bazel.lock",
  "bun.lock",
  "deno.lock",
  "composer.lock",
  "Package.resolved",
  "Pipfile.lock",
  "flake.lock",
  "mcmod.info",
  // TOML lock files
  "Cargo.lock",
  "Gopkg.lock",
  "pdm.lock",
  "poetry.lock",
  "uv.lock",
];

/// The extensions understood by `SourceType::from_path`.
const JS_EXTENSIONS: &[&str] = &["ts", "tsx", "cts", "mts", "js", "jsx", "cjs", "mjs"];

const ADDITIONAL_JS_EXTENSIONS: &[&str] = &[
  "_js",
  "bones",
  "es",
  "es6",
  "gs",
  "jake",
  "javascript",
  "jsb",
  "jscad",
  "jsfl",
  "jslib",
  "jsm",
  "jspre",
  "jss",
  "njs",
  "pac",
  "sjs",
  "ssjs",
  "xsjs",
  "xsjslib",
];

const JS_FILE_NAMES: &[&str] = &["Jakefile", "start.frag", "end.frag"];

const TOML_FILE_NAMES: &[&str] = &["Pipfile", "Cargo.toml.orig"];

const JSON_EXTENSIONS: &[&str] = &[
  "json",
  "4DForm",
  "4DProject",
  "avsc",
  "geojson",
  "gltf",
  "har",
  "ice",
  "JSON-tmLanguage",
  "mcmeta",
  "sarif",
  "tact",
  "tfstate",
  "topojson",
  "webapp",
  "webmanifest",
  "yy",
  "yyp",
];

const JSON_FILE_NAMES: &[&str] = &[
  ".all-contributorsrc",
  ".arcconfig",
  ".auto-changelog",
  ".c8rc",
  ".htmlhintrc",
  ".imgbotconfig",
  ".nycrc",
  ".tern-config",
  ".tern-project",
  ".watchmanconfig",
  ".babelrc",
  ".jscsrc",
  ".jshintrc",
  ".jslintrc",
  ".swcrc",
];

const JSONC_EXTENSIONS: &[&str] = &[
  "jsonc",
  "code-snippets",
  "code-workspace",
  "sublime-build",
  "sublime-color-scheme",
  "sublime-commands",
  "sublime-completions",
  "sublime-keymap",
  "sublime-macro",
  "sublime-menu",
  "sublime-mousemap",
  "sublime-project",
  "sublime-settings",
  "sublime-theme",
  "sublime-workspace",
  "sublime_metrics",
  "sublime_session",
];

const GRAPHQL_EXTENSIONS: &[&str] = &["graphql", "gql", "graphqls"];

const CSS_EXTENSIONS: &[&str] = &["css", "wxss", "pcss", "postcss"];

const YAML_RC_FILE_NAMES: &[&str] = &[".prettierrc", ".stylelintrc", ".lintstagedrc"];

const YAML_FILE_NAMES: &[&str] = &[
  ".clang-format",
  ".clang-tidy",
  ".clangd",
  ".gemrc",
  "CITATION.cff",
  "glide.lock",
  "pixi.lock",
];

const YAML_EXTENSIONS: &[&str] = &[
  "yml",
  "mir",
  "reek",
  "rviz",
  "sublime-syntax",
  "syntax",
  "yaml",
  "yaml-tmlanguage",
];

// oxfmt does not format Markdown with `oxc_formatter_markdown` yet (it uses Prettier),
// so these are what Prettier considers to be Markdown.
const MARKDOWN_EXTENSIONS: &[&str] = &[
  "md", "livemd", "markdown", "mdown", "mdwn", "mkd", "mkdn", "mkdown", "ronn", "scd", "workbook",
];

const MARKDOWN_FILE_NAMES: &[&str] = &["contents.lr", "README"];

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn classifies_files() {
    let kind = |path: &str| FileKind::from_path(Path::new(path));
    assert!(matches!(kind("dir/file.ts"), Some(FileKind::Js(_))));
    assert!(matches!(kind("Jakefile"), Some(FileKind::Js(_))));
    assert!(matches!(kind("shell.start.frag"), Some(FileKind::Js(_))));
    assert_eq!(kind("shader.frag"), None);
    assert_eq!(kind("data.json"), Some(FileKind::Json(JsonVariant::Json)));
    assert_eq!(kind(".babelrc"), Some(FileKind::Json(JsonVariant::Json)));
    assert_eq!(kind("data.json.example"), Some(FileKind::Json(JsonVariant::Json)));
    assert_eq!(kind("tsconfig.jsonc"), Some(FileKind::Json(JsonVariant::Jsonc)));
    assert_eq!(kind("data.json5"), Some(FileKind::Json(JsonVariant::Json5)));
    assert_eq!(kind("composer.json"), Some(FileKind::Json(JsonVariant::JsonStringify)));
    assert_eq!(kind("sub/package.json"), Some(FileKind::PackageJson));
    assert_eq!(kind("query.gql"), Some(FileKind::Graphql));
    assert_eq!(kind("style.css"), Some(FileKind::Css(CssVariant::Css)));
    assert_eq!(kind("style.scss"), Some(FileKind::Css(CssVariant::Scss)));
    assert_eq!(kind("style.less"), Some(FileKind::Css(CssVariant::Less)));
    assert_eq!(kind("config.yml"), Some(FileKind::Yaml));
    assert_eq!(kind(".clang-format"), Some(FileKind::Yaml));
    assert_eq!(kind(".prettierrc"), Some(FileKind::YamlRc));
    assert_eq!(kind("README.md"), Some(FileKind::Markdown));
    assert_eq!(kind("Cargo.toml"), Some(FileKind::Toml));
    assert_eq!(kind("Pipfile"), Some(FileKind::Toml));
    assert_eq!(kind("file.txt"), None);
  }

  #[test]
  fn excludes_lock_files() {
    for name in EXCLUDE_FILE_NAMES {
      assert_eq!(FileKind::from_path(Path::new(name)), None, "{name}");
    }
  }

  #[test]
  fn matched_files_are_classified() {
    let config = Configuration {
      experimental_markdown: Some(true),
      ..Default::default()
    };
    for ext in file_extensions(&config) {
      let path = format!("file.{ext}");
      assert!(FileKind::from_path(Path::new(&path)).is_some(), "{path}");
    }
    for name in file_names(&config) {
      assert!(FileKind::from_path(Path::new(&name)).is_some(), "{name}");
    }
  }

  #[test]
  fn markdown_is_only_matched_when_enabled() {
    let config = Configuration::default();
    assert!(!file_extensions(&config).contains(&"md".to_string()));
    assert!(!file_names(&config).contains(&"README".to_string()));
    assert!(file_extensions(&config).contains(&"json".to_string()));

    let config = Configuration {
      experimental_markdown: Some(true),
      ..Default::default()
    };
    assert!(file_extensions(&config).contains(&"md".to_string()));
    assert!(file_names(&config).contains(&"README".to_string()));
  }
}
