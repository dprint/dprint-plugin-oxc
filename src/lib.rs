pub mod configuration;
mod embed;
mod file_kind;
mod format_text;
mod options;

pub use file_kind::file_extensions;
pub use file_kind::file_names;

pub use format_text::format_text;

#[cfg(feature = "wasm")]
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod wasm_plugin;

#[cfg(feature = "wasm")]
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub use wasm_plugin::*;
