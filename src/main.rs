use std::env;
use std::fs;
use tracing::level_filters::LevelFilter;
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use wasmer::Value::I32;
use wasmer::{Instance, Module, Store, Value, imports};

fn unpack_u64_result(value: &Value) -> Result<(i32, i32), String> {
    match value {
        Value::I64(val) => {
            let packed = *val as u64;
            if packed == 0 {
                return Err("plugin returned 0 (null), indicating an error".into());
            }
            let len = (packed & 0xFFFF_FFFF) as i32;
            let ptr = (packed >> 32) as i32;
            Ok((ptr, len))
        }
        _ => Err("plugin did not return an I64".into()),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(LevelFilter::INFO)
        .with(tracing_subscriber::fmt::Layer::new())
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        info!("Usage: wasm loader <plugin.wasm> <input_string>");
        std::process::exit(1);
    }

    let wasm_path = &args[1];
    let input = &args[2];

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

    Ok(())
}
