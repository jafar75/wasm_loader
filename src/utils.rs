use wasmer::Value;

pub fn unpack_u64_result(value: &Value) -> Result<(i32, i32), Box<dyn std::error::Error>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unpack_u64_result_normal_case() {
        let ptr: i32 = 0x12345678;
        let len: i32 = 0x43211234;
        let packed = ((ptr as u64) << 32) | (len as u64);
        let (unpacked_ptr, unpacked_len) = unpack_u64_result(&Value::I64(packed as i64)).unwrap();
        assert_eq!(unpacked_ptr, ptr);
        assert_eq!(unpacked_len, len);
    }

    #[test]
    #[should_panic(expected = "plugin returned 0 (null), indicating an error")]
    fn test_unpack_u64_result_zero() {
        let packed = 0u64;
        let _ = unpack_u64_result(&Value::I64(packed as i64)).unwrap();
    }
}
