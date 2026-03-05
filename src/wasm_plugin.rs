use std::cell::UnsafeCell;
use std::path::Path;
use std::sync::Arc;

use super::configuration::Configuration;
use super::configuration::resolve_config;

use dprint_core::configuration::ConfigKeyMap;
use dprint_core::configuration::GlobalConfiguration;
use dprint_core::generate_plugin_code;
use dprint_core::plugins::CheckConfigUpdatesMessage;
use dprint_core::plugins::ConfigChange;
use dprint_core::plugins::FileMatchingInfo;
use dprint_core::plugins::FormatResult;
use dprint_core::plugins::PluginInfo;
use dprint_core::plugins::PluginResolveConfigurationResult;
use dprint_core::plugins::SyncFormatRequest;
use dprint_core::plugins::SyncHostFormatRequest;
use dprint_core::plugins::SyncPluginHandler;
use oxc_formatter::EmbeddedFormatterCallback;
use oxc_formatter::ExternalCallbacks;

// This is ok to do because Wasm plugins are executed on a single thread.
// Same pattern as used in generate_plugin_code! macro.
// https://github.com/dprint/dprint/blob/2e839405dcf75541bb3b59978c804304fedaa581/crates/core/src/plugins/wasm/mod.rs#L89-L114
struct WasmCallback<T>(UnsafeCell<T>);

unsafe impl<T> Send for WasmCallback<T> {}
unsafe impl<T> Sync for WasmCallback<T> {}

impl<T> WasmCallback<T> {
  fn new(value: T) -> Self {
    WasmCallback(UnsafeCell::new(value))
  }

  fn call<Args, Output>(&self, args: Args) -> Output
  where
    T: FnMut(Args) -> Output,
  {
    (unsafe { &mut *self.0.get() })(args)
  }
}

// Adapted from oxc-project/oxc
// https://github.com/oxc-project/oxc/blob/50eb16052247694e483828b2a603ade5e410c569/apps/oxfmt/src/core/external_formatter.rs#L362-L375
fn language_to_extension(language: &str) -> &'static str {
  match language {
    "tagged-css" | "styled-jsx" => "scss",
    "tagged-graphql" => "gql", // Not used, oxc will always use format_embedded_doc
    "tagged-html" => "html",
    "tagged-markdown" => "md",
    "angular-template" => "component.html", // Extension used by markup_fmt
    "angular-styles" => "scss",
    _ => "txt",
  }
}

struct OxcPluginHandler;

impl SyncPluginHandler<Configuration> for OxcPluginHandler {
  fn resolve_config(
    &mut self,
    config: ConfigKeyMap,
    global_config: &GlobalConfiguration,
  ) -> PluginResolveConfigurationResult<Configuration> {
    let result = resolve_config(config, global_config);
    let file_extensions = vec![
      "ts".to_string(),
      "tsx".to_string(),
      "cts".to_string(),
      "mts".to_string(),
      "js".to_string(),
      "jsx".to_string(),
      "cjs".to_string(),
      "mjs".to_string(),
    ];
    PluginResolveConfigurationResult {
      config: result.config,
      diagnostics: result.diagnostics,
      file_matching: FileMatchingInfo {
        file_extensions,
        file_names: vec![],
      },
    }
  }

  fn check_config_updates(&self, _message: CheckConfigUpdatesMessage) -> anyhow::Result<Vec<ConfigChange>> {
    Ok(Vec::new())
  }

  fn plugin_info(&mut self) -> PluginInfo {
    let version = env!("CARGO_PKG_VERSION").to_string();
    PluginInfo {
      name: env!("CARGO_PKG_NAME").to_string(),
      version: version.clone(),
      config_key: "oxc".to_string(),
      help_url: "https://dprint.dev/plugins/oxc".to_string(),
      config_schema_url: format!(
        "https://plugins.dprint.dev/dprint/dprint-plugin-oxc/{}/schema.json",
        version
      ),
      update_url: Some("https://plugins.dprint.dev/dprint/dprint-plugin-oxc/latest.json".to_string()),
    }
  }

  fn license_text(&mut self) -> String {
    std::str::from_utf8(include_bytes!("../LICENSE")).unwrap().into()
  }

  fn format(
    &mut self,
    request: SyncFormatRequest<Configuration>,
    format_with_host: impl FnMut(SyncHostFormatRequest) -> FormatResult,
  ) -> FormatResult {
    if request.range.is_some() {
      return Ok(None); // not implemented
    }

    let override_config = ConfigKeyMap::new();
    let format_callback = WasmCallback::new(format_with_host);

    // SAFETY: The embedded_formatter closure is only used synchronously within format_text
    // and does not escape. The 'static bound on EmbeddedFormatterCallback is overly
    // restrictive for this use case. We know format_with_host lives for the duration
    // of this function call, which is longer than the format_text call.
    let embedded_formatter: EmbeddedFormatterCallback = unsafe {
      std::mem::transmute::<
        Arc<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>,
        Arc<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync + 'static>,
      >(Arc::new(move |language: &str, text: &str| {
        let file_path = format!("/tmp/embedded.{}", language_to_extension(language));
        let request = SyncHostFormatRequest {
          file_path: Path::new(&file_path),
          file_bytes: text.as_bytes(),
          range: None,
          override_config: &override_config,
        };

        match format_callback.call(request) {
          Ok(Some(bytes)) => String::from_utf8(bytes).map_err(|e| e.to_string()),
          Ok(None) => Ok(text.to_string()),
          Err(e) => Err(format!("Failed to format embedded {}: {:#}", language, e)),
        }
      }))
    };

    let text = String::from_utf8_lossy(&request.file_bytes);
    let maybe_text = super::format_text(
      request.file_path,
      &text,
      request.config,
      Some(ExternalCallbacks::new().with_embedded_formatter(Some(embedded_formatter))),
    )?;
    Ok(maybe_text.map(|t| t.into_bytes()))
  }
}

generate_plugin_code!(OxcPluginHandler, OxcPluginHandler);
