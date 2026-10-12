use std::path::Path;

use oxc_formatter_css::CssVariant;
use oxc_formatter_json::JsonVariant;
use oxc_span::SourceType;

use crate::configuration::Configuration;
use crate::configuration::Language;

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
    // dprint matches file names and extensions case insensitively, so the
    // file is classified that way too in order to format every matched file
    let file_name = path.file_name().and_then(|f| f.to_str())?.to_ascii_lowercase();
    let file_name = file_name.as_str();
    let path = Path::new(file_name);

    if let Ok(source_type) = SourceType::from_path(path) {
      return Some(FileKind::Js(source_type));
    }

    // machine generated files that should never be reformatted
    if contains(EXCLUDE_FILE_NAMES, file_name) {
      return None;
    }

    let extension = path.extension().and_then(|ext| ext.to_str());
    let has_extension = |extensions: &[&str]| extension.is_some_and(|ext| contains(extensions, ext));

    if is_extra_js_file(file_name, extension) {
      return Some(FileKind::Js(SourceType::default()));
    }
    if contains(TOML_FILE_NAMES, file_name) || extension == Some("toml") || file_name.ends_with(".toml.example") {
      return Some(FileKind::Toml);
    }
    if file_name == "package.json" {
      return Some(FileKind::PackageJson);
    }
    if file_name == "composer.json" || extension == Some("importmap") {
      return Some(FileKind::Json(JsonVariant::JsonStringify));
    }
    if contains(JSON_FILE_NAMES, file_name)
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
    if contains(YAML_RC_FILE_NAMES, file_name) {
      return Some(FileKind::YamlRc);
    }
    if contains(YAML_FILE_NAMES, file_name) || has_extension(YAML_EXTENSIONS) {
      return Some(FileKind::Yaml);
    }
    if contains(MARKDOWN_FILE_NAMES, file_name) || has_extension(MARKDOWN_EXTENSIONS) {
      return Some(FileKind::Markdown);
    }
    None
  }

  /// Gets the language whose `<language>.enabled` configuration decides
  /// if the file is formatted.
  pub fn language(&self) -> Language {
    match self {
      FileKind::Js(source_type) if source_type.is_typescript() => Language::TypeScript,
      FileKind::Js(_) => Language::JavaScript,
      FileKind::Json(JsonVariant::Jsonc) => Language::Jsonc,
      FileKind::Json(JsonVariant::Json5) => Language::Json5,
      FileKind::Json(_) | FileKind::PackageJson => Language::Json,
      FileKind::Graphql => Language::Graphql,
      FileKind::Css(CssVariant::Scss) => Language::Scss,
      FileKind::Css(CssVariant::Less) => Language::Less,
      FileKind::Css(_) => Language::Css,
      FileKind::Yaml | FileKind::YamlRc => Language::Yaml,
      FileKind::Markdown => Language::Markdown,
      FileKind::Toml => Language::Toml,
    }
  }
}

/// The file extensions the plugin formats with the provided configuration.
pub fn file_extensions(config: &Configuration) -> Vec<String> {
  enabled_items(
    config,
    &[
      (Language::TypeScript, TS_EXTENSIONS),
      (Language::JavaScript, JS_EXTENSIONS),
      (Language::JavaScript, ADDITIONAL_JS_EXTENSIONS),
      (Language::Toml, &["toml"]),
      (Language::Json, &["importmap"]),
      (Language::Json, JSON_EXTENSIONS),
      (Language::Jsonc, JSONC_EXTENSIONS),
      (Language::Json5, &["json5"]),
      (Language::Graphql, GRAPHQL_EXTENSIONS),
      (Language::Css, CSS_EXTENSIONS),
      (Language::Scss, &["scss"]),
      (Language::Less, &["less"]),
      (Language::Yaml, YAML_EXTENSIONS),
      (Language::Markdown, MARKDOWN_EXTENSIONS),
    ],
  )
}

/// The names of the files the plugin formats regardless of their extension
/// with the provided configuration.
pub fn file_names(config: &Configuration) -> Vec<String> {
  enabled_items(
    config,
    &[
      (Language::JavaScript, JS_FILE_NAMES),
      (Language::Toml, TOML_FILE_NAMES),
      (Language::Json, JSON_FILE_NAMES),
      (Language::Yaml, YAML_RC_FILE_NAMES),
      (Language::Yaml, YAML_FILE_NAMES),
      (Language::Markdown, MARKDOWN_FILE_NAMES),
    ],
  )
}

fn enabled_items(config: &Configuration, items: &[(Language, &[&str])]) -> Vec<String> {
  items
    .iter()
    .filter(|(language, _)| config.is_language_enabled(*language))
    .flat_map(|(_, items)| items.iter())
    .map(|item| item.to_string())
    .collect()
}

fn is_extra_js_file(file_name: &str, extension: Option<&str>) -> bool {
  if contains(JS_FILE_NAMES, file_name) {
    return true;
  }
  let Some(extension) = extension else {
    return false;
  };
  if contains(ADDITIONAL_JS_EXTENSIONS, extension) {
    return true;
  }
  // only `*.start.frag` and `*.end.frag` are JS
  if extension == "frag" {
    let stem = &file_name[..file_name.len() - ".frag".len()];
    return stem.ends_with(".start") || stem.ends_with(".end");
  }
  false
}

fn contains(items: &[&str], value: &str) -> bool {
  items.iter().any(|item| item.eq_ignore_ascii_case(value))
}

// A few of the files that are classified above can't be described to dprint by an
// extension or file name without also matching files the plugin doesn't format
// (`*.json.example`, `*.toml.example`, `*.tfstate.backup`, `*.start.frag`, and
// `*.end.frag`), so those are only formatted when associated with the plugin.
//
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

// these two are the extensions understood by `SourceType::from_path`
const TS_EXTENSIONS: &[&str] = &["ts", "tsx", "cts", "mts"];
const JS_EXTENSIONS: &[&str] = &["js", "jsx", "cjs", "mjs"];

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
  fn classifies_files_case_insensitively() {
    let kind = |path: &str| FileKind::from_path(Path::new(path));
    assert!(matches!(kind("File.TS"), Some(FileKind::Js(_))));
    assert!(matches!(kind("jakefile"), Some(FileKind::Js(_))));
    assert_eq!(kind("FILE.JSON"), Some(FileKind::Json(JsonVariant::Json)));
    assert_eq!(kind("form.4dform"), Some(FileKind::Json(JsonVariant::Json)));
    assert_eq!(kind("Package.JSON"), Some(FileKind::PackageJson));
    assert_eq!(kind("STYLE.SCSS"), Some(FileKind::Css(CssVariant::Scss)));
    assert_eq!(kind("citation.CFF"), Some(FileKind::Yaml));
    assert_eq!(kind("pipfile"), Some(FileKind::Toml));
    assert_eq!(kind("README.MD"), Some(FileKind::Markdown));
    assert_eq!(kind("readme"), Some(FileKind::Markdown));
    assert_eq!(kind("CARGO.LOCK"), None);
  }

  #[test]
  fn excludes_lock_files() {
    for name in EXCLUDE_FILE_NAMES {
      assert_eq!(FileKind::from_path(Path::new(name)), None, "{name}");
    }
  }

  #[test]
  fn matched_files_are_classified() {
    let config = Configuration::default();
    for ext in file_extensions(&config) {
      let path = format!("file.{ext}");
      assert!(FileKind::from_path(Path::new(&path)).is_some(), "{path}");
    }
    for name in file_names(&config) {
      assert!(FileKind::from_path(Path::new(&name)).is_some(), "{name}");
    }
  }

  #[test]
  fn markdown_is_matched() {
    let config = Configuration::default();
    assert!(file_extensions(&config).contains(&"md".to_string()));
    assert!(file_names(&config).contains(&"README".to_string()));
    assert!(file_extensions(&config).contains(&"json".to_string()));
  }

  #[test]
  fn disabled_languages_are_not_matched() {
    let config = Configuration {
      markdown_enabled: Some(false),
      typescript_enabled: Some(false),
      scss_enabled: Some(false),
      ..Default::default()
    };
    let extensions = file_extensions(&config);
    for ext in ["md", "markdown", "ts", "tsx", "scss"] {
      assert!(!extensions.contains(&ext.to_string()), "{ext}");
    }
    for ext in ["js", "jsx", "css", "less", "json"] {
      assert!(extensions.contains(&ext.to_string()), "{ext}");
    }
    let names = file_names(&config);
    assert!(!names.contains(&"README".to_string()));
    assert!(names.contains(&"Pipfile".to_string()));
  }

  #[test]
  fn disabling_a_language_only_unmatches_its_files() {
    let paths = |config: &Configuration| {
      let extensions = file_extensions(config).into_iter().map(|ext| format!("file.{ext}"));
      extensions.chain(file_names(config)).collect::<Vec<_>>()
    };
    let all_paths = paths(&Configuration::default());
    type DisableFn = fn(&mut Configuration);
    let disable_fns: [(Language, DisableFn); 12] = [
      (Language::JavaScript, |config| config.javascript_enabled = Some(false)),
      (Language::TypeScript, |config| config.typescript_enabled = Some(false)),
      (Language::Json, |config| config.json_enabled = Some(false)),
      (Language::Jsonc, |config| config.jsonc_enabled = Some(false)),
      (Language::Json5, |config| config.json5_enabled = Some(false)),
      (Language::Graphql, |config| config.graphql_enabled = Some(false)),
      (Language::Css, |config| config.css_enabled = Some(false)),
      (Language::Scss, |config| config.scss_enabled = Some(false)),
      (Language::Less, |config| config.less_enabled = Some(false)),
      (Language::Yaml, |config| config.yaml_enabled = Some(false)),
      (Language::Markdown, |config| config.markdown_enabled = Some(false)),
      (Language::Toml, |config| config.toml_enabled = Some(false)),
    ];
    for (disabled, disable) in disable_fns {
      let mut config = Configuration::default();
      disable(&mut config);
      let matched = paths(&config);
      let mut unmatched_count = 0;
      for path in &all_paths {
        let language = FileKind::from_path(Path::new(path)).unwrap().language();
        assert_eq!(matched.contains(path), language != disabled, "{path}");
        unmatched_count += (language == disabled) as usize;
      }
      assert!(unmatched_count > 0, "{disabled:?}");
    }
  }
}
