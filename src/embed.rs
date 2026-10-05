use std::sync::Arc;

use oxc_allocator::Allocator;
use oxc_formatter::CssInJsTemplate;
use oxc_formatter::JsEmbeddedIn;
use oxc_formatter::MarkdownInJsTemplate;
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
use oxc_formatter_css::CssFormatOptions;
use oxc_formatter_css::CssVariant;
use oxc_formatter_graphql::GraphqlFormatOptions;
use oxc_formatter_json::JsonFormatOptions;
use oxc_formatter_json::JsonVariant;
use oxc_formatter_markdown::MarkdownFormatOptions;
use oxc_formatter_markdown::XxxInMarkdownCodeBlock;
use oxc_formatter_yaml::YamlFormatOptions;
use oxc_span::SourceType;

use crate::configuration::Configuration;
use crate::configuration::EmbeddedLanguageFormatting;
use crate::options::build_core_options;
use crate::options::build_css_options;
use crate::options::build_graphql_options;
use crate::options::build_js_options;
use crate::options::build_json_options;
use crate::options::build_markdown_options;
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
    Ok(match request.language {
      "javascript" => to_response(oxc_formatter::format_to_ir(
        session,
        text,
        SourceType::from_extension("js").expect("js is a supported source type"),
        options.js.clone(),
        request
          .parent_context
          .and_then(|context| context.downcast_ref::<XxxInMarkdownCodeBlock>())
          .map(|_| JsEmbeddedIn::MarkdownCodeBlock),
      )),
      "typescript" | "angular-ts" => to_response(oxc_formatter::format_to_ir(
        session,
        text,
        SourceType::from_extension("ts").expect("ts is a supported source type"),
        options.js.clone(),
        request
          .parent_context
          .and_then(|context| context.downcast_ref::<XxxInMarkdownCodeBlock>())
          .map(|_| JsEmbeddedIn::MarkdownCodeBlock),
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
          is_css_in_js,
        ))
      }
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
          options.js.clone(),
          request
            .parent_context
            .and_then(|context| context.downcast_ref::<XxxInMarkdownCodeBlock>())
            .map(|_| JsEmbeddedIn::MarkdownCodeBlock),
        )),
        Err(_) => DispatchResponse::PreserveOriginal,
      },
      // oxfmt formats these with Prettier (ex. html) or not at all
    })
  })
}

/// The options of the embedded languages, which are those of the file they're embedded in.
struct EmbeddedOptions {
  js: oxc_formatter::JsFormatOptions,
  graphql: GraphqlFormatOptions,
  css: CssFormatOptions,
  yaml: YamlFormatOptions,
  json: JsonFormatOptions,
  markdown: MarkdownFormatOptions,
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
    }
  }
}

/// Embedded code that fails to parse is left as-is rather than failing to format the file.
fn to_response<'a, E>(result: Result<EmbeddedIr<'a>, E>) -> DispatchResponse<'a> {
  match result {
    Ok(embedded) => DispatchResponse::Formatted(embedded.into()),
    Err(_) => DispatchResponse::PreserveOriginal,
  }
}

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
        parent_context: None,
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
