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

generate_str_to_from![QuoteProperties, [AsNeeded, "asNeeded"], [Preserve, "preserve"], [Consistent, "consistent"]];

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
}
