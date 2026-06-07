mod common;

use cassette_host::{CassetteHost, ExportKind, HostError, format_signature, kind_label};
use common::*;
use wasmtime::Engine;

#[test]
fn format_signature_i32_i32_to_i32() {
    let engine = Engine::default();
    let ft = wasmtime::FuncType::new(&engine, [wasmtime::ValType::I32, wasmtime::ValType::I32], [wasmtime::ValType::I32]);
    let sig = format_signature(&ft);
    assert_eq!(sig, "(i32, i32) -> (i32)");
}

#[test]
fn format_signature_no_params_no_results() {
    let engine = Engine::default();
    let ft = wasmtime::FuncType::new(&engine, [], []);
    let sig = format_signature(&ft);
    assert_eq!(sig, "() -> ()");
}

#[test]
fn format_signature_multiple_results() {
    let engine = Engine::default();
    let ft = wasmtime::FuncType::new(&engine, [wasmtime::ValType::I32], [wasmtime::ValType::I32, wasmtime::ValType::I64]);
    let sig = format_signature(&ft);
    assert_eq!(sig, "(i32) -> (i32, i64)");
}

#[test]
fn kind_label_function() {
    let engine = Engine::default();
    let ft = wasmtime::FuncType::new(&engine, [], []);
    let et = wasmtime::ExternType::Func(ft);
    assert_eq!(kind_label(&et), "function");
}

#[test]
fn kind_label_memory() {
    let mem_ty = wasmtime::MemoryType::new(1, None);
    let et = wasmtime::ExternType::Memory(mem_ty);
    assert_eq!(kind_label(&et), "memory");
}

#[test]
fn kind_label_global() {
    let global_ty = wasmtime::GlobalType::new(wasmtime::ValType::I32, wasmtime::Mutability::Const);
    let et = wasmtime::ExternType::Global(global_ty);
    assert_eq!(kind_label(&et), "global");
}

#[test]
fn kind_label_table() {
    let table_ty = wasmtime::TableType::new(wasmtime::RefType::FUNCREF, 1, None);
    let et = wasmtime::ExternType::Table(table_ty);
    assert_eq!(kind_label(&et), "table");
}

#[test]
fn host_error_display_invalid_wasm() {
    let err = HostError::InvalidWasm { message: "bad bytes".to_string() };
    let msg = format!("{err}");
    assert!(msg.contains("invalid WASM"));
    assert!(msg.contains("bad bytes"));
}

#[test]
fn host_error_display_export_not_found() {
    let err = HostError::ExportNotFound {
        name: "add".to_string(),
        available: vec!["greet".to_string(), "memory".to_string()],
    };
    let msg = format!("{err}");
    assert!(msg.contains("add"));
    assert!(msg.contains("greet"));
    assert!(msg.contains("memory"));
}

#[test]
fn host_error_display_signature_mismatch() {
    let err = HostError::SignatureMismatch {
        name: "add".to_string(),
        expected: "(i32, i32) -> (i32)".to_string(),
        actual: "(i32) -> (i32)".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("add"));
    assert!(msg.contains("expected"));
    assert!(msg.contains("i32"));
}

#[test]
fn host_error_display_memory_not_found() {
    assert_eq!(format!("{}", HostError::MemoryNotFound), "memory export not found");
}

#[test]
fn host_error_display_heap_base_not_found() {
    assert_eq!(format!("{}", HostError::HeapBaseNotFound), "__heap_base global not found");
}

#[test]
fn host_error_display_trap() {
    let err = HostError::Trap { message: "unreachable".to_string() };
    let msg = format!("{err}");
    assert!(msg.contains("WASM trap"));
    assert!(msg.contains("unreachable"));
}

#[test]
fn host_error_display_no_plugin_loaded() {
    assert_eq!(format!("{}", HostError::NoPluginLoaded), "no plugin loaded");
}

#[test]
fn host_error_display_memory_out_of_bounds() {
    let err = HostError::MemoryOutOfBounds {
        offset: 65536,
        len: 10,
        memory_size: 65536,
    };
    let msg = format!("{err}");
    assert!(msg.contains("65536"));
    assert!(msg.contains("65546"));
}

#[test]
fn host_error_display_buffer_overflow() {
    let err = HostError::BufferOverflow {
        written: 512,
        capacity: 256,
    };
    let msg = format!("{err}");
    assert!(msg.contains("512"));
    assert!(msg.contains("256"));
}

#[test]
fn host_error_display_utf8_error() {
    let err = HostError::Utf8Error { detail: "invalid byte".to_string() };
    assert!(format!("{err}").contains("UTF-8"));
}

#[test]
fn host_error_display_io() {
    let err = HostError::Io {
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"),
        path: std::path::PathBuf::from("/tmp/test.wasm"),
    };
    let msg = format!("{err}");
    assert!(msg.contains("/tmp/test.wasm"));
}

#[test]
fn host_error_is_methods() {
    assert!(HostError::InvalidWasm { message: "x".into() }.is_invalid_wasm());
    assert!(HostError::ExportNotFound { name: "x".into(), available: vec![] }.is_export_not_found());
    assert!(HostError::SignatureMismatch { name: "x".into(), expected: "a".into(), actual: "b".into() }.is_signature_mismatch());
    assert!(HostError::Trap { message: "x".into() }.is_trap());
    assert!(HostError::MemoryNotFound.is_memory_not_found());
    assert!(HostError::HeapBaseNotFound.is_heap_base_not_found());
}

#[test]
fn host_error_is_methods_negative() {
    let err = HostError::InvalidWasm { message: "x".into() };
    assert!(!err.is_trap());
    assert!(!err.is_export_not_found());
    assert!(!err.is_signature_mismatch());
    assert!(!err.is_memory_not_found());
    assert!(!err.is_heap_base_not_found());
}

#[test]
fn export_kind_display() {
    assert_eq!(format!("{}", ExportKind::Function), "function");
    assert_eq!(format!("{}", ExportKind::Memory), "memory");
    assert_eq!(format!("{}", ExportKind::Table), "table");
    assert_eq!(format!("{}", ExportKind::Global), "global");
    assert_eq!(format!("{}", ExportKind::Tag), "tag");
}

#[test]
fn cassette_host_new_succeeds() {
    assert!(CassetteHost::new().is_ok());
}

#[test]
fn cassette_host_default_succeeds() {
    let _host = CassetteHost::default();
}

#[test]
fn no_plugin_loaded_returns_error_on_call() {
    let mut host = CassetteHost::new().unwrap();
    let result = host.call_i32_i32_to_i32("add", 1, 2);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), HostError::NoPluginLoaded));
}

#[test]
fn no_plugin_loaded_returns_empty_exports() {
    let mut host = CassetteHost::new().unwrap();
    assert!(host.list_exports().is_empty());
}

#[test]
fn no_plugin_loaded_has_export_returns_false() {
    let mut host = CassetteHost::new().unwrap();
    assert!(!host.has_export("add"));
}

#[test]
fn load_invalid_bytes_fails() {
    let mut host = CassetteHost::new().unwrap();
    let result = host.load_plugin_from_bytes(&completely_invalid_bytes());
    assert!(result.is_err());
    assert!(result.unwrap_err().is_invalid_wasm());
}

#[test]
fn load_corrupted_wasm_header_fails() {
    let mut host = CassetteHost::new().unwrap();
    let result = host.load_plugin_from_bytes(&corrupted_wasm());
    assert!(result.is_err());
}

#[test]
fn load_module_with_unresolved_imports_fails() {
    let mut host = CassetteHost::new().unwrap();
    let result = host.load_plugin_from_bytes(&wasm_with_imports());
    assert!(result.is_err());
    match result.unwrap_err() {
        HostError::InstantiationFailed { message } => {
            assert!(message.contains("env") || message.contains("import") || message.contains("linker"));
        }
        other => panic!("expected InstantiationFailed, got: {other}"),
    }
}

#[test]
fn load_empty_module_succeeds() {
    let mut host = CassetteHost::new().unwrap();
    assert!(host.load_plugin_from_bytes(&wasm_empty_module()).is_ok());
}

#[test]
fn load_nonexistent_file_fails() {
    let mut host = CassetteHost::new().unwrap();
    let result = host.load_plugin_from_file(std::path::Path::new("/nonexistent/path/test.wasm"));
    assert!(result.is_err());
    match result.unwrap_err() {
        HostError::Io { path, .. } => {
            assert!(path.to_string_lossy().contains("nonexistent"));
        }
        other => panic!("expected Io error, got: {other}"),
    }
}