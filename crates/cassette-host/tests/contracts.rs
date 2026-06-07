mod common;

use cassette_host::{CassetteHost, ExportKind};
use common::*;

#[test]
fn plugin_exports_add_function() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    let sig = host.get_func_signature("add").unwrap();
    assert_eq!(sig.params.len(), 2);
    assert_eq!(sig.results.len(), 1);
    assert_eq!(sig.format(), "(i32, i32) -> (i32)");
}

#[test]
fn plugin_add_signature_is_i32_i32_to_i32() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_greet_name()).unwrap();
    let sig = host.get_func_signature("add").unwrap();
    assert_eq!(sig.format(), "(i32, i32) -> (i32)");
}

#[test]
fn plugin_greet_name_signature_is_four_i32_to_i32() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_greet_name()).unwrap();
    let sig = host.get_func_signature("greet_name").unwrap();
    assert_eq!(sig.params.len(), 4);
    assert_eq!(sig.results.len(), 1);
    assert_eq!(sig.format(), "(i32, i32, i32, i32) -> (i32)");
}

#[test]
fn plugin_memory_export_exists() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    let exports = host.list_exports();
    let memory_exports: Vec<_> = exports
        .iter()
        .filter(|e| e.kind == ExportKind::Memory)
        .collect();
    assert_eq!(memory_exports.len(), 1);
    assert_eq!(memory_exports[0].name, "memory");
}

#[test]
fn plugin_heap_base_exists_and_is_global() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    let exports = host.list_exports();
    let heap_base: Vec<_> = exports
        .iter()
        .filter(|e| e.name == "__heap_base")
        .collect();
    assert_eq!(heap_base.len(), 1);
    assert_eq!(heap_base[0].kind, ExportKind::Global);
}

#[test]
fn plugin_heap_base_value_is_positive() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    let heap_base = host.heap_base().unwrap();
    assert!(heap_base > 0);
}

#[test]
fn contract_add_is_deterministic() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    let r1 = host.call_i32_i32_to_i32("add", 7, 13).unwrap();
    let r2 = host.call_i32_i32_to_i32("add", 7, 13).unwrap();
    assert_eq!(r1, r2);
    assert_eq!(r1, 20);
}

#[test]
fn contract_memory_is_shared_across_calls() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    host.write_memory(1024, b"test data").unwrap();
    let _ = host.call_void_to_i32("greet").unwrap();
    let read_back = host.read_memory(1024, 9).unwrap();
    assert_eq!(&read_back, b"test data");
}

#[test]
fn contract_plugin_memory_persistence() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    host.write_memory(1024, b"ABC").unwrap();
    let first_read = host.read_memory(1024, 3).unwrap();
    host.write_memory(2048, b"XYZ").unwrap();
    let second_read = host.read_memory(1024, 3).unwrap();
    assert_eq!(first_read, second_read);
    assert_eq!(&first_read[..], b"ABC");
}

#[test]
fn contract_export_kind_classification() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    let exports = host.list_exports();
    let kinds: std::collections::HashMap<String, ExportKind> = exports
        .into_iter()
        .map(|e| (e.name, e.kind))
        .collect();
    assert_eq!(kinds.get("add"), Some(&ExportKind::Function));
    assert_eq!(kinds.get("greet"), Some(&ExportKind::Function));
    assert_eq!(kinds.get("memory"), Some(&ExportKind::Memory));
    assert_eq!(kinds.get("__heap_base"), Some(&ExportKind::Global));
}

#[test]
fn contract_add_produces_correct_boundary_values() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    assert_eq!(host.call_i32_i32_to_i32("add", 0, 0).unwrap(), 0);
    assert_eq!(host.call_i32_i32_to_i32("add", i32::MAX, 0).unwrap(), i32::MAX);
    assert_eq!(host.call_i32_i32_to_i32("add", i32::MIN, 0).unwrap(), i32::MIN);
    assert_eq!(host.call_i32_i32_to_i32("add", -1, 1).unwrap(), 0);
}

#[test]
fn contract_greet_name_writes_to_output_buffer() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_greet_name()).unwrap();
    host.write_memory(1024, b"abcde").unwrap();
    host.write_memory(2048, &[0u8; 256]).unwrap();
    let written = host.call_i32_i32_i32_i32_to_i32("greet_name", 1024, 5, 2048, 256).unwrap();
    assert!(written > 0);
    let output = host.read_memory(2048, written as usize).unwrap();
    let result = String::from_utf8(output).unwrap();
    assert!(result.starts_with("Hi"));
}

#[test]
fn contract_calling_wrong_signature_function_returns_error() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    let result = host.call_i32_i32_to_i32("greet", 0, 0);
    assert!(result.is_err());
}