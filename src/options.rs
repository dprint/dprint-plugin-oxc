use oxc_formatter::ArrowParentheses;
use oxc_formatter::AttributePosition;
use oxc_formatter::CommentLineStrategy;
use oxc_formatter::CustomGroupDefinition;
use oxc_formatter::Expand;
use oxc_formatter::GroupEntry;
use oxc_formatter::ImportModifier;
use oxc_formatter::ImportSelector;
use oxc_formatter::JsFormatOptions;
use oxc_formatter::JsdocOptions;
use oxc_formatter::LineWrappingStyle;
use oxc_formatter::OperatorPosition;
use oxc_formatter::QuoteProperties;
use oxc_formatter::QuoteStyle;
use oxc_formatter::Semicolons;
use oxc_formatter::SortImportsOptions;
use oxc_formatter::SortOrder;
use oxc_formatter::SortTailwindcssOptions;
use oxc_formatter::TrailingCommas;
use oxc_formatter_core::CoreFormatOptions;
use oxc_formatter_core::FormatOptions;
use oxc_formatter_core::IndentStyle;
use oxc_formatter_core::IndentWidth;
use oxc_formatter_core::LineEnding;
use oxc_formatter_core::LineWidth;
use oxc_formatter_css::CssFormatOptions;
use oxc_formatter_css::CssVariant;
use oxc_formatter_graphql::GraphqlFormatOptions;
use oxc_formatter_json::JsonFormatOptions;
use oxc_formatter_json::JsonVariant;
use oxc_formatter_markdown::MarkdownFormatOptions;
use oxc_formatter_yaml::YamlFormatOptions;

use crate::configuration::Configuration;

// The functions in this file map this plugin's configuration onto the options of each
// of oxc's formatters the same way oxfmt maps its configuration onto them (see
// `apps/oxfmt/src/core/options` in the oxc repo). The options are shared between the
// languages, so for example `quoteStyle` applies to JS, CSS, JSON5, YAML, and Markdown.

/// Builds the options every formatter shares.
pub fn build_core_options(config: &Configuration) -> CoreFormatOptions {
  let mut options = CoreFormatOptions::default();

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

  if let Some(value) = config.indent_width
    && let Ok(width) = IndentWidth::try_from(value)
  {
    options.indent_width = width;
  }

  if let Some(value) = config.line_width
    && let Ok(width) = LineWidth::try_from(value)
  {
    options.line_width = width;
  }

  options
}

pub fn build_js_options(config: &Configuration) -> JsFormatOptions {
  let mut options = JsFormatOptions::default();
  options.apply_core(build_core_options(config));

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

  if let Some(attribute_position) = config.attribute_position {
    options.attribute_position = match attribute_position {
      crate::configuration::AttributePosition::Auto => AttributePosition::Auto,
      crate::configuration::AttributePosition::Multiline => AttributePosition::Multiline,
    };
  }

  if let Some(expand) = config.expand {
    options.expand = match expand {
      crate::configuration::Expand::Auto => Expand::Auto,
      crate::configuration::Expand::Never => Expand::Never,
    };
  }

  if let Some(operator_position) = config.operator_position {
    options.operator_position = match operator_position {
      crate::configuration::OperatorPosition::Start => OperatorPosition::Start,
      crate::configuration::OperatorPosition::End => OperatorPosition::End,
    };
  }

  if let Some(experimental_ternaries) = config.experimental_ternaries {
    options.experimental_ternaries = experimental_ternaries;
  }

  if let Some(html_whitespace_sensitivity_ignore) = config.html_whitespace_sensitivity_ignore {
    options.html_whitespace_sensitivity_ignore = html_whitespace_sensitivity_ignore;
  }

  options.sort_imports = config
    .experimental_sort_imports
    .as_ref()
    .map(build_sort_imports_options);

  if let Some(ref tailwindcss) = config.experimental_tailwindcss {
    options.sort_tailwindcss = Some(SortTailwindcssOptions {
      functions: tailwindcss.functions.clone(),
      attributes: tailwindcss.attributes.clone(),
      preserve_whitespace: tailwindcss.preserve_whitespace,
    });
  }

  if let Some(ref jsdoc) = config.jsdoc {
    options.jsdoc = Some(JsdocOptions {
      capitalize_descriptions: jsdoc.capitalize_descriptions,
      comment_line_strategy: jsdoc
        .comment_line_strategy
        .map(|strategy| match strategy {
          crate::configuration::CommentLineStrategy::SingleLine => CommentLineStrategy::SingleLine,
          crate::configuration::CommentLineStrategy::Multiline => CommentLineStrategy::Multiline,
          crate::configuration::CommentLineStrategy::Keep => CommentLineStrategy::Keep,
        })
        .unwrap_or_default(),
      separate_tag_groups: jsdoc.separate_tag_groups,
      separate_returns_from_param: jsdoc.separate_returns_from_param,
      bracket_spacing: jsdoc.bracket_spacing,
      description_with_dot: jsdoc.description_with_dot,
      add_default_to_description: jsdoc.add_default_to_description,
      prefer_code_fences: jsdoc.prefer_code_fences,
      line_wrapping_style: jsdoc
        .line_wrapping_style
        .map(|style| match style {
          crate::configuration::LineWrappingStyle::Greedy => LineWrappingStyle::Greedy,
          crate::configuration::LineWrappingStyle::Balance => LineWrappingStyle::Balance,
        })
        .unwrap_or_default(),
      description_tag: jsdoc.description_tag,
      keep_unparsable_example_indent: jsdoc.keep_unparsable_example_indent,
    });
  }

  options
}

pub fn build_sort_imports_options(sort_imports: &crate::configuration::SortImportsOptions) -> SortImportsOptions {
  SortImportsOptions {
    partition_by_newline: sort_imports.partition_by_newline,
    partition_by_comment: sort_imports.partition_by_comment,
    sort_side_effects: sort_imports.sort_side_effects,
    order: sort_imports
      .order
      .map(|o| match o {
        crate::configuration::SortOrder::Asc => SortOrder::Asc,
        crate::configuration::SortOrder::Desc => SortOrder::Desc,
      })
      .unwrap_or_default(),
    ignore_case: sort_imports.ignore_case.unwrap_or(true),
    newlines_between: sort_imports.newlines_between.unwrap_or(true),
    internal_pattern: sort_imports.internal_pattern.clone(),
    groups: sort_imports
      .groups
      .iter()
      .map(|group| group.iter().map(|s| GroupEntry::parse(s)).collect())
      .collect(),
    custom_groups: sort_imports
      .custom_groups
      .iter()
      .map(|g| CustomGroupDefinition {
        group_name: g.group_name.clone(),
        element_name_pattern: g.element_name_pattern.clone(),
        selector: g.selector.map(|selector| match selector {
          crate::configuration::ImportSelector::Type => ImportSelector::Type,
          crate::configuration::ImportSelector::SideEffectStyle => ImportSelector::SideEffectStyle,
          crate::configuration::ImportSelector::SideEffect => ImportSelector::SideEffect,
          crate::configuration::ImportSelector::Style => ImportSelector::Style,
          crate::configuration::ImportSelector::Index => ImportSelector::Index,
          crate::configuration::ImportSelector::Sibling => ImportSelector::Sibling,
          crate::configuration::ImportSelector::Parent => ImportSelector::Parent,
          crate::configuration::ImportSelector::Subpath => ImportSelector::Subpath,
          crate::configuration::ImportSelector::Internal => ImportSelector::Internal,
          crate::configuration::ImportSelector::Builtin => ImportSelector::Builtin,
          crate::configuration::ImportSelector::External => ImportSelector::External,
          crate::configuration::ImportSelector::Import => ImportSelector::Import,
        }),
        modifiers: g
          .modifiers
          .iter()
          .map(|modifier| match modifier {
            crate::configuration::ImportModifier::SideEffect => ImportModifier::SideEffect,
            crate::configuration::ImportModifier::Type => ImportModifier::Type,
            crate::configuration::ImportModifier::Value => ImportModifier::Value,
            crate::configuration::ImportModifier::Default => ImportModifier::Default,
            crate::configuration::ImportModifier::Wildcard => ImportModifier::Wildcard,
            crate::configuration::ImportModifier::Named => ImportModifier::Named,
          })
          .collect(),
      })
      .collect(),
    newline_boundary_overrides: sort_imports.newline_boundary_overrides.clone(),
  }
}

pub fn build_json_options(config: &Configuration, variant: JsonVariant) -> JsonFormatOptions {
  let mut options = JsonFormatOptions {
    variant,
    ..Default::default()
  };
  options.apply_core(build_core_options(config));

  // "all" and "es5" are indistinguishable for JSON
  if let Some(trailing_commas) = config.trailing_commas {
    options.trailing_commas = match trailing_commas {
      crate::configuration::TrailingCommas::All | crate::configuration::TrailingCommas::Es5 => {
        oxc_formatter_json::TrailingCommas::Always
      }
      crate::configuration::TrailingCommas::None => oxc_formatter_json::TrailingCommas::Never,
    };
  }

  if let Some(bracket_spacing) = config.bracket_spacing {
    options.bracket_spacing = bracket_spacing.into();
  }

  if let Some(expand) = config.expand {
    options.expand = match expand {
      crate::configuration::Expand::Auto => oxc_formatter_json::Expand::Auto,
      crate::configuration::Expand::Never => oxc_formatter_json::Expand::Never,
    };
  }

  if let Some(single_quote) = is_single_quote(config) {
    options.single_quote = single_quote.into();
  }

  if let Some(quote_properties) = config.quote_properties {
    options.quote_props = match quote_properties {
      crate::configuration::QuoteProperties::AsNeeded => oxc_formatter_json::QuoteProps::AsNeeded,
      crate::configuration::QuoteProperties::Preserve => oxc_formatter_json::QuoteProps::Preserve,
      crate::configuration::QuoteProperties::Consistent => oxc_formatter_json::QuoteProps::Consistent,
    };
  }

  options
}

pub fn build_css_options(config: &Configuration, variant: CssVariant) -> CssFormatOptions {
  // `sort_tailwindcss` is left off because sorting the classes of an `@apply`
  // requires a sorter, which oxc only has an implementation of in JS
  let mut options = CssFormatOptions {
    variant,
    ..Default::default()
  };
  options.apply_core(build_core_options(config));

  if let Some(single_quote) = is_single_quote(config) {
    options.single_quote = single_quote.into();
  }

  // "all" and "es5" are indistinguishable for CSS, where this only applies to SCSS maps
  if let Some(trailing_commas) = config.trailing_commas {
    options.trailing_commas = match trailing_commas {
      crate::configuration::TrailingCommas::All | crate::configuration::TrailingCommas::Es5 => {
        oxc_formatter_css::TrailingCommas::Always
      }
      crate::configuration::TrailingCommas::None => oxc_formatter_css::TrailingCommas::Never,
    };
  }

  options
}

pub fn build_graphql_options(config: &Configuration) -> GraphqlFormatOptions {
  let mut options = GraphqlFormatOptions::default();
  options.apply_core(build_core_options(config));

  if let Some(bracket_spacing) = config.bracket_spacing {
    options.bracket_spacing = bracket_spacing.into();
  }

  options
}

pub fn build_yaml_options(config: &Configuration) -> YamlFormatOptions {
  let mut options = YamlFormatOptions::default();
  options.apply_core(build_core_options(config));

  if let Some(prose_wrap) = config.prose_wrap {
    options.prose_wrap = match prose_wrap {
      crate::configuration::ProseWrap::Preserve => oxc_formatter_yaml::ProseWrap::Preserve,
      crate::configuration::ProseWrap::Always => oxc_formatter_yaml::ProseWrap::Always,
      crate::configuration::ProseWrap::Never => oxc_formatter_yaml::ProseWrap::Never,
    };
  }

  if let Some(single_quote) = is_single_quote(config) {
    options.single_quote = single_quote.into();
  }

  if let Some(bracket_spacing) = config.bracket_spacing {
    options.bracket_spacing = bracket_spacing.into();
  }

  // "all" and "es5" are indistinguishable for YAML, where this only applies to flow collections
  if let Some(trailing_commas) = config.trailing_commas {
    options.trailing_commas = match trailing_commas {
      crate::configuration::TrailingCommas::All | crate::configuration::TrailingCommas::Es5 => {
        oxc_formatter_yaml::TrailingCommas::Always
      }
      crate::configuration::TrailingCommas::None => oxc_formatter_yaml::TrailingCommas::Never,
    };
  }

  options
}

pub fn build_markdown_options(config: &Configuration) -> MarkdownFormatOptions {
  let mut options = MarkdownFormatOptions::default();
  options.apply_core(build_core_options(config));

  if let Some(prose_wrap) = config.prose_wrap {
    options.prose_wrap = match prose_wrap {
      crate::configuration::ProseWrap::Preserve => oxc_formatter_markdown::ProseWrap::Preserve,
      crate::configuration::ProseWrap::Always => oxc_formatter_markdown::ProseWrap::Always,
      crate::configuration::ProseWrap::Never => oxc_formatter_markdown::ProseWrap::Never,
    };
  }

  if let Some(single_quote) = is_single_quote(config) {
    options.single_quote = single_quote.into();
  }

  options
}

pub fn build_toml_options(config: &Configuration) -> oxc_toml::Options {
  let core_options = build_core_options(config);
  oxc_toml::Options {
    column_width: core_options.line_width.value() as usize,
    indent_string: if core_options.indent_style.is_tab() {
      "\t".to_string()
    } else {
      " ".repeat(core_options.indent_width.value() as usize)
    },
    crlf: core_options.line_ending.is_carriage_return_line_feed(),
    array_trailing_comma: !matches!(config.trailing_commas, Some(crate::configuration::TrailingCommas::None)),
    trailing_newline: true,
    ..Default::default()
  }
}

fn is_single_quote(config: &Configuration) -> Option<bool> {
  config
    .quote_style
    .map(|quote_style| quote_style == crate::configuration::QuoteStyle::Single)
}
