use anyhow::Result;
use anyhow::bail;
use oxc_allocator::Allocator;
use oxc_formatter::ArrowParentheses;
use oxc_formatter::FormatOptions;
use oxc_formatter::Formatter;
use oxc_formatter::IndentStyle;
use oxc_formatter::IndentWidth;
use oxc_formatter::LineEnding;
use oxc_formatter::LineWidth;
use oxc_formatter::QuoteProperties;
use oxc_formatter::QuoteStyle;
use oxc_formatter::Semicolons;
use oxc_formatter::TrailingCommas;
use oxc_parser::ParseOptions;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::path::Path;

use crate::configuration::Configuration;

pub fn format_text(file_path: &Path, input_text: &str, config: &Configuration) -> Result<Option<String>> {
  let source_type = match SourceType::from_path(file_path) {
    Ok(source_type) => source_type,
    Err(_) => return Ok(None),
  };

  let allocator = Allocator::default();
  let parse_options = ParseOptions { preserve_parens: false, ..Default::default() };
  let parsed = Parser::new(&allocator, input_text, source_type)
    .with_options(parse_options)
    .parse();

  if !parsed.errors.is_empty() {
    let mut error_text = String::new();
    for (i, error) in parsed.errors.iter().enumerate() {
      if i > 0 {
        error_text.push('\n');
      }
      error_text.push_str(&error.to_string());
    }
    bail!("{}", error_text);
  }

  let options = build_format_options(config);
  let formatter = Formatter::new(&allocator, options);
  let output = formatter.build(&parsed.program);

  if output == input_text {
    Ok(None)
  } else {
    Ok(Some(output))
  }
}

fn build_format_options(config: &Configuration) -> FormatOptions {
  let mut options = FormatOptions::default();

  if let Some(line_ending) = config.line_ending {
    options.line_ending = match line_ending {
      crate::configuration::LineEnding::Lf => LineEnding::Lf,
      crate::configuration::LineEnding::Cr => LineEnding::Cr,
      crate::configuration::LineEnding::Crlf => LineEnding::Crlf,
    };
  }

  if let Some(indent_style) = config.indent_style {
    options.indent_style = match indent_style {
      crate::configuration::IndentStyle::Tab => IndentStyle::Tab,
      crate::configuration::IndentStyle::Space => IndentStyle::Space,
    };
  }

  if let Some(value) = config.indent_width {
    if let Ok(width) = IndentWidth::try_from(value) {
      options.indent_width = width;
    }
  }

  if let Some(value) = config.line_width {
    if let Ok(width) = LineWidth::try_from(value) {
      options.line_width = width;
    }
  }

  if let Some(semicolons) = config.semicolons {
    options.semicolons = match semicolons {
      crate::configuration::Semicolons::Always => Semicolons::Always,
      crate::configuration::Semicolons::AsNeeded => Semicolons::AsNeeded,
    };
  }

  if let Some(quote_style) = config.quote_style {
    options.quote_style = match quote_style {
      crate::configuration::QuoteStyle::Single => QuoteStyle::Single,
      crate::configuration::QuoteStyle::Double => QuoteStyle::Double,
    };
  }

  if let Some(quote_style) = config.jsx_quote_style {
    options.jsx_quote_style = match quote_style {
      crate::configuration::QuoteStyle::Single => QuoteStyle::Single,
      crate::configuration::QuoteStyle::Double => QuoteStyle::Double,
    };
  }

  if let Some(quote_properties) = config.quote_properties {
    options.quote_properties = match quote_properties {
      crate::configuration::QuoteProperties::AsNeeded => QuoteProperties::AsNeeded,
      crate::configuration::QuoteProperties::Preserve => QuoteProperties::Preserve,
      crate::configuration::QuoteProperties::Consistent => QuoteProperties::Consistent,
    };
  }

  if let Some(arrow_parens) = config.arrow_parentheses {
    options.arrow_parentheses = match arrow_parens {
      crate::configuration::ArrowParentheses::Always => ArrowParentheses::Always,
      crate::configuration::ArrowParentheses::AsNeeded => ArrowParentheses::AsNeeded,
    };
  }

  if let Some(trailing_commas) = config.trailing_commas {
    options.trailing_commas = match trailing_commas {
      crate::configuration::TrailingCommas::All => TrailingCommas::All,
      crate::configuration::TrailingCommas::Es5 => TrailingCommas::Es5,
      crate::configuration::TrailingCommas::None => TrailingCommas::None,
    };
  }

  if let Some(bracket_spacing) = config.bracket_spacing {
    options.bracket_spacing = bracket_spacing.into();
  }

  if let Some(bracket_same_line) = config.bracket_same_line {
    options.bracket_same_line = bracket_same_line.into();
  }

  options
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
