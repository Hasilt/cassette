use anyhow::Result;
use std::fs;
use wasmtime::{Engine, Linker, Module, Store};

fn main() -> Result<()> {
    let wasm_bytes = fs::read("target/wasm32-unknown-unknown/release/hello_plugin.wasm")?;

    let engine = Engine::default();
    let module = Module::from_binary(&engine, &wasm_bytes)?;
    let mut store = Store::new(&engine, ());
    let linker = Linker::new(&engine);
    let instance = linker.instantiate(&mut store, &module)?;

    let add = instance.get_typed_func::<(i32, i32), i32>(&mut store, "add")?;

    let result = add.call(&mut store, (2, 3))?;

    println!("add(2, 3) = {result}");
    Ok(())
}