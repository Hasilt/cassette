# cassetee

A small WebAssembly plugin host and a demo plugin, written in Rust.

The host (`cassette-host`) uses wasmtime to load WASM modules, inspect their
exports, call typed functions, and read/write shared memory. The demo plugin
(`hello-plugin`) is a `#![no_std]` crate that compiles to
`wasm32-unknown-unknown` and exports a few functions to show how the host and
plugin talk to each other.

## Project structure

```
crates/
  cassette-host/   the WASM host runtime
  hello-plugin/     a minimal WASM plugin for testing
```

## Building

Build the host and the plugin together:

```
cargo build
```

The plugin needs to be compiled to a WASM target for the host to load it:

```
cargo build -p hello-plugin --target wasm32-unknown-unknown --release
```

If you just want the host library without running the binary, `cargo build`
is enough. The end-to-end tests need the WASM artifact, though.

## Testing

Unit and integration tests can run without the WASM plugin:

```
cargo test -p cassette-host
```

The end-to-end tests in `tests/e2e.rs` need the compiled plugin. They will
skip automatically if the WASM file is not present. To run the full suite:

```
cargo build -p hello-plugin --target wasm32-unknown-unknown --release
cargo test -p cassette-host
```

## How it works

The host creates a `CassetteHost` instance, loads a WASM module from a file
or raw bytes, and then calls functions on it. Here is roughly what `main.rs`
does:

```rust
let mut host = CassetteHost::new()?;

host.load_plugin_from_file(Path::new("target/wasm32-unknown-unknown/release/hello_plugin.wasm"))?;

let result = host.call_add(2, 3)?;
println!("add(2, 3) = {result}");

let greeting = host.call_greet_name("World")?;
println!("greet_name(\"World\") = \"{greeting}\"");
```

The host supports several calling conventions depending on the function
signature:

- `call_void_to_i32` for functions with no parameters returning i32
- `call_i32_to_i32` for single-argument i32 functions
- `call_i32_i32_to_i32` for two-argument i32 functions
- `call_i32_i32_i32_i32_to_i32` for four-argument i32 functions

For passing strings between host and plugin, the host writes input bytes
into WASM memory at a known offset, calls the function with pointer and length
arguments, and reads the result back from shared memory. The `call_greet_name`
convenience method handles this protocol.

You can also inspect what a plugin exports:

```rust
let exports = host.list_exports();
for export in &exports {
    println!("{}: {:?}", export.name, export.kind);
}
```

And interact with memory directly:

```rust
host.write_memory(offset, b"hello")?;
let bytes = host.read_memory(offset, 5)?;
let text = host.read_cstring(offset)?;
```

## Error handling

The host returns `HostError` for everything that can go wrong: invalid WASM
bytes, missing exports, signature mismatches, out-of-bounds memory access,
traps, UTF-8 decoding failures, and so on. Each variant carries enough detail
to figure out what happened, and there are convenience methods like
`is_invalid_wasm()`, `is_trap()`, and `is_signature_mismatch()` for matching
on specific cases.

## The demo plugin

`hello-plugin` is a `#![no_std]` crate that exports three functions:

- `greet() -> *const u8` - returns a pointer to a static string
- `add(a: i32, b: i32) -> i32` - wraps i32 addition
- `greet_name(name_ptr, name_len, out_ptr, out_max) -> i32` - reads a name from
  memory, writes a greeting back, and returns the number of bytes written

It uses `crate-type = ["cdylib"]` so it compiles to a standalone WASM module
with no standard library dependencies.