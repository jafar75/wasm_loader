use crate::utils::unpack_u64_result;
use std::fs;
use tracing::{error, info};
use wasmer::Value::I32;
use wasmer::{Instance, Module, Store, Value, imports};

pub fn run_plugin(wasm_path: &str, input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let wasm_bytes = fs::read(wasm_path)?;
    let mut store = Store::default();
    let module = Module::new(&store, wasm_bytes)?;
    let import_object = imports! {};
    let instance = Instance::new(&mut store, &module, &import_object)?;

    info!("Loaded plugin successfully.");

    // Get memory
    let memory = instance.exports.get_memory("memory")?;

    // Get alloc and dealloc functions
    let alloc = instance.exports.get_function("alloc")?;
    let dealloc = instance.exports.get_function("dealloc")?;

    // Allocate memory for input string in plugin
    let input_len = input.len() as i32;
    let alloc_result = alloc.call(&mut store, &[Value::I32(input_len)])?;
    let input_ptr = match alloc_result[0] {
        Value::I32(ptr) => ptr,
        _ => {
            error!("Failed to get input ptr");
            return Err("alloc did not return a pointer".into());
        }
    };

    info!("allocated memory successfully.");

    // Write input into plugin memory
    let mem = memory.view(&store);
    mem.write(input_ptr as u64, input.as_bytes())?;

    info!("Wrote to memory.");

    // Call `process(ptr, len)` → returns [ptr, len]
    let process = instance.exports.get_function("process")?;
    let result = process.call(&mut store, &[I32(input_ptr), I32(input_len)])?;

    let (output_ptr, output_len) = unpack_u64_result(&result[0])?;

    // Read string from plugin memory
    let output = {
        let mem = memory.view(&store);
        let mut buffer = vec![0_u8; output_len as usize];
        mem.read(output_ptr as u64, &mut buffer)
            .expect("TODO: handle read output");
        String::from_utf8(buffer)?
    };

    info!("Plugin output: {}", output);

    // Deallocate output string to avoid memory leak
    dealloc.call(&mut store, &[I32(output_ptr), I32(output_len)])?;
    dealloc.call(&mut store, &[I32(input_ptr), I32(input_len)])?;

    Ok(output)
}
