# wasm_loader
wasm plugin host in Rust!  This is a fun challenge to build a wasm module, load it into a host and play with it via send input and fetching output!

# running tests
`cargo test`

# build the plugin
inside plugins/string_lover

`cargo build --target wasm32-unknown-unknown`

# running host
inside root

`cargo run -- ./plugins/string_lover/target/wasm32-unknown-unknown/debug/string_lover.wasm "HELLO, WASM!"`


# Future works
- Using `Miri` or `Address Santizer` to detect any potential leaks
- Developing a second plugin, which it's input, is the output of the `string_lover` plugin, doing some operation and so on.
- Doing some performance test.
- Finding a way to better alloc/dealloc the memory. in the current version, if `dealloc()` not be called by host, leaks can be happend. Finding some more automatic way (if any).
