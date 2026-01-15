use super::Configuration;
use super::IndentStyle;
use super::LineEnding;
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
    experimental_sort_imports: resolve_sort_imports_options(&mut config, &mut diagnostics),
    experimental_tailwindcss: resolve_tailwindcss_options(&mut config, &mut diagnostics),
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

  let config_path = get_nullable_value::<String>(&mut obj, "config", &mut inner_diagnostics);
  let stylesheet = get_nullable_value::<String>(&mut obj, "stylesheet", &mut inner_diagnostics);
  let preserve_whitespace =
    get_nullable_value::<bool>(&mut obj, "preserveWhitespace", &mut inner_diagnostics).unwrap_or(false);
  let preserve_duplicates =
    get_nullable_value::<bool>(&mut obj, "preserveDuplicates", &mut inner_diagnostics).unwrap_or(false);

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
    config: config_path,
    stylesheet,
    functions,
    attributes,
    preserve_whitespace,
    preserve_duplicates,
  })
}
