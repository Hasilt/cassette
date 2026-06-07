mod error;
mod host;
mod signature;

pub use error::HostError;
pub use host::CassetteHost;
pub use signature::{ExportInfo, ExportKind, FuncSignature, format_signature, kind_label};

pub fn load_plugin(_path: &str) {
    unimplemented!("plugin loading not yet implemented")
}