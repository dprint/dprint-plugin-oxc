use dprint_core::configuration::ParseConfigurationError;
use dprint_core::generate_str_to_from;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LineEnding {
  Lf,
  Cr,
  Crlf,
}

generate_str_to_from![LineEnding, [Lf, "lf"], [Cr, "cr"], [Crlf, "crlf"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IndentStyle {
  Tab,
  Space,
}

generate_str_to_from![IndentStyle, [Tab, "tab"], [Space, "space"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Semicolons {
  Always,
  AsNeeded,
}

generate_str_to_from![Semicolons, [Always, "always"], [AsNeeded, "asNeeded"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuoteStyle {
  Single,
  Double,
}

generate_str_to_from![QuoteStyle, [Single, "single"], [Double, "double"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuoteProperties {
  AsNeeded,
  Preserve,
  Consistent,
}

generate_str_to_from![
  QuoteProperties,
  [AsNeeded, "asNeeded"],
  [Preserve, "preserve"],
  [Consistent, "consistent"]
];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArrowParentheses {
  Always,
  AsNeeded,
}

generate_str_to_from![ArrowParentheses, [Always, "always"], [AsNeeded, "asNeeded"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TrailingCommas {
  All,
  Es5,
  None,
}

generate_str_to_from![TrailingCommas, [All, "all"], [Es5, "es5"], [None, "none"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AttributePosition {
  Auto,
  Multiline,
}

generate_str_to_from![AttributePosition, [Auto, "auto"], [Multiline, "multiline"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Expand {
  Auto,
  Never,
}

generate_str_to_from![Expand, [Auto, "auto"], [Never, "never"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EmbeddedLanguageFormatting {
  Auto,
  Off,
}

generate_str_to_from![EmbeddedLanguageFormatting, [Auto, "auto"], [Off, "off"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OperatorPosition {
  Start,
  End,
}

generate_str_to_from![OperatorPosition, [Start, "start"], [End, "end"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortOrder {
  Asc,
  Desc,
}

generate_str_to_from![SortOrder, [Asc, "asc"], [Desc, "desc"]];

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortImportsOptions {
  #[serde(default)]
  pub partition_by_newline: bool,
  #[serde(default)]
  pub partition_by_comment: bool,
  #[serde(default)]
  pub sort_side_effects: bool,
  pub order: Option<SortOrder>,
  pub ignore_case: Option<bool>,
  pub newlines_between: Option<bool>,
  #[serde(default)]
  pub internal_pattern: Vec<String>,
  #[serde(default)]
  pub groups: Vec<Vec<String>>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TailwindcssOptions {
  pub config: Option<String>,
  pub stylesheet: Option<String>,
  #[serde(default)]
  pub functions: Vec<String>,
  #[serde(default)]
  pub attributes: Vec<String>,
  #[serde(default)]
  pub preserve_whitespace: bool,
  #[serde(default)]
  pub preserve_duplicates: bool,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
  pub line_ending: Option<LineEnding>,
  pub indent_style: Option<IndentStyle>,
  pub indent_width: Option<u8>,
  pub line_width: Option<u16>,
  pub semicolons: Option<Semicolons>,
  pub quote_style: Option<QuoteStyle>,
  pub jsx_quote_style: Option<QuoteStyle>,
  pub quote_properties: Option<QuoteProperties>,
  pub arrow_parentheses: Option<ArrowParentheses>,
  pub trailing_commas: Option<TrailingCommas>,
  pub bracket_spacing: Option<bool>,
  pub bracket_same_line: Option<bool>,
  pub attribute_position: Option<AttributePosition>,
  pub expand: Option<Expand>,
  pub embedded_language_formatting: Option<EmbeddedLanguageFormatting>,
  pub experimental_operator_position: Option<OperatorPosition>,
  pub experimental_ternaries: Option<bool>,
  pub experimental_sort_imports: Option<SortImportsOptions>,
  pub experimental_tailwindcss: Option<TailwindcssOptions>,
}
