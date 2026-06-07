mod common;

use cassette_host::{CassetteHost, ExportKind, HostError};
use common::*;

#[test]
fn call_add_basic() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    assert_eq!(host.call_i32_i32_to_i32("add", 2, 3).unwrap(), 5);
}

#[test]
fn call_add_zero() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    assert_eq!(host.call_i32_i32_to_i32("add", 0, 0).unwrap(), 0);
}

#[test]
fn call_add_negative_numbers() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    assert_eq!(host.call_i32_i32_to_i32("add", -5, 3).unwrap(), -2);
}

#[test]
fn call_add_large_numbers() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    assert_eq!(host.call_i32_i32_to_i32("add", 1000000, 2000000).unwrap(), 3000000);
}

#[test]
fn call_add_i32_max_overflow() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    assert_eq!(host.call_i32_i32_to_i32("add", i32::MAX, 1).unwrap(), i32::MIN);
}

#[test]
fn call_add_i32_min_underflow() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    assert_eq!(host.call_i32_i32_to_i32("add", i32::MIN, -1).unwrap(), i32::MAX);
}

#[test]
fn call_missing_export_returns_error() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    let result = host.call_i32_i32_to_i32("nonexistent", 1, 2);
    assert!(result.is_err());
    match result.unwrap_err() {
        HostError::ExportNotFound { name, available } => {
            assert_eq!(name, "nonexistent");
            assert!(available.contains(&"add".to_string()));
        }
        other => panic!("expected ExportNotFound, got: {other}"),
    }
}

#[test]
fn call_wrong_signature_returns_error() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_wrong_signature()).unwrap();
    let result = host.call_i32_i32_to_i32("add", 1, 2);
    assert!(result.is_err());
    match result.unwrap_err() {
        HostError::SignatureMismatch { name, expected, actual } => {
            assert_eq!(name, "add");
            assert!(expected.contains("i32"));
            assert!(actual.contains("f64"));
        }
        other => panic!("expected SignatureMismatch, got: {other}"),
    }
}

#[test]
fn call_wrong_param_count_returns_error() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_wrong_param_count()).unwrap();
    let result = host.call_i32_i32_to_i32("add", 1, 2);
    assert!(result.is_err());
    assert!(result.unwrap_err().is_signature_mismatch());
}

#[test]
fn call_trap_returns_error() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_trap_on_call()).unwrap();
    let result = host.call_i32_i32_to_i32("add", 1, 2);
    assert!(result.is_err());
    assert!(result.unwrap_err().is_trap());
}

#[test]
fn memory_write_and_read_roundtrip() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    let data = b"hello world";
    host.write_memory(1024, data).unwrap();
    let read_back = host.read_memory(1024, data.len()).unwrap();
    assert_eq!(read_back, data);
}

#[test]
fn memory_write_preserves_adjacent_data() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    host.write_memory(1024, b"aaaa").unwrap();
    host.write_memory(1028, b"bbbb").unwrap();
    let all = host.read_memory(1024, 8).unwrap();
    assert_eq!(&all, b"aaaabbbb");
}

#[test]
fn memory_write_zero_length() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    assert!(host.write_memory(1024, b"").is_ok());
}

#[test]
fn memory_read_zero_length() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    let result = host.read_memory(1024, 0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn memory_operations_without_plugin_fails() {
    let mut host = CassetteHost::new().unwrap();
    let result = host.write_memory(0, b"test");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), HostError::NoPluginLoaded));
}

#[test]
fn memory_read_without_plugin_fails() {
    let mut host = CassetteHost::new().unwrap();
    let result = host.read_memory(0, 4);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), HostError::NoPluginLoaded));
}

#[test]
fn memory_write_without_memory_export_fails() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_missing_memory()).unwrap();
    let result = host.write_memory(0, b"test");
    assert!(result.is_err());
    assert!(result.unwrap_err().is_memory_not_found());
}

#[test]
fn memory_read_without_memory_export_fails() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_missing_memory()).unwrap();
    let result = host.read_memory(0, 4);
    assert!(result.is_err());
    assert!(result.unwrap_err().is_memory_not_found());
}

#[test]
fn heap_base_returns_correct_value() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    assert_eq!(host.heap_base().unwrap(), 1024);
}

#[test]
fn heap_base_without_plugin_fails() {
    let mut host = CassetteHost::new().unwrap();
    assert!(matches!(host.heap_base().unwrap_err(), HostError::NoPluginLoaded));
}

#[test]
fn heap_base_without_global_fails() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_missing_heap_base()).unwrap();
    assert!(host.heap_base().unwrap_err().is_heap_base_not_found());
}

#[test]
fn memory_size_returns_correct_value() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    assert_eq!(host.memory_size().unwrap(), 65536);
}

#[test]
fn read_cstring_basic() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    host.write_memory(1024, b"hello\0").unwrap();
    assert_eq!(host.read_cstring(1024).unwrap(), "hello");
}

#[test]
fn read_cstring_empty() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    host.write_memory(1024, b"\0").unwrap();
    assert_eq!(host.read_cstring(1024).unwrap(), "");
}

#[test]
fn read_cstring_with_surrounding_data() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    host.write_memory(1024, b"XXXabc\0YYY\0").unwrap();
    assert_eq!(host.read_cstring(1024 + 3).unwrap(), "abc");
}

#[test]
fn read_cstring_reads_until_wasm_null_padding() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    host.write_memory(1024, b"hello").unwrap();
    let result = host.read_cstring(1024).unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn read_cstring_at_end_of_memory_fails() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    let mem_size = host.memory_size().unwrap();
    host.write_memory(mem_size - 4, &[0xAA, 0xBB, 0xCC, 0xDD]).unwrap();
    let result = host.read_cstring(mem_size - 4);
    assert!(result.is_err());
}

#[test]
fn read_cstring_non_utf8_fails() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    host.write_memory(1024, &[0xFF, 0xFE, 0x00]).unwrap();
    let result = host.read_cstring(1024);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), HostError::Utf8Error { .. }));
}

#[test]
fn has_export_returns_true_for_existing() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    assert!(host.has_export("add"));
}

#[test]
fn has_export_returns_false_for_missing() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    assert!(!host.has_export("nonexistent"));
}

#[test]
fn get_func_signature_returns_correct_type() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    let sig = host.get_func_signature("add").unwrap();
    assert_eq!(sig.params.len(), 2);
    assert_eq!(sig.results.len(), 1);
}

#[test]
fn get_func_signature_returns_none_for_nonexistent() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    assert!(host.get_func_signature("nonexistent").is_none());
}

#[test]
fn get_func_signature_returns_none_for_non_function() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    assert!(host.get_func_signature("memory").is_none());
}

#[test]
fn export_names_returns_all_names() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    let names = host.export_names();
    assert!(names.contains(&"add".to_string()));
    assert!(names.contains(&"memory".to_string()));
    assert!(names.contains(&"__heap_base".to_string()));
    assert!(names.contains(&"greet".to_string()));
}

#[test]
fn reloading_plugin_replaces_previous() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    assert!(host.has_export("add"));
    assert!(!host.has_export("greet"));

    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    assert!(host.has_export("greet"));
}

#[test]
fn list_exports_minimal_add() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    let exports = host.list_exports();
    assert_eq!(exports.len(), 1);
    assert_eq!(exports[0].name, "add");
    assert_eq!(exports[0].kind, ExportKind::Function);
    assert!(exports[0].signature.is_some());
}

#[test]
fn load_minimal_add_succeeds() {
    let mut host = CassetteHost::new().unwrap();
    assert!(host.load_plugin_from_bytes(&wasm_minimal_add()).is_ok());
}