use std::env;
use std::fs;
use wasmer::{imports, Instance, Module, Store, Value, Memory, Function};
use wasmer::Value::I32;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: wasm loader <plugin.wasm> <input_string>");
        std::process::exit(1);
    }

    let wasm_path = &args[1];
    let input = &args[2];

    let wasm_bytes = fs::read(wasm_path)?;
    let mut store = Store::default();
    let module = Module::new(&store, wasm_bytes)?;
    let import_object = imports! {};
    let instance = Instance::new(&mut store, &module, &import_object)?;

    println!("Loaded plugin successfully.");

    // Get memory
    let memory = instance.exports.get_memory("memory")?;

    // Get alloc and dealloc functions
    let alloc = instance.exports.get_function("alloc")?;
    let dealloc = instance.exports.get_function("dealloc")?;

    // Allocate memory for input string in plugin
    let input_len = input.len() as i32;
    let alloc_result = alloc.call(&mut store, &[Value::I32(input_len)])?;
    println!("alloc result: {:?}", alloc_result);
    let input_ptr = match alloc_result[0] {
        Value::I32(ptr) => ptr,
        _ => return Err("alloc did not return a pointer".into()),
    };

    println!("allocated memory successfully.");

    // Write input into plugin memory
    let mem = memory.view(&store);
    mem.write(input_ptr as u64, input.as_bytes())?;

    println!("Wrote to memory.");

    // Call `process(ptr, len)` → returns [ptr, len]
    let process = instance.exports.get_function("process")?;
    let result = process.call(&mut store, &[I32(input_ptr), I32(input_len)])?;

    println!("Process returned {:?}", result);

    let packed = match result[0] {
        Value::I64(val) => val as u64,
        _ => return Err("process did not return a u64 (I64)".into()),
    };

    // Decode packed u64 into ptr and len
    let output_ptr = (packed >> 32) as i32;
    let output_len = (packed & 0xFFFFFFFF) as i32;

    println!("output pointer: {},  output len: {}", output_ptr, output_len);

    // Read string from plugin memory
    let output = {
        let mem = memory.view(&store);
        let mut output = vec![0_u8; output_len as usize];
        mem.read(output_ptr as u64, &mut output).expect("TODO: handle read output");
        String::from_utf8(output)?
    };

    println!("Plugin output: {}", output);

    // Deallocate output string to avoid memory leak
    dealloc.call(&mut store, &[I32(output_ptr), I32(output_len)])?;
    dealloc.call(&mut store, &[I32(input_ptr), I32(input_len)])?;

    Ok(())
}
