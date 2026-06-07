use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum HostError {
    InvalidWasm { message: String },
    InstantiationFailed { message: String },
    Io { source: std::io::Error, path: PathBuf },
    ExportNotFound { name: String, available: Vec<String> },
    SignatureMismatch {
        name: String,
        expected: String,
        actual: String,
    },
    MemoryNotFound,
    HeapBaseNotFound,
    MemoryOutOfBounds {
        offset: usize,
        len: usize,
        memory_size: usize,
    },
    BufferOverflow {
        written: usize,
        capacity: usize,
    },
    Utf8Error { detail: String },
    NoNullTerminator { offset: usize },
    Trap { message: String },
    NoPluginLoaded,
    Other(anyhow::Error),
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HostError::InvalidWasm { message } => write!(f, "invalid WASM: {message}"),
            HostError::InstantiationFailed { message } => {
                write!(f, "WASM instantiation failed: {message}")
            }
            HostError::Io { source, path } => {
                write!(f, "IO error reading '{}': {source}", path.display())
            }
            HostError::ExportNotFound { name, available } => write!(
                f,
                "export '{name}' not found; available: {}",
                available.join(", ")
            ),
            HostError::SignatureMismatch {
                name,
                expected,
                actual,
            } => write!(
                f,
                "signature mismatch for '{name}': expected ({expected}), got ({actual})"
            ),
            HostError::MemoryNotFound => write!(f, "memory export not found"),
            HostError::HeapBaseNotFound => write!(f, "__heap_base global not found"),
            HostError::MemoryOutOfBounds {
                offset,
                len,
                memory_size,
            } => write!(
                f,
                "memory access out of bounds: [{offset}..{}] > {memory_size}",
                offset + len
            ),
            HostError::BufferOverflow { written, capacity } => {
                write!(f, "buffer overflow: {written} bytes into capacity {capacity}")
            }
            HostError::Utf8Error { detail } => write!(f, "UTF-8 error: {detail}"),
            HostError::NoNullTerminator { offset } => {
                write!(f, "no null terminator found starting at offset {offset}")
            }
            HostError::Trap { message } => write!(f, "WASM trap: {message}"),
            HostError::NoPluginLoaded => write!(f, "no plugin loaded"),
            HostError::Other(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for HostError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HostError::Io { source, .. } => Some(source),
            HostError::Other(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

impl HostError {
    pub fn is_invalid_wasm(&self) -> bool {
        matches!(self, HostError::InvalidWasm { .. })
    }

    pub fn is_export_not_found(&self) -> bool {
        matches!(self, HostError::ExportNotFound { .. })
    }

    pub fn is_signature_mismatch(&self) -> bool {
        matches!(self, HostError::SignatureMismatch { .. })
    }

    pub fn is_trap(&self) -> bool {
        matches!(self, HostError::Trap { .. })
    }

    pub fn is_memory_not_found(&self) -> bool {
        matches!(self, HostError::MemoryNotFound)
    }

    pub fn is_heap_base_not_found(&self) -> bool {
        matches!(self, HostError::HeapBaseNotFound)
    }
}