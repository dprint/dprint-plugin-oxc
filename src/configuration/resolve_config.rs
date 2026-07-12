use super::CommentLineStrategy;
use super::Configuration;
use super::CustomGroupDefinition;
use super::ImportModifier;
use super::IndentStyle;
use super::JsdocOptions;
use super::LineEnding;
use super::LineWrappingStyle;
use super::SortImportsOptions;
use super::SortOrder;
use super::TailwindcssOptions;
use dprint_core::configuration::*;

/// Resolves configuration from a collection of key value strings.
///
/// # Example
///
/// ```
/// use dprint_core::configuration::ConfigKeyMap;
/// use dprint_core::configuration::resolve_global_config;
/// use dprint_plugin_oxc::configuration::resolve_config;
///
/// let mut config_map = ConfigKeyMap::new(); // get a collection of key value pairs from somewhere
/// let global_config_result = resolve_global_config(&mut config_map);
///
/// // check global_config_result.diagnostics here...
///
/// let config_result = resolve_config(
///     config_map,
///     &global_config_result.config
/// );
///
/// // check config_result.diagnostics here and use config_result.config
/// ```
pub fn resolve_config(
  config: ConfigKeyMap,
  global_config: &GlobalConfiguration,
) -> ResolveConfigurationResult<Configuration> {
  let mut diagnostics = Vec::new();
  let mut config = config;

  let indent_style = get_nullable_value(&mut config, "indentStyle", &mut diagnostics).or(global_config.use_tabs.map(
    |value| match value {
      true => IndentStyle::Tab,
      false => IndentStyle::Space,
    },
  ));
  let indent_width = get_nullable_value(&mut config, "indentWidth", &mut diagnostics)
    .or_else(|| get_nullable_value(&mut config, "indentSize", &mut diagnostics))
    .or(global_config.indent_width);
  let line_width = get_nullable_value(&mut config, "lineWidth", &mut diagnostics).or(
    global_config
      .line_width
      .map(|l| std::cmp::min(u16::MAX as u32, l) as u16),
  );

  let resolved_config = Configuration {
    line_ending: get_nullable_value(&mut config, "lineEnding", &mut diagnostics).or(
      match global_config.new_line_kind {
        Some(NewLineKind::CarriageReturnLineFeed) => Some(LineEnding::Crlf),
        Some(NewLineKind::LineFeed) => Some(LineEnding::Lf),
        _ => None,
      },
    ),
    indent_style,
    indent_width,
    line_width,
    semicolons: get_nullable_value(&mut config, "semicolons", &mut diagnostics),
    quote_style: get_nullable_value(&mut config, "quoteStyle", &mut diagnostics),
    jsx_quote_style: get_nullable_value(&mut config, "jsxQuoteStyle", &mut diagnostics),
    quote_properties: get_nullable_value(&mut config, "quoteProperties", &mut diagnostics),
    arrow_parentheses: get_nullable_value(&mut config, "arrowParentheses", &mut diagnostics),
    trailing_commas: get_nullable_value(&mut config, "trailingCommas", &mut diagnostics)
      .or_else(|| get_nullable_value(&mut config, "trailingComma", &mut diagnostics)),
    bracket_spacing: get_nullable_value(&mut config, "bracketSpacing", &mut diagnostics),
    bracket_same_line: get_nullable_value(&mut config, "bracketSameLine", &mut diagnostics),
    attribute_position: get_nullable_value(&mut config, "attributePosition", &mut diagnostics),
    expand: get_nullable_value(&mut config, "expand", &mut diagnostics),
    embedded_language_formatting: get_nullable_value(&mut config, "embeddedLanguageFormatting", &mut diagnostics),
    experimental_operator_position: get_nullable_value(&mut config, "experimentalOperatorPosition", &mut diagnostics),
    experimental_ternaries: get_nullable_value(&mut config, "experimentalTernaries", &mut diagnostics),
    html_whitespace_sensitivity_ignore: get_nullable_value(
      &mut config,
      "htmlWhitespaceSensitivityIgnore",
      &mut diagnostics,
    ),
    experimental_sort_imports: resolve_sort_imports_options(&mut config, &mut diagnostics),
    experimental_tailwindcss: resolve_tailwindcss_options(&mut config, &mut diagnostics),
    jsdoc: resolve_jsdoc_options(&mut config, &mut diagnostics),
  };

  diagnostics.extend(get_unknown_property_diagnostics(config));

  ResolveConfigurationResult {
    config: resolved_config,
    diagnostics,
  }
}

fn resolve_sort_imports_options(
  config: &mut ConfigKeyMap,
  diagnostics: &mut Vec<ConfigurationDiagnostic>,
) -> Option<SortImportsOptions> {
  let value = config.shift_remove("experimentalSortImports")?;

  let obj = match value.into_object() {
    Some(obj) => obj,
    None => {
      diagnostics.push(ConfigurationDiagnostic {
        property_name: "experimentalSortImports".to_string(),
        message: "expected an object".to_string(),
      });
      return None;
    }
  };

  let mut obj = obj;
  let mut inner_diagnostics = Vec::new();

  let partition_by_newline =
    get_nullable_value::<bool>(&mut obj, "partitionByNewline", &mut inner_diagnostics).unwrap_or(false);
  let partition_by_comment =
    get_nullable_value::<bool>(&mut obj, "partitionByComment", &mut inner_diagnostics).unwrap_or(false);
  let sort_side_effects =
    get_nullable_value::<bool>(&mut obj, "sortSideEffects", &mut inner_diagnostics).unwrap_or(false);
  let order = get_nullable_value::<SortOrder>(&mut obj, "order", &mut inner_diagnostics);
  let ignore_case = get_nullable_value::<bool>(&mut obj, "ignoreCase", &mut inner_diagnostics);
  let newlines_between = get_nullable_value::<bool>(&mut obj, "newlinesBetween", &mut inner_diagnostics);

  // Parse internalPattern as array of strings
  let internal_pattern = obj
    .shift_remove("internalPattern")
    .and_then(|v| v.into_array())
    .map(|arr| arr.into_iter().filter_map(|v| v.into_string()).collect::<Vec<_>>())
    .unwrap_or_default();

  // Parse groups as array of arrays of strings
  let groups = obj
    .shift_remove("groups")
    .and_then(|v| v.into_array())
    .map(|arr| {
      arr
        .into_iter()
        .filter_map(|v| {
          v.into_array()
            .map(|inner| inner.into_iter().filter_map(|s| s.into_string()).collect::<Vec<_>>())
        })
        .collect::<Vec<_>>()
    })
    .unwrap_or_else(|| {
      vec![
        vec!["builtin".to_string()],
        vec!["external".to_string()],
        vec!["internal".to_string(), "subpath".to_string()],
        vec!["parent".to_string(), "sibling".to_string(), "index".to_string()],
        vec!["style".to_string()],
        vec!["unknown".to_string()],
      ]
    });

  // Parse customGroups as array of objects with groupName and elementNamePattern
  let custom_groups = obj
    .shift_remove("customGroups")
    .and_then(|v| v.into_array())
    .map(|arr| {
      arr
        .into_iter()
        .filter_map(|v| {
          let mut obj = v.into_object()?;
          let group_name = obj
            .shift_remove("groupName")
            .and_then(|v| v.into_string())
            .unwrap_or_default();
          let element_name_pattern = obj
            .shift_remove("elementNamePattern")
            .and_then(|v| v.into_array())
            .map(|arr| arr.into_iter().filter_map(|v| v.into_string()).collect::<Vec<_>>())
            .unwrap_or_default();
          let selector = get_nullable_value(&mut obj, "selector", &mut inner_diagnostics);
          let modifiers = obj
            .shift_remove("modifiers")
            .and_then(|v| v.into_array())
            .map(|arr| {
              arr
                .into_iter()
                .filter_map(|v| v.into_string())
                .filter_map(|value| value.parse::<ImportModifier>().ok())
                .collect()
            })
            .unwrap_or_default();
          Some(CustomGroupDefinition {
            group_name,
            element_name_pattern,
            selector,
            modifiers,
          })
        })
        .collect::<Vec<_>>()
    })
    .unwrap_or_default();

  // Report unknown properties within experimentalSortImports
  for (key, _) in obj {
    inner_diagnostics.push(ConfigurationDiagnostic {
      property_name: format!("experimentalSortImports.{}", key),
      message: "Unknown property".to_string(),
    });
  }

  diagnostics.extend(inner_diagnostics);

  Some(SortImportsOptions {
    partition_by_newline,
    partition_by_comment,
    sort_side_effects,
    order,
    ignore_case,
    newlines_between,
    internal_pattern,
    groups,
    custom_groups,
  })
}

fn resolve_tailwindcss_options(
  config: &mut ConfigKeyMap,
  diagnostics: &mut Vec<ConfigurationDiagnostic>,
) -> Option<TailwindcssOptions> {
  let value = config.shift_remove("experimentalTailwindcss")?;

  let obj = match value.into_object() {
    Some(obj) => obj,
    None => {
      diagnostics.push(ConfigurationDiagnostic {
        property_name: "experimentalTailwindcss".to_string(),
        message: "expected an object".to_string(),
      });
      return None;
    }
  };

  let mut obj = obj;
  let mut inner_diagnostics = Vec::new();

  let preserve_whitespace =
    get_nullable_value::<bool>(&mut obj, "preserveWhitespace", &mut inner_diagnostics).unwrap_or(false);

  // Parse functions as array of strings
  let functions = obj
    .shift_remove("functions")
    .and_then(|v| v.into_array())
    .map(|arr| arr.into_iter().filter_map(|v| v.into_string()).collect::<Vec<_>>())
    .unwrap_or_default();

  // Parse attributes as array of strings
  let attributes = obj
    .shift_remove("attributes")
    .and_then(|v| v.into_array())
    .map(|arr| arr.into_iter().filter_map(|v| v.into_string()).collect::<Vec<_>>())
    .unwrap_or_default();

  // Report unknown properties within experimentalTailwindcss
  for (key, _) in obj {
    inner_diagnostics.push(ConfigurationDiagnostic {
      property_name: format!("experimentalTailwindcss.{}", key),
      message: "Unknown property".to_string(),
    });
  }

  diagnostics.extend(inner_diagnostics);

  Some(TailwindcssOptions {
    functions,
    attributes,
    preserve_whitespace,
  })
}

fn resolve_jsdoc_options(
  config: &mut ConfigKeyMap,
  diagnostics: &mut Vec<ConfigurationDiagnostic>,
) -> Option<JsdocOptions> {
  let value = config.shift_remove("jsdoc")?;
  let mut obj = match value.into_object() {
    Some(obj) => obj,
    None => {
      diagnostics.push(ConfigurationDiagnostic {
        property_name: "jsdoc".to_string(),
        message: "expected an object".to_string(),
      });
      return None;
    }
  };
  let mut inner_diagnostics = Vec::new();
  let options = JsdocOptions {
    capitalize_descriptions: get_nullable_value(&mut obj, "capitalizeDescriptions", &mut inner_diagnostics)
      .unwrap_or(true),
    comment_line_strategy: get_nullable_value::<CommentLineStrategy>(
      &mut obj,
      "commentLineStrategy",
      &mut inner_diagnostics,
    ),
    separate_tag_groups: get_nullable_value(&mut obj, "separateTagGroups", &mut inner_diagnostics).unwrap_or(false),
    separate_returns_from_param: get_nullable_value(&mut obj, "separateReturnsFromParam", &mut inner_diagnostics)
      .unwrap_or(false),
    bracket_spacing: get_nullable_value(&mut obj, "bracketSpacing", &mut inner_diagnostics).unwrap_or(false),
    description_with_dot: get_nullable_value(&mut obj, "descriptionWithDot", &mut inner_diagnostics).unwrap_or(false),
    add_default_to_description: get_nullable_value(&mut obj, "addDefaultToDescription", &mut inner_diagnostics)
      .unwrap_or(true),
    prefer_code_fences: get_nullable_value(&mut obj, "preferCodeFences", &mut inner_diagnostics).unwrap_or(false),
    line_wrapping_style: get_nullable_value::<LineWrappingStyle>(&mut obj, "lineWrappingStyle", &mut inner_diagnostics),
    description_tag: get_nullable_value(&mut obj, "descriptionTag", &mut inner_diagnostics).unwrap_or(false),
    keep_unparsable_example_indent: get_nullable_value(&mut obj, "keepUnparsableExampleIndent", &mut inner_diagnostics)
      .unwrap_or(false),
  };
  for (key, _) in obj {
    inner_diagnostics.push(ConfigurationDiagnostic {
      property_name: format!("jsdoc.{}", key),
      message: "Unknown property".to_string(),
    });
  }
  diagnostics.extend(inner_diagnostics);
  Some(options)
}
