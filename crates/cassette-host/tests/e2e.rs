mod common;

use cassette_host::CassetteHost;
use common::*;

#[test]
fn end_to_end_add_workflow() {
    let path = match hello_plugin_path() {
        Some(p) => p,
        None => {
            eprintln!("skipping e2e test: hello_plugin.wasm not found (run: cargo build -p hello-plugin --target wasm32-unknown-unknown --release)");
            return;
        }
    };

    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_file(&path).unwrap();

    assert_eq!(host.call_add(2, 3).unwrap(), 5);
    assert_eq!(host.call_add(0, 0).unwrap(), 0);
    assert_eq!(host.call_add(-5, 10).unwrap(), 5);
    assert_eq!(host.call_add(100, 200).unwrap(), 300);
}

#[test]
fn end_to_end_greet_name_workflow() {
    let path = match hello_plugin_path() {
        Some(p) => p,
        None => {
            eprintln!("skipping e2e test: hello_plugin.wasm not found");
            return;
        }
    };

    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_file(&path).unwrap();

    assert_eq!(host.call_greet_name("World").unwrap(), "Hello, World!");
    assert_eq!(host.call_greet_name("Rust").unwrap(), "Hello, Rust!");
    assert_eq!(host.call_greet_name("Cassette").unwrap(), "Hello, Cassette!");
}

#[test]
fn end_to_end_greet_returns_pointer() {
    let path = match hello_plugin_path() {
        Some(p) => p,
        None => {
            eprintln!("skipping e2e test: hello_plugin.wasm not found");
            return;
        }
    };

    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_file(&path).unwrap();

    let result = host.call_greet().unwrap();
    assert!(result.contains("Hello"));
    assert!(result.contains("Cassette"));
}

#[test]
fn end_to_end_export_discovery() {
    let path = match hello_plugin_path() {
        Some(p) => p,
        None => {
            eprintln!("skipping e2e test: hello_plugin.wasm not found");
            return;
        }
    };

    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_file(&path).unwrap();

    let names = host.export_names();
    assert!(names.contains(&"add".to_string()), "expected 'add' export");
    assert!(names.contains(&"greet_name".to_string()), "expected 'greet_name' export");
    assert!(names.contains(&"greet".to_string()), "expected 'greet' export");
    assert!(names.contains(&"memory".to_string()), "expected 'memory' export");
}

#[test]
fn end_to_end_memory_operations() {
    let path = match hello_plugin_path() {
        Some(p) => p,
        None => {
            eprintln!("skipping e2e test: hello_plugin.wasm not found");
            return;
        }
    };

    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_file(&path).unwrap();

    let heap_base = host.heap_base().unwrap();
    assert!(heap_base > 0);
    host.write_memory(heap_base, b"test").unwrap();
    let read_back = host.read_memory(heap_base, 4).unwrap();
    assert_eq!(&read_back, b"test");
    assert!(host.memory_size().unwrap() > 0);
}

#[test]
fn end_to_end_add_with_boundary_values() {
    let path = match hello_plugin_path() {
        Some(p) => p,
        None => {
            eprintln!("skipping e2e test: hello_plugin.wasm not found");
            return;
        }
    };

    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_file(&path).unwrap();

    assert_eq!(host.call_add(0, 0).unwrap(), 0);
    assert_eq!(host.call_add(1, -1).unwrap(), 0);
    assert_eq!(host.call_add(i32::MAX, 0).unwrap(), i32::MAX);
    assert_eq!(host.call_add(i32::MIN, 0).unwrap(), i32::MIN);
}

#[test]
fn end_to_end_greet_name_with_empty_string() {
    let path = match hello_plugin_path() {
        Some(p) => p,
        None => {
            eprintln!("skipping e2e test: hello_plugin.wasm not found");
            return;
        }
    };

    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_file(&path).unwrap();
    assert_eq!(host.call_greet_name("").unwrap(), "Hello, !");
}

#[test]
fn end_to_end_greet_name_with_long_name() {
    let path = match hello_plugin_path() {
        Some(p) => p,
        None => {
            eprintln!("skipping e2e test: hello_plugin.wasm not found");
            return;
        }
    };

    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_file(&path).unwrap();

    let long_name = "A".repeat(200);
    let result = host.call_greet_name(&long_name).unwrap();
    assert!(result.starts_with("Hello, "));
    assert!(result.ends_with("!"));
}

#[test]
fn end_to_end_multiple_sequential_calls() {
    let path = match hello_plugin_path() {
        Some(p) => p,
        None => {
            eprintln!("skipping e2e test: hello_plugin.wasm not found");
            return;
        }
    };

    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_file(&path).unwrap();

    for i in 0..10i32 {
        assert_eq!(host.call_add(i, i + 1).unwrap(), i * 2 + 1);
    }
    for name in ["Alice", "Bob", "Carol"] {
        assert!(host.call_greet_name(name).unwrap().contains(name));
    }
}

#[test]
fn end_to_end_load_from_bytes() {
    let path = match hello_plugin_path() {
        Some(p) => p,
        None => {
            eprintln!("skipping e2e test: hello_plugin.wasm not found");
            return;
        }
    };

    let bytes = std::fs::read(&path).unwrap();
    let mut host = CassetteHost::new().unwrap();
    host.load_plugin_from_bytes(&bytes).unwrap();
    assert_eq!(host.call_add(10, 20).unwrap(), 30);
}