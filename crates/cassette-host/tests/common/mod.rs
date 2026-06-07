#![allow(dead_code)]

use cassette_host::{CassetteHost, HostError};

pub fn wasm_minimal_add() -> Vec<u8> {
    wat::parse_str(
        r#"(module
  (func (export "add") (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add)
)"#,
    )
    .expect("failed to parse minimal_add WAT")
}

pub fn wasm_with_memory() -> Vec<u8> {
    wat::parse_str(
        r#"(module
  (memory (export "memory") 1)
  (global (export "__heap_base") i32 (i32.const 1024))
  (func (export "add") (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add)
  (func (export "greet") (result i32)
    i32.const 1024)
)"#,
    )
    .expect("failed to parse with_memory WAT")
}

pub fn wasm_with_greet_name() -> Vec<u8> {
    wat::parse_str(
        r#"(module
  (memory (export "memory") 1)
  (global (export "__heap_base") i32 (i32.const 1024))

  (func (export "add") (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add)

  (func (export "greet") (result i32)
    i32.const 1024)

  (func (export "greet_name") (param $name_ptr i32) (param $name_len i32) (param $out_ptr i32) (param $out_max i32) (result i32)
    (local $i i32)
    (local $written i32)

    i32.const 0
    local.set $written

    ;; Write 'H'
    local.get $out_ptr
    local.get $written
    i32.add
    i32.const 72
    i32.store8
    local.get $written
    i32.const 1
    i32.add
    local.set $written

    ;; Write 'i'
    local.get $out_ptr
    local.get $written
    i32.add
    i32.const 105
    i32.store8
    local.get $written
    i32.const 1
    i32.add
    local.set $written

    ;; Copy name bytes
    i32.const 0
    local.set $i
    block $break_copy
    loop $copy_loop
      local.get $i
      local.get $name_len
      i32.ge_u
      br_if $break_copy

      ;; Store: address = out_ptr + written, value = load8(name_ptr + i)
      local.get $out_ptr
      local.get $written
      i32.add
      local.get $name_ptr
      local.get $i
      i32.add
      i32.load8_u
      i32.store8

      local.get $written
      i32.const 1
      i32.add
      local.set $written
      local.get $i
      i32.const 1
      i32.add
      local.set $i
      br $copy_loop
    end
    end

    local.get $written
  )
)"#,
    )
    .expect("failed to parse greet_name WAT")
}

pub fn wasm_trap_on_call() -> Vec<u8> {
    wat::parse_str(
        r#"(module
  (func (export "add") (param i32 i32) (result i32)
    unreachable)
)"#,
    )
    .expect("failed to parse trap_on_call WAT")
}

pub fn wasm_wrong_signature() -> Vec<u8> {
    wat::parse_str(
        r#"(module
  (func (export "add") (param i32 i32) (result f64)
    f64.const 1.0)
)"#,
    )
    .expect("failed to parse wrong_signature WAT")
}

pub fn wasm_wrong_param_count() -> Vec<u8> {
    wat::parse_str(
        r#"(module
  (func (export "add") (param i32) (result i32)
    local.get 0)
)"#,
    )
    .expect("failed to parse wrong_param_count WAT")
}

pub fn wasm_empty_module() -> Vec<u8> {
    wat::parse_str("(module)").expect("failed to parse empty WAT")
}

pub fn wasm_missing_memory() -> Vec<u8> {
    wat::parse_str(
        r#"(module
  (func (export "add") (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add)
)"#,
    )
    .expect("failed to parse missing_memory WAT")
}

pub fn wasm_missing_heap_base() -> Vec<u8> {
    wat::parse_str(
        r#"(module
  (memory (export "memory") 1)
  (func (export "add") (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add)
)"#,
    )
    .expect("failed to parse missing_heap_base WAT")
}

pub fn wasm_with_imports() -> Vec<u8> {
    wat::parse_str(
        r#"(module
  (import "env" "log" (func $log (param i32)))
  (func (export "do_thing") (param i32)
    local.get 0
    call $log)
)"#,
    )
    .expect("failed to parse with_imports WAT")
}

pub fn wasm_globals_module() -> Vec<u8> {
    wat::parse_str(
        r#"(module
  (global (export "counter") (mut i32) (i32.const 0))
  (func (export "increment") (result i32)
    global.get 0
    i32.const 1
    i32.add
    global.set 0
    global.get 0)
)"#,
    )
    .expect("failed to parse globals WAT")
}

pub fn wasm_i32_overflow() -> Vec<u8> {
    wat::parse_str(
        r#"(module
  (func (export "add") (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add)
)"#,
    )
    .expect("failed to parse i32_overflow WAT")
}

pub fn corrupted_wasm() -> Vec<u8> {
    vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0xFF, 0xFF]
}

pub fn completely_invalid_bytes() -> Vec<u8> {
    vec![0xDE, 0xAD, 0xBE, 0xEF]
}

pub fn hello_plugin_path() -> Option<std::path::PathBuf> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target/wasm32-unknown-unknown/release/hello_plugin.wasm");
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

pub fn load_hello_plugin() -> Option<CassetteHost> {
    let path = hello_plugin_path()?;
    let mut host = CassetteHost::new().ok()?;
    host.load_plugin_from_file(&path).ok()?;
    Some(host)
}

pub fn assert_host_error(result: &Result<(), HostError>, predicate: impl Fn(&HostError) -> bool) {
    match result {
        Err(e) if predicate(e) => {}
        Err(e) => panic!("host error did not match predicate: {e}"),
        Ok(()) => panic!("expected error, got success"),
    }
}