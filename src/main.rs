mod runner;
mod utils;

use crate::runner::run_plugin;
use std::env;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initializing tracing crate
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

    run_plugin(wasm_path, input)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_with_hello() {
        let output = run_plugin_test("Hello from test").unwrap();
        assert_eq!(output, "tset morf olleH");
    }

    #[test]
    fn test_plugin_with_empty() {
        let output = run_plugin_test("").unwrap();
        assert_eq!(output, "");
    }

    fn run_plugin_test(input: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Path to .wasm built from plugin crate
        let wasm_path =
            "./plugins/string_lover/target/wasm32-unknown-unknown/debug/string_lover.wasm";

        // Call your internal logic here (same as what main would do)
        run_plugin(wasm_path, input)
    }
}
