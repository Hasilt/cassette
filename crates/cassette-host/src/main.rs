use anyhow::Result;

use cassette_host::CassetteHost;

fn main() -> Result<()> {
    let mut host = CassetteHost::new()
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let wasm_path = "target/wasm32-unknown-unknown/release/hello_plugin.wasm";
    host.load_plugin_from_file(std::path::Path::new(wasm_path))
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let result = host.call_add(2, 3)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("add(2, 3) = {result}");

    let greeting = host.call_greet_name("World")
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("greet_name(\"World\") = \"{greeting}\"");

    Ok(())
}