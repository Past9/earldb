use error::{ Error, MemoryError, AssertionError };
use alloc::heap;


pub static ERR_ARITHMETIC_OVERFLOW: & 'static str = "Arithmetic overflow";

pub fn is_power_of_two(n: usize) -> bool {
    return (n != 0) && (n & (n - 1)) == 0;
}

pub fn safe_alloc(size: u64, align: usize) -> Result<*mut u8, Error> {
    let s = try!(safe_u64_as_usize(size));
    Ok(
        unsafe { heap::allocate(s, align) }
    )
}

pub fn safe_realloc(ptr: *mut u8, old_size: u64, size: u64, align: usize) -> Result<*mut u8, Error> {
    let os = try!(safe_u64_as_usize(old_size));
    let ns = try!(safe_u64_as_usize(size));
    Ok(
        unsafe { heap::reallocate(ptr, os, ns, align) }
    )
}

pub fn safe_u64_as_usize(n: u64) -> Result<usize, AssertionError> {
    if n > (usize::max_value() as u64) { return Err(AssertionError::new(ERR_ARITHMETIC_OVERFLOW)) }
    Ok(n as usize) 
}


#[cfg(test)]
mod util_tests {

    use storage::util;

    // is_power_of_two() tests
    #[test]
    fn is_power_of_two_returns_false_for_0() {
        assert!(!util::is_power_of_two(0));
    }

    #[test]
    fn is_power_of_two_returns_false_for_non_exponential_multiples_of_2() {
        assert!(!util::is_power_of_two(6));
        assert!(!util::is_power_of_two(60));
        assert!(!util::is_power_of_two(100));
        assert!(!util::is_power_of_two(1208));
        assert!(!util::is_power_of_two(2026));
        assert!(!util::is_power_of_two(3232));
        assert!(!util::is_power_of_two(4598));
    }

    #[test]
    fn is_power_of_two_returns_false_for_uneven_numbers() {
        assert!(!util::is_power_of_two(3));
        assert!(!util::is_power_of_two(5));
        assert!(!util::is_power_of_two(87));
        assert!(!util::is_power_of_two(329));
        assert!(!util::is_power_of_two(9431));
        assert!(!util::is_power_of_two(23421));
        assert!(!util::is_power_of_two(534899));
    }

    #[test]
    fn is_power_of_two_returns_true_for_whole_powers_of_2() {
        assert!(util::is_power_of_two(1));
        assert!(util::is_power_of_two(2));
        assert!(util::is_power_of_two(4));
        assert!(util::is_power_of_two(8));
        assert!(util::is_power_of_two(16));
        assert!(util::is_power_of_two(32));
        assert!(util::is_power_of_two(64));
        assert!(util::is_power_of_two(128));
        assert!(util::is_power_of_two(256));
        assert!(util::is_power_of_two(512));
        assert!(util::is_power_of_two(1024));
        assert!(util::is_power_of_two(2048));
        assert!(util::is_power_of_two(4096));
        assert!(util::is_power_of_two(8192));
        assert!(util::is_power_of_two(16384));
        assert!(util::is_power_of_two(32768));
        assert!(util::is_power_of_two(65536));
        assert!(util::is_power_of_two(131072));
        assert!(util::is_power_of_two(262144));
        assert!(util::is_power_of_two(524288));
        assert!(util::is_power_of_two(1048576));
        assert!(util::is_power_of_two(2097152));
        assert!(util::is_power_of_two(4194304));
        assert!(util::is_power_of_two(8388608));
        assert!(util::is_power_of_two(16777216));
    }

}
