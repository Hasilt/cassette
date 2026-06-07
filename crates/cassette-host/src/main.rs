use anyhow::{bail, Result};
use std::fs;
use wasmtime::{Engine, ExternType, FuncType, Linker, Memory, Module, Store};

fn format_signature(func_ty: &FuncType) -> String {
    let params: Vec<String> = func_ty.params().map(|p| p.to_string()).collect();
    let results: Vec<String> = func_ty.results().map(|r| r.to_string()).collect();
    format!("({}) -> ({})", params.join(", "), results.join(", "))
}

fn kind_label(ty: &ExternType) -> &'static str {
    match ty {
        ExternType::Func(_) => "function",
        ExternType::Memory(_) => "memory",
        ExternType::Table(_) => "table",
        ExternType::Global(_) => "global",
        ExternType::Tag(_) => "tag",
    }
}

fn collect_export_labels(store: &mut Store<()>, instance: &wasmtime::Instance) -> Vec<String> {
    let names: Vec<String> = instance
        .exports(&mut *store)
        .map(|e| e.name().to_string())
        .collect();
    names
        .into_iter()
        .filter_map(|name| {
            let ext = instance.get_export(&mut *store, &name)?;
            let kind = kind_label(&ext.ty(&*store));
            Some(format!("  * {name} ({kind})"))
        })
        .collect()
}



fn main() -> Result<()> {
    let wasm_bytes = fs::read("target/wasm32-unknown-unknown/release/hello_plugin.wasm")?;

    let engine = Engine::default();
    let module = Module::from_binary(&engine, &wasm_bytes)?;
    let mut store = Store::new(&engine, ());
    let linker = Linker::new(&engine);
    let instance = linker.instantiate(&mut store, &module)?;

    // --- add(2, 3) ---
    let name = "add";

    let func = match instance.get_func(&mut store, name) {
        Some(f) => f,
        None => {
            let exports = collect_export_labels(&mut store, &instance);

            eprintln!("\n  Function '{name}' does not exist.\n");
            eprintln!("  Available exports:\n");
            for line in exports {
                eprintln!("{line}");
            }
            eprintln!();
            bail!("export '{name}' not found");
        }
    };

    let func_ty = func.ty(&mut store);
    let expected = FuncType::new(
        &engine,
        [wasmtime::ValType::I32, wasmtime::ValType::I32],
        [wasmtime::ValType::I32],
    );

    if !func_ty.matches(&expected) {
        eprintln!("\n  Expected signature:");
        eprintln!("  {}", format_signature(&expected));
        eprintln!();
        eprintln!("  Actual signature:");
        eprintln!("  {}", format_signature(&func_ty));
        eprintln!();
        bail!("signature mismatch for '{name}'");
    }

    let add = func.typed::<(i32, i32), i32>(&mut store)?;
    let result = add.call(&mut store, (2, 3))?;
    println!("add(2, 3) = {result}");

    // --- greet_name("World") ---
    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or_else(|| anyhow::anyhow!("missing 'memory' export"))?;

    let heap_base = {
        let global = instance
            .get_global(&mut store, "__heap_base")
            .ok_or_else(|| anyhow::anyhow!("missing '__heap_base' export"))?;
        global.get(&mut store).unwrap_i32() as usize
    };

    let input = b"World";
    let out_capacity = 256;

    // Layout in linear memory starting at heap_base:
    //   [0 .. input.len()]         → input string ("World")
    //   [256 .. 256 + out_capacity] → output buffer ("Hello, World!")
    let input_offset = heap_base;
    let output_offset = heap_base + 256;

    write_bytes(&mut store, &memory, input_offset, input);
    write_bytes(&mut store, &memory, output_offset, &[0u8; 256]);

    let greet_name_sig = FuncType::new(
        &engine,
        [
            wasmtime::ValType::I32,
            wasmtime::ValType::I32,
            wasmtime::ValType::I32,
            wasmtime::ValType::I32,
        ],
        [wasmtime::ValType::I32],
    );

    let func = match instance.get_func(&mut store, "greet_name") {
        Some(f) => f,
        None => {
            let exports = collect_export_labels(&mut store, &instance);
            eprintln!("\n  Function 'greet_name' does not exist.\n");
            eprintln!("  Available exports:\n");
            for line in exports {
                eprintln!("{line}");
            }
            eprintln!();
            bail!("export 'greet_name' not found");
        }
    };

    let func_ty = func.ty(&mut store);
    if !func_ty.matches(&greet_name_sig) {
        eprintln!("\n  Expected signature:");
        eprintln!("  {}", format_signature(&greet_name_sig));
        eprintln!();
        eprintln!("  Actual signature:");
        eprintln!("  {}", format_signature(&func_ty));
        eprintln!();
        bail!("signature mismatch for 'greet_name'");
    }

    let greet_name = func.typed::<(i32, i32, i32, i32), i32>(&mut store)?;
    let written = greet_name.call(
        &mut store,
        (input_offset as i32, input.len() as i32, output_offset as i32, out_capacity as i32),
    )?;

    let result = {
        let data = memory.data(&store);
        std::str::from_utf8(&data[output_offset..output_offset + written as usize])
            .map_err(|e| anyhow::anyhow!("result is not valid UTF-8: {e}"))?
    };

    println!("greet_name(\"World\") = \"{result}\"");
    println!("  written = {written} bytes");

    Ok(())
}

fn write_bytes(store: &mut Store<()>, memory: &Memory, offset: usize, bytes: &[u8]) {
    memory.data_mut(store)[offset..offset + bytes.len()].copy_from_slice(bytes);
}