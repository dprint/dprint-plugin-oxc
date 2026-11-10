use std::sync::Arc;

use oxc_allocator::Allocator;
use oxc_formatter::CssInJsTemplate;
use oxc_formatter::ExpressionLayout;
use oxc_formatter::ExpressionRoot;
use oxc_formatter::HtmlInJsTemplate;
use oxc_formatter::JsEmbeddedIn;
use oxc_formatter::JsRoot;
use oxc_formatter::MarkdownInJsTemplate;
use oxc_formatter::ProgramLayout;
use oxc_formatter::ProgramSemicolons;
use oxc_formatter_core::DispatchRequest;
use oxc_formatter_core::DispatchResponse;
use oxc_formatter_core::EmbeddedIr;
use oxc_formatter_core::FormatDispatcher;
use oxc_formatter_core::FormatOptions;
use oxc_formatter_core::FormatSession;
use oxc_formatter_core::InputKind;
use oxc_formatter_core::PrintWidth;
use oxc_formatter_core::PrinterOptions;
use oxc_formatter_core::SessionServices;
use oxc_formatter_core::StringEmbedder;
use oxc_formatter_css::CssEmbeddedIn;
use oxc_formatter_css::CssFormatOptions;
use oxc_formatter_css::CssVariant;
use oxc_formatter_graphql::GraphqlFormatOptions;
use oxc_formatter_htmlish::CssInHtmlStyleAttribute;
use oxc_formatter_htmlish::HtmlishEmbeddedIn;
use oxc_formatter_htmlish::HtmlishFormatOptions;
use oxc_formatter_htmlish::HtmlishVariant;
use oxc_formatter_htmlish::JsInHtmlEventHandler;
use oxc_formatter_htmlish::JsInVueTemplate;
use oxc_formatter_json::JsonFormatOptions;
use oxc_formatter_json::JsonVariant;
use oxc_formatter_markdown::MarkdownFormatOptions;
use oxc_formatter_markdown::XxxInMarkdownCodeBlock;
use oxc_formatter_toml::TomlFormatOptions;
use oxc_formatter_yaml::YamlFormatOptions;
use oxc_span::SourceType;

use crate::configuration::Configuration;
use crate::configuration::EmbeddedLanguageFormatting;
use crate::options::build_core_options;
use crate::options::build_css_options;
use crate::options::build_graphql_options;
use crate::options::build_html_options;
use crate::options::build_js_options;
use crate::options::build_json_options;
use crate::options::build_markdown_options;
use crate::options::build_toml_options;
use crate::options::build_yaml_options;

/// Builds the services of a root formatter run, which are what format the code that's
/// embedded in a file (ex. CSS or GraphQL in a JS template literal, a fenced code block
/// in a JSDoc comment, or the front matter of a CSS file).
///
/// This mirrors the services of oxfmt's pure Rust build (see `apps/oxfmt/src/core/embed`
/// in the oxc repo), so there is no Tailwind sorter because oxc only has one in JS.
pub fn build_session_services(config: &Configuration) -> SessionServices {
  if config.embedded_language_formatting == Some(EmbeddedLanguageFormatting::Off) {
    return SessionServices::default();
  }

  let dispatcher = build_dispatcher(config);
  let string_embedder: StringEmbedder = {
    let dispatcher = dispatcher.clone();
    let printer_options = build_core_options(config).as_print_options();
    Arc::new(move |language: &str, code: &str, print_width: usize| {
      format_fence(language, code, print_width, &dispatcher, printer_options.clone())
    })
  };

  SessionServices {
    dispatcher: Some(dispatcher),
    string_embedder: Some(string_embedder),
    tailwind_sorter: None,
  }
}

fn build_dispatcher(config: &Configuration) -> FormatDispatcher {
  let options = EmbeddedOptions::new(config);
  Arc::new(move |session: &FormatSession<'_>, request: DispatchRequest<'_>| {
    let text = request.text;
    let (js_root, js_embedded_in) = js_root_of(&request);
    Ok(match request.language {
      "javascript" => to_response(oxc_formatter::format_to_ir(
        session,
        text,
        SourceType::from_extension("js").expect("js is a supported source type"),
        js_options(&options, js_root),
        js_root,
        js_embedded_in,
      )),
      "typescript" | "angular-ts" => to_response(oxc_formatter::format_to_ir(
        session,
        text,
        SourceType::from_extension("ts").expect("ts is a supported source type"),
        js_options(&options, js_root),
        js_root,
        js_embedded_in,
      )),
      "graphql" | "gql" => to_response(oxc_formatter_graphql::format_to_ir(session, text, options.graphql)),
      "css" | "postcss" | "scss" | "less" => {
        // css-in-js is always parsed as SCSS with `${}` placeholder markers
        let is_css_in_js = request
          .parent_context
          .is_some_and(|context| context.downcast_ref::<CssInJsTemplate>().is_some());
        let variant = match request.language {
          _ if is_css_in_js => CssVariant::Scss,
          "scss" => CssVariant::Scss,
          "less" => CssVariant::Less,
          _ => CssVariant::Css,
        };
        to_response(oxc_formatter_css::format_to_ir(
          session,
          text,
          CssFormatOptions { variant, ..options.css },
          if is_css_in_js {
            Some(CssEmbeddedIn::CssInJsTemplate)
          } else {
            request
              .parent_context_as::<CssInHtmlStyleAttribute>()
              .map(|_| CssEmbeddedIn::HtmlStyleAttribute)
          },
        ))
      }
      "html" | "vue" => {
        let embedded_in = request
          .parent_context_as::<HtmlInJsTemplate>()
          .map(|context| HtmlishEmbeddedIn {
            indent_multiple_roots: context.indent_multiple_roots,
            placeholder: Some(context.placeholder),
          })
          .or_else(|| (request.parent_context_as::<HtmlInJsdocFence>().is_none()).then(HtmlishEmbeddedIn::default));
        to_response(oxc_formatter_htmlish::format_to_ir(
          session,
          text,
          HtmlishFormatOptions {
            variant: if request.language == "vue" {
              HtmlishVariant::Vue
            } else {
              HtmlishVariant::Html
            },
            ..options.html.clone()
          },
          embedded_in,
        ))
      }
      "toml" => to_response(oxc_formatter_toml::format_to_ir(session, text, options.toml)),
      "yaml" | "yml" => to_response(oxc_formatter_yaml::format_to_ir(session, text, options.yaml)),
      "json" | "jsonc" | "json5" => {
        let variant = match request.language {
          "jsonc" => JsonVariant::Jsonc,
          "json5" => JsonVariant::Json5,
          _ => JsonVariant::Json,
        };
        to_response(oxc_formatter_json::format_to_ir(
          session,
          text,
          JsonFormatOptions {
            variant,
            ..options.json
          },
        ))
      }
      "markdown" | "md" => {
        let in_js_template = request
          .parent_context
          .is_some_and(|context| context.downcast_ref::<MarkdownInJsTemplate>().is_some())
          || request
            .parent_context
            .and_then(|context| context.downcast_ref::<XxxInMarkdownCodeBlock>())
            .is_some_and(|context| context.in_js_template);
        to_response(oxc_formatter_markdown::format_to_ir(
          session,
          text,
          options.markdown,
          in_js_template,
        ))
      }
      language => match SourceType::from_extension(language) {
        Ok(source_type) => to_response(oxc_formatter::format_to_ir(
          session,
          text,
          source_type,
          js_options(&options, js_root),
          js_root,
          js_embedded_in,
        )),
        Err(_) => DispatchResponse::PreserveOriginal,
      },
      // oxfmt formats other languages with Prettier or not at all
    })
  })
}

fn js_root_of(request: &DispatchRequest<'_>) -> (JsRoot, JsEmbeddedIn) {
  let in_attribute = |semicolons| JsRoot::Program {
    semicolons,
    layout: ProgramLayout::Indented,
  };
  let expression = |layout| ExpressionRoot {
    layout,
    tailwind_classes: false,
    ends_at_double_brace: false,
    reads_as_statements: false,
    in_parens: false,
  };
  if let Some(&part) = request.parent_context_as::<JsInVueTemplate>() {
    let root = match part {
      JsInVueTemplate::Interpolation => JsRoot::Expression(ExpressionRoot {
        ends_at_double_brace: true,
        ..expression(ExpressionLayout::Bare)
      }),
      JsInVueTemplate::Binding { tailwind_classes } => JsRoot::Expression(ExpressionRoot {
        tailwind_classes,
        ..expression(ExpressionLayout::Hug { literals: true })
      }),
      JsInVueTemplate::Expression {
        tailwind_classes,
        in_parens,
      } => JsRoot::Expression(ExpressionRoot {
        tailwind_classes,
        in_parens,
        ..expression(ExpressionLayout::Hug { literals: false })
      }),
      JsInVueTemplate::HandlerExpression => JsRoot::Expression(ExpressionRoot {
        reads_as_statements: true,
        in_parens: true,
        ..expression(ExpressionLayout::Hug { literals: false })
      }),
      JsInVueTemplate::ForSource => JsRoot::Expression(expression(ExpressionLayout::Group)),
      JsInVueTemplate::Handler => in_attribute(ProgramSemicolons::MarksStatements),
      JsInVueTemplate::ForAlias => JsRoot::FunctionParams { parens_when_many: true },
      JsInVueTemplate::SlotProps => JsRoot::FunctionParams {
        parens_when_many: false,
      },
      JsInVueTemplate::ScriptGeneric => JsRoot::TypeParameters,
    };
    let in_html_attribute = part != JsInVueTemplate::Interpolation;
    return (
      root,
      JsEmbeddedIn {
        in_html_attribute,
        inlines_code: true,
      },
    );
  }
  if request.parent_context_as::<JsInHtmlEventHandler>().is_some() {
    let root = in_attribute(ProgramSemicolons::LoneExpressionOmitted);
    return (
      root,
      JsEmbeddedIn {
        in_html_attribute: true,
        inlines_code: false,
      },
    );
  }
  let semicolons = if request.parent_context_as::<XxxInMarkdownCodeBlock>().is_some() {
    ProgramSemicolons::LoneJsxOmitted
  } else {
    ProgramSemicolons::AsOptions
  };
  let root = JsRoot::Program {
    semicolons,
    layout: ProgramLayout::Bare,
  };
  (root, JsEmbeddedIn::default())
}

fn js_options(options: &EmbeddedOptions, root: JsRoot) -> oxc_formatter::JsFormatOptions {
  let mut js = options.js.clone();
  if !matches!(
    root,
    JsRoot::Program {
      layout: ProgramLayout::Bare,
      ..
    }
  ) {
    js.sort_imports = None;
  }
  js
}

/// The options of the embedded languages, which are those of the file they're embedded in.
struct EmbeddedOptions {
  js: oxc_formatter::JsFormatOptions,
  graphql: GraphqlFormatOptions,
  css: CssFormatOptions,
  yaml: YamlFormatOptions,
  json: JsonFormatOptions,
  markdown: MarkdownFormatOptions,
  toml: TomlFormatOptions,
  html: HtmlishFormatOptions,
}

impl EmbeddedOptions {
  fn new(config: &Configuration) -> Self {
    Self {
      js: build_js_options(config),
      graphql: build_graphql_options(config),
      css: build_css_options(config, CssVariant::Css),
      yaml: build_yaml_options(config),
      json: build_json_options(config, JsonVariant::Json),
      markdown: build_markdown_options(config),
      toml: build_toml_options(config),
      html: build_html_options(config, HtmlishVariant::Html),
    }
  }
}

/// Embedded code that fails to parse is left as-is rather than failing to format the file.
fn to_response<'a, E>(result: Result<EmbeddedIr<'a>, E>) -> DispatchResponse<'a> {
  match result {
    Ok(embedded) => DispatchResponse::Formatted(embedded),
    Err(_) => DispatchResponse::PreserveOriginal,
  }
}

// marks a standalone HTML document in a JSDoc fence
struct HtmlInJsdocFence;

/// Formats a fenced code block in a JSDoc comment, where an error keeps the block as-is.
fn format_fence(
  language: &str,
  code: &str,
  print_width: usize,
  dispatcher: &FormatDispatcher,
  printer_options: PrinterOptions,
) -> Result<String, String> {
  let allocator = Allocator::default();
  let session = FormatSession::with_services(
    &allocator,
    InputKind::Fragment,
    SessionServices {
      dispatcher: Some(dispatcher.clone()),
      ..Default::default()
    },
  );
  // the width is what's available to the block at its position in the comment
  let printer_options =
    printer_options.with_print_width(PrintWidth::new(u32::try_from(print_width).unwrap_or(u32::MAX)));
  session
    .dispatch_to_string(
      DispatchRequest {
        language,
        text: code,
        input_kind: InputKind::Fragment,
        parent_context: Some(&HtmlInJsdocFence),
      },
      printer_options,
    )?
    .map(|mut code| {
      // the block is re-embedded line by line, so there is no trailing newline
      code.truncate(code.trim_end().len());
      code
    })
    .ok_or_else(|| format!("No formatter for '{language}'."))
}
