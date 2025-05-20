

use std::slice;
use std::str;
use std::ptr;


#[unsafe(no_mangle)]
pub extern "C" fn alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf); // give up ownership to let host writing to it
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    unsafe {
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}

/// Entry point for our string lover wasm module
#[unsafe(no_mangle)]
pub extern "C" fn process(input_ptr: *const u8, input_len: usize) -> u64 {
    // Read input string
    let input = unsafe { slice::from_raw_parts(input_ptr, input_len) };
    let input_str = match str::from_utf8(input) {
        Ok(s) => s,
        Err(_) => return 0, // could not read the input data
    };

    // Reverse string
    let output = input_str.chars().rev().collect::<String>();
    let len = output.len();
    let boxed = output.into_bytes().into_boxed_slice();

    // Leak memory so pointer remains valid
    let ptr = Box::leak(boxed).as_mut_ptr();

    // Return pointer and length encoded in u64: high 32 bits = ptr, low 32 bits = len
    ((ptr as u64) << 32) | len as u64
}
