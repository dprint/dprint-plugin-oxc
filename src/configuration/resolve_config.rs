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
use super::SortPackageJsonOptions;
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
    operator_position: get_nullable_value(&mut config, "operatorPosition", &mut diagnostics),
    experimental_ternaries: get_nullable_value(&mut config, "experimentalTernaries", &mut diagnostics),
    html_whitespace_sensitivity_ignore: get_nullable_value(
      &mut config,
      "htmlWhitespaceSensitivityIgnore",
      &mut diagnostics,
    ),
    prose_wrap: get_nullable_value(&mut config, "proseWrap", &mut diagnostics),
    embedded_language_formatting: get_nullable_value(&mut config, "embeddedLanguageFormatting", &mut diagnostics),
    insert_final_newline: get_nullable_value(&mut config, "insertFinalNewline", &mut diagnostics),
    sort_package_json: resolve_sort_package_json_options(&mut config, &mut diagnostics),
    experimental_sort_imports: resolve_sort_imports_options(&mut config, &mut diagnostics),
    experimental_tailwindcss: resolve_tailwindcss_options(&mut config, &mut diagnostics),
    jsdoc: resolve_jsdoc_options(&mut config, &mut diagnostics),
  };

  if let Some(sort_imports) = &resolved_config.experimental_sort_imports
    && let Err(message) = crate::options::build_sort_imports_options(sort_imports).validate()
  {
    diagnostics.push(ConfigurationDiagnostic {
      property_name: "sortImports".to_string(),
      message,
    });
  }

  diagnostics.extend(get_unknown_property_diagnostics(config));

  ResolveConfigurationResult {
    config: resolved_config,
    diagnostics,
  }
}

fn resolve_sort_package_json_options(
  config: &mut ConfigKeyMap,
  diagnostics: &mut Vec<ConfigurationDiagnostic>,
) -> Option<SortPackageJsonOptions> {
  let property_name = "sortPackageJson";
  match config.shift_remove(property_name)? {
    ConfigKeyValue::Bool(enabled) => Some(SortPackageJsonOptions {
      enabled,
      sort_scripts: false,
    }),
    ConfigKeyValue::Object(mut obj) => {
      let sort_scripts = get_nullable_value(&mut obj, "sortScripts", diagnostics).unwrap_or(false);
      for (key, _) in obj {
        diagnostics.push(ConfigurationDiagnostic {
          property_name: format!("{property_name}.{key}"),
          message: "Unknown property".to_string(),
        });
      }
      Some(SortPackageJsonOptions {
        enabled: true,
        sort_scripts,
      })
    }
    ConfigKeyValue::Null => None,
    _ => {
      diagnostics.push(ConfigurationDiagnostic {
        property_name: property_name.to_string(),
        message: "expected a boolean or an object".to_string(),
      });
      None
    }
  }
}

fn resolve_sort_imports_options(
  config: &mut ConfigKeyMap,
  diagnostics: &mut Vec<ConfigurationDiagnostic>,
) -> Option<SortImportsOptions> {
  let (property_name, mut obj) = take_toggle_object(config, &["sortImports", "experimentalSortImports"], diagnostics)?;
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

  let newline_boundary_overrides: Vec<Option<bool>> = obj
    .shift_remove("newlineBoundaryOverrides")
    .and_then(|v| v.into_array())
    .map(|values| {
      values
        .into_iter()
        .enumerate()
        .filter_map(|(index, value)| match value {
          ConfigKeyValue::Bool(value) => Some(Some(value)),
          ConfigKeyValue::Null => Some(None),
          _ => {
            inner_diagnostics.push(ConfigurationDiagnostic {
              property_name: format!("{property_name}.newlineBoundaryOverrides.{index}"),
              message: "Expected a boolean or null.".to_string(),
            });
            None
          }
        })
        .collect()
    })
    .unwrap_or_default();

  // Parse internalPattern as array of strings
  let internal_pattern = obj
    .shift_remove("internalPattern")
    .and_then(|v| v.into_array())
    .map(|arr| arr.into_iter().filter_map(|v| v.into_string()).collect::<Vec<_>>())
    .unwrap_or_else(|| vec!["~/".to_string(), "@/".to_string(), "#".to_string()]);

  // Parse groups, where each item is a group name, an array of group names, or a
  // `{ "newlinesBetween": bool }` marker for the boundary between two groups
  let (groups, marker_overrides) = match obj.shift_remove("groups").and_then(|v| v.into_array()) {
    Some(items) => resolve_sort_imports_groups(items, &property_name, &mut inner_diagnostics),
    None => (
      vec![
        vec!["builtin".to_string()],
        vec!["external".to_string()],
        vec!["internal".to_string(), "subpath".to_string()],
        vec!["parent".to_string(), "sibling".to_string(), "index".to_string()],
        vec!["style".to_string()],
        vec!["unknown".to_string()],
      ],
      Vec::new(),
    ),
  };
  let newline_boundary_overrides = if marker_overrides.iter().any(Option::is_some) {
    marker_overrides
  } else {
    newline_boundary_overrides
  };

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
      property_name: format!("{property_name}.{key}"),
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
    newline_boundary_overrides,
    internal_pattern,
    groups,
    custom_groups,
  })
}

fn resolve_tailwindcss_options(
  config: &mut ConfigKeyMap,
  diagnostics: &mut Vec<ConfigurationDiagnostic>,
) -> Option<TailwindcssOptions> {
  let (property_name, mut obj) =
    take_toggle_object(config, &["sortTailwindcss", "experimentalTailwindcss"], diagnostics)?;
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
      property_name: format!("{property_name}.{key}"),
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
  let (property_name, mut obj) = take_toggle_object(config, &["jsdoc"], diagnostics)?;
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
      property_name: format!("{property_name}.{key}"),
      message: "Unknown property".to_string(),
    });
  }
  diagnostics.extend(inner_diagnostics);
  Some(options)
}

fn resolve_sort_imports_groups(
  items: Vec<ConfigKeyValue>,
  property_name: &str,
  diagnostics: &mut Vec<ConfigurationDiagnostic>,
) -> (Vec<Vec<String>>, Vec<Option<bool>>) {
  let mut groups: Vec<Vec<String>> = Vec::new();
  let mut newline_boundary_overrides = Vec::new();
  let mut pending_override = None;
  let mut add_diagnostic = |message: &str| {
    diagnostics.push(ConfigurationDiagnostic {
      property_name: format!("{property_name}.groups"),
      message: message.to_string(),
    });
  };

  for item in items {
    let group = match item {
      ConfigKeyValue::String(name) => vec![name],
      ConfigKeyValue::Array(names) => names.into_iter().filter_map(|name| name.into_string()).collect(),
      ConfigKeyValue::Object(mut marker) => {
        match marker.shift_remove("newlinesBetween") {
          Some(ConfigKeyValue::Bool(value)) if marker.is_empty() => {
            if groups.is_empty() {
              add_diagnostic("`{ \"newlinesBetween\" }` marker cannot appear at the start of `groups`");
            } else if pending_override.is_some() {
              add_diagnostic("consecutive `{ \"newlinesBetween\" }` markers are not allowed in `groups`");
            } else {
              pending_override = Some(value);
            }
          }
          _ => add_diagnostic("expected an object item to only have a boolean `newlinesBetween` property"),
        }
        continue;
      }
      _ => {
        add_diagnostic("expected a string, an array of strings, or a `{ \"newlinesBetween\" }` marker");
        continue;
      }
    };
    if !groups.is_empty() {
      newline_boundary_overrides.push(pending_override.take());
    }
    groups.push(group);
  }

  if pending_override.is_some() {
    add_diagnostic("`{ \"newlinesBetween\" }` marker cannot appear at the end of `groups`");
  }

  (groups, newline_boundary_overrides)
}

/// Takes an option that is either a boolean toggle or an object of options, returning
/// the name of the property that was used along with its options when it's enabled.
fn take_toggle_object(
  config: &mut ConfigKeyMap,
  property_names: &[&str],
  diagnostics: &mut Vec<ConfigurationDiagnostic>,
) -> Option<(String, ConfigKeyMap)> {
  let (property_name, value) = property_names
    .iter()
    .find_map(|name| config.shift_remove(*name).map(|value| (name.to_string(), value)))?;
  match value {
    ConfigKeyValue::Object(obj) => Some((property_name, obj)),
    ConfigKeyValue::Bool(true) => Some((property_name, ConfigKeyMap::new())),
    ConfigKeyValue::Bool(false) | ConfigKeyValue::Null => None,
    _ => {
      diagnostics.push(ConfigurationDiagnostic {
        property_name,
        message: "expected a boolean or an object".to_string(),
      });
      None
    }
  }
}
