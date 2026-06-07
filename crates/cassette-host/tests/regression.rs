mod common;

use cassette_host::{CassetteHost, HostError};
use common::*;

#[test]
fn regression_i32_add_wraps_on_overflow() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_i32_overflow()).unwrap();
    assert_eq!(host.call_i32_i32_to_i32("add", i32::MAX, 1).unwrap(), i32::MIN);
    assert_eq!(host.call_i32_i32_to_i32("add", i32::MIN, -1).unwrap(), i32::MAX);
}

#[test]
fn regression_trap_produces_host_error_not_panic() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_trap_on_call()).unwrap();
    let result = host.call_i32_i32_to_i32("add", 1, 2);
    assert!(result.is_err());
    match result.unwrap_err() {
        HostError::Trap { message } => assert!(!message.is_empty()),
        other => panic!("expected Trap error, got: {other}"),
    }
}

#[test]
fn regression_missing_export_lists_available() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    let result = host.call_i32_i32_i32_i32_to_i32("greet_name", 0, 0, 0, 0);
    match result.unwrap_err() {
        HostError::ExportNotFound { name, available } => {
            assert_eq!(name, "greet_name");
            assert!(available.contains(&"add".to_string()));
        }
        other => panic!("expected ExportNotFound, got: {other}"),
    }
}

#[test]
fn regression_signature_mismatch_reports_actual_signature() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_wrong_signature()).unwrap();
    match host.call_i32_i32_to_i32("add", 1, 2).unwrap_err() {
        HostError::SignatureMismatch { name, expected, actual } => {
            assert_eq!(name, "add");
            assert!(expected.contains("i32"));
            assert!(actual.contains("f64"));
        }
        other => panic!("expected SignatureMismatch, got: {other}"),
    }
}

#[test]
fn regression_empty_module_has_no_exports() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_empty_module()).unwrap();
    assert!(host.list_exports().is_empty());
    assert!(host.export_names().is_empty());
    assert!(!host.has_export("anything"));
}

#[test]
fn regression_operations_after_failed_load_preserve_state() {
    let mut host = CassetteHost::new().unwrap();
    assert!(host.load_plugin_from_bytes(&completely_invalid_bytes()).is_err());
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    assert_eq!(host.call_i32_i32_to_i32("add", 2, 3).unwrap(), 5);
}

#[test]
fn regression_calling_without_load_gives_clear_error() {
    let mut host = CassetteHost::new().unwrap();
    let results: Vec<Result<(), HostError>> = vec![
        host.call_i32_i32_to_i32("add", 1, 2).map(|_| ()),
        host.write_memory(0, b"test").map(|_| ()),
        host.read_memory(0, 4).map(|_| ()),
        host.heap_base().map(|_| ()),
        host.memory_size().map(|_| ()),
    ];
    for result in results {
        match result {
            Err(HostError::NoPluginLoaded) => {}
            Err(e) => panic!("expected NoPluginLoaded, got: {e}"),
            Ok(()) => panic!("expected error, got success"),
        }
    }
}

#[test]
fn regression_cstring_at_memory_boundary_truncates() {
    let mut host = cassette_host::CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    let mem_size = host.memory_size().unwrap();
    host.write_memory(mem_size - 3, &[0x41, 0x42, 0x43]).unwrap();
    let result = host.read_cstring(mem_size - 3);
    assert!(result.is_err() || result.unwrap() == "ABC");
}

#[test]
fn regression_non_utf8_cstring() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    host.write_memory(1024, &[0xC3, 0x28, 0x00]).unwrap();
    assert!(matches!(host.read_cstring(1024).unwrap_err(), HostError::Utf8Error { .. }));
}

#[test]
fn regression_double_load_replaces_instance() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_minimal_add()).unwrap();
    assert_eq!(host.call_i32_i32_to_i32("add", 3, 4).unwrap(), 7);

    host.load_plugin_from_bytes(&wasm_with_memory()).unwrap();
    assert_eq!(host.call_i32_i32_to_i32("add", 3, 4).unwrap(), 7);
    assert!(host.has_export("greet"));
}

#[test]
fn regression_call_greet_name_with_empty_input() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_with_greet_name()).unwrap();
    let written = host.call_i32_i32_i32_i32_to_i32("greet_name", 1024, 0, 2048, 256).unwrap();
    assert!(written >= 0);
    if written > 0 {
        let output = host.read_memory(2048, written as usize).unwrap();
        assert!(!output.is_empty());
    }
}

#[test]
fn regression_globals_module_signature_check() {
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&wasm_globals_module()).unwrap();
    let sig = host.get_func_signature("increment").unwrap();
    assert_eq!(sig.params.len(), 0);
    assert_eq!(sig.results.len(), 1);
    assert_eq!(sig.format(), "() -> (i32)");
}

#[test]
fn regression_corrupted_wasm_with_valid_header_fails() {
    let mut host = CassetteHost::new().unwrap();
    assert!(host.load_plugin_from_bytes(&corrupted_wasm()).is_err());
}