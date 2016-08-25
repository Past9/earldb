use error::AssertionError;
use storage::binary_storage;

pub fn u64_as_usize(n: u64) -> Result<usize, AssertionError> {
    try!(AssertionError::assert_not(
        n > usize::max_value() as u64, 
        binary_storage::ERR_ARITHMETIC_OVERFLOW
    ));
    Ok(n as usize)
}

pub fn usize_add(a: usize, b: usize) -> Result<usize, AssertionError> {
    match a.checked_add(b) {
        Some(n) => Ok(n),
        None => Err(AssertionError::new(binary_storage::ERR_ARITHMETIC_OVERFLOW))
    }
}

pub fn u64_add(a: u64, b: u64) -> Result<u64, AssertionError> {
    match a.checked_add(b) {
        Some(n) => Ok(n),
        None => Err(AssertionError::new(binary_storage::ERR_ARITHMETIC_OVERFLOW))
    }
}

pub fn xor_checksum(bytes: &[u8]) -> u8 {
    let mut res = 0x0;
    for byte in bytes {
        res = res ^ byte;
    }
    res
}

