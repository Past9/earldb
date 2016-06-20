use error::{ Error };

pub static ERR_STORAGE_ALLOC: &'static str = 
    "Storage allocation failed";
pub static ERR_ARITHMETIC_OVERFLOW: &'static str = 
    "Operation failed due to arithmetic overflow";
pub static ERR_STORAGE_OUT_OF_BOUNDS: &'static str =
    "Storage address out of bounds";
pub static ERR_EXPAND_SIZE_TOO_SMALL: &'static str = 
    "Expansion size must be greater that zero";
pub static ERR_INITIAL_CAP_TOO_SMALL: &'static str = 
    "Initial capacity must be greater than zero";
pub static ERR_MAX_PAGE_SIZE_NOT_POWER_OF_2: &'static str = 
    "Max page size must be a power of 2";
pub static ERR_INITIAL_CAP_NOT_POWER_OF_2: &'static str = 
    "Initial capacity must be a power of 2";
pub static ERR_EXPAND_SIZE_NOT_POWER_OF_2: &'static str = 
    "Expansion size must be a power of 2";
pub static ERR_WRITE_BEFORE_TXN_BOUNDARY: & 'static str = 
    "Cannot write before transaction boundary";
pub static ERR_WRITE_PAST_END: & 'static str = 
    "Cannot write past end of allocated storage";
pub static ERR_READ_AFTER_TXN_BOUNDARY: & 'static str = 
    "Cannot read after transaction boundary";
pub static ERR_READ_PAST_END: & 'static str = 
    "Cannot read past end of allocated storage";
pub static ERR_OPERATION_INVALID_WHEN_NOT_USING_TXN_BOUNDARY: & 'static str = 
    "Cannot perform this operation when transaction boundary is not in use";
pub static ERR_SET_TXN_BOUNDARY_PAST_END: & 'static str = 
    "Cannot set transaction boundary past end of allocated storage";
pub static ERR_OPERATION_INVALID_WHEN_OPEN: & 'static str = 
    "Cannot perform this operation when storage is open";
pub static ERR_OPERATION_INVALID_WHEN_CLOSED: & 'static str = 
    "Cannot perform this operation when storage is closed";
pub static ERR_WRITE_NOTHING: & 'static str = 
    "End of write must be after start of write";
pub static ERR_READ_NOTHING: & 'static str = 
    "End of read must be after start of read";

pub trait BinaryStorage {

    fn open(&mut self) -> Result<(), Error>;
    fn close(&mut self) -> Result<(), Error>;

    fn is_open(&self) -> bool;

    fn w_i8(&mut self, offset: u64, data: i8) -> Result<(), Error>;
    fn w_i16(&mut self, offset: u64, data: i16) -> Result<(), Error>;
    fn w_i32(&mut self, offset: u64, data: i32) -> Result<(), Error>;
    fn w_i64(&mut self, offset: u64, data: i64) -> Result<(), Error>;

    fn w_u8(&mut self, offset: u64, data: u8) -> Result<(), Error>;
    fn w_u16(&mut self, offset: u64, data: u16) -> Result<(), Error>;
    fn w_u32(&mut self, offset: u64, data: u32) -> Result<(), Error>;
    fn w_u64(&mut self, offset: u64, data: u64) -> Result<(), Error>;

    fn w_f32(&mut self, offset: u64, data: f32) -> Result<(), Error>;
    fn w_f64(&mut self, offset: u64, data: f64) -> Result<(), Error>;

    fn w_bool(&mut self, offset: u64, data: bool) -> Result<(), Error>;

    fn w_bytes(&mut self, offset: u64, data: &[u8]) -> Result<(), Error>;
    fn w_str(&mut self, offset: u64, data: &str) -> Result<(), Error>;


    fn r_i8(&mut self, offset: u64) -> Result<i8, Error>;
    fn r_i16(&mut self, offset: u64) -> Result<i16, Error>;
    fn r_i32(&mut self, offset: u64) -> Result<i32, Error>;
    fn r_i64(&mut self, offset: u64) -> Result<i64, Error>;

    fn r_u8(&mut self, offset: u64) -> Result<u8, Error>;
    fn r_u16(&mut self, offset: u64) -> Result<u16, Error>;
    fn r_u32(&mut self, offset: u64) -> Result<u32, Error>;
    fn r_u64(&mut self, offset: u64) -> Result<u64, Error>;

    fn r_f32(&mut self, offset: u64) -> Result<f32, Error>;
    fn r_f64(&mut self, offset: u64) -> Result<f64, Error>;

    fn r_bool(&mut self, offset: u64) -> Result<bool, Error>;

    fn r_bytes(&mut self, offset: u64, len: usize) -> Result<Vec<u8>, Error>;
    fn r_str(&mut self, offset: u64, len: usize) -> Result<String, Error>;

    fn fill(&mut self, start: Option<u64>, end: Option<u64>, val: u8) -> Result<(), Error>;
    fn is_filled(&mut self, start: Option<u64>, end: Option<u64>, val: u8) -> Result<bool, Error>;

    fn get_use_txn_boundary(&self) -> bool;
    fn set_use_txn_boundary(&mut self, val: bool);

    fn get_txn_boundary(&self) -> Result<u64, Error>;
    fn set_txn_boundary(&mut self, offset: u64) -> Result<(), Error>;

    fn get_expand_size(&self) -> u64;
    fn set_expand_size(&mut self, expand_size: u64) -> Result<(), Error>;

    fn get_capacity(&self) -> Result<u64, Error>;

    fn expand(&mut self, min_capacity: u64) -> Result<(), Error>;

}


#[cfg(test)]
pub mod tests {

    use std::str;

    use std::error::Error;
    use storage::binary_storage;
    use storage::binary_storage::BinaryStorage;


    // open(), close(), and is_open() tests 
    pub fn open_returns_err_when_already_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_OPEN,
            s.open().unwrap_err().description()
        );
    }

    pub fn close_returns_err_when_already_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.close().unwrap_err().description()
        );
    }

    pub fn open_returns_ok_when_previously_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert!(s.open().is_ok());
    }

    pub fn close_returns_ok_when_previously_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.close().is_ok());
    }

    pub fn is_closed_when_new<T: BinaryStorage>(s: T) {
        assert!(!s.is_open());
    }

    pub fn is_open_after_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.is_open());
    }

    pub fn is_closed_after_open_and_close<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.close().unwrap();
        assert!(!s.is_open());
    }

    // w_i8() tests
    pub fn w_i8_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        let res = s.w_i8(0, i8::max_value());
        assert_eq!(binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED, res.unwrap_err().description());
    }

    pub fn w_i8_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.w_i8(0, i8::max_value()).is_ok());
    }

    pub fn w_i8_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        s.w_i8(0, i8::max_value()).unwrap_err();
        s.open().unwrap();
        assert_eq!(0, s.r_i8(0).unwrap());
    }

    pub fn w_i8_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(4).unwrap();
        assert_eq!(
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY, 
            s.w_i8(3, i8::max_value()).unwrap_err().description()
        );
        assert!(s.w_i8(4, i8::max_value()).is_ok());
        s.set_txn_boundary(16).unwrap();
        assert_eq!(0, s.r_i8(3).unwrap());
        assert_eq!(i8::max_value(), s.r_i8(4).unwrap());
    }

    pub fn w_i8_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
        assert!(s.w_i8(256, i8::max_value()).is_ok());
        assert_eq!(512, s.get_capacity().unwrap());
        assert_eq!(i8::max_value(), s.r_i8(256).unwrap());
    }

    // w_i16() tests
    pub fn w_i16_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        let res = s.w_i16(0, i16::max_value());
        assert_eq!(binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED, res.unwrap_err().description());
    }

    pub fn w_i16_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.w_i16(0, i16::max_value()).is_ok());
    }

    pub fn w_i16_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        s.w_i16(0, i16::max_value()).unwrap_err();
        s.open().unwrap();
        assert_eq!(0, s.r_i16(0).unwrap());
    }

    pub fn w_i16_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(4).unwrap();
        assert_eq!(
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY, 
            s.w_i16(3, i16::max_value()).unwrap_err().description()
        );
        assert!(s.w_i16(4, i16::max_value()).is_ok());
        s.set_txn_boundary(16).unwrap();
        assert_eq!(0, s.r_i16(2).unwrap());
        assert_eq!(i16::max_value(), s.r_i16(4).unwrap());
    }

    pub fn w_i16_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
        assert!(s.w_i16(256, i16::max_value()).is_ok());
        assert_eq!(512, s.get_capacity().unwrap());
        assert_eq!(i16::max_value(), s.r_i16(256).unwrap());
    }

    // w_i32() tests
    pub fn w_i32_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        let res = s.w_i32(0, i32::max_value());
        assert_eq!(binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED, res.unwrap_err().description());
    }

    pub fn w_i32_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.w_i32(0, i32::max_value()).is_ok());
    }

    pub fn w_i32_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        s.w_i32(0, i32::max_value()).unwrap_err();
        s.open().unwrap();
        assert_eq!(0, s.r_i32(0).unwrap());
    }

    pub fn w_i32_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(8).unwrap();
        assert_eq!(
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY, 
            s.w_i32(7, i32::max_value()).unwrap_err().description()
        );
        assert!(s.w_i32(8, i32::max_value()).is_ok());
        s.set_txn_boundary(16).unwrap();
        assert_eq!(0, s.r_i32(4).unwrap());
        assert_eq!(i32::max_value(), s.r_i32(8).unwrap());
    }

    pub fn w_i32_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
        assert!(s.w_i32(256, i32::max_value()).is_ok());
        assert_eq!(512, s.get_capacity().unwrap());
        assert_eq!(i32::max_value(), s.r_i32(256).unwrap());
    }

    // w_i64() tests
    pub fn w_i64_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        let res = s.w_i64(0, i64::max_value());
        assert_eq!(binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED, res.unwrap_err().description());
    }

    pub fn w_i64_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.w_i64(0, i64::max_value()).is_ok());
    }

    pub fn w_i64_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        s.w_i64(0, i64::max_value()).unwrap_err();
        s.open().unwrap();
        assert_eq!(0, s.r_i64(0).unwrap());
    }

    pub fn w_i64_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(8).unwrap();
        assert_eq!(
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY, 
            s.w_i64(7, i64::max_value()).unwrap_err().description()
        );
        assert!(s.w_i64(8, i64::max_value()).is_ok());
        s.set_txn_boundary(16).unwrap();
        assert_eq!(0, s.r_i64(0).unwrap());
        assert_eq!(i64::max_value(), s.r_i64(8).unwrap());
    }

    pub fn w_i64_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
        assert!(s.w_i64(256, i64::max_value()).is_ok());
        assert_eq!(512, s.get_capacity().unwrap());
        assert_eq!(i64::max_value(), s.r_i64(256).unwrap());
    }

    // w_u8() tests
    pub fn w_u8_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        let res = s.w_u8(0, u8::max_value());
        assert_eq!(binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED, res.unwrap_err().description());
    }

    pub fn w_u8_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.w_u8(0, u8::max_value()).is_ok());
    }

    pub fn w_u8_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        s.w_u8(0, u8::max_value()).unwrap_err();
        s.open().unwrap();
        assert_eq!(0, s.r_u8(0).unwrap());
    }

    pub fn w_u8_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(4).unwrap();
        assert_eq!(
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY, 
            s.w_u8(3, u8::max_value()).unwrap_err().description()
        );
        assert!(s.w_u8(4, u8::max_value()).is_ok());
        s.set_txn_boundary(16).unwrap();
        assert_eq!(0, s.r_u8(3).unwrap());
        assert_eq!(u8::max_value(), s.r_u8(4).unwrap());
    }

    pub fn w_u8_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
        assert!(s.w_u8(256, u8::max_value()).is_ok());
        assert_eq!(512, s.get_capacity().unwrap());
        assert_eq!(u8::max_value(), s.r_u8(256).unwrap());
    }

    // w_u16() tests
    pub fn w_u16_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        let res = s.w_u16(0, u16::max_value());
        assert_eq!(binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED, res.unwrap_err().description());
    }

    pub fn w_u16_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.w_u16(0, u16::max_value()).is_ok());
    }

    pub fn w_u16_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        s.w_u16(0, u16::max_value()).unwrap_err();
        s.open().unwrap();
        assert_eq!(0, s.r_u16(0).unwrap());
    }

    pub fn w_u16_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(4).unwrap();
        assert_eq!(
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY, 
            s.w_u16(3, u16::max_value()).unwrap_err().description()
        );
        assert!(s.w_u16(4, u16::max_value()).is_ok());
        s.set_txn_boundary(16).unwrap();
        assert_eq!(0, s.r_u16(2).unwrap());
        assert_eq!(u16::max_value(), s.r_u16(4).unwrap());
    }

    pub fn w_u16_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
        assert!(s.w_u16(256, u16::max_value()).is_ok());
        assert_eq!(512, s.get_capacity().unwrap());
        assert_eq!(u16::max_value(), s.r_u16(256).unwrap());
    }

    // w_u32() tests
    pub fn w_u32_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        let res = s.w_u32(0, u32::max_value());
        assert_eq!(binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED, res.unwrap_err().description());
    }

    pub fn w_u32_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.w_u32(0, u32::max_value()).is_ok());
    }

    pub fn w_u32_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        s.w_u32(0, u32::max_value()).unwrap_err();
        s.open().unwrap();
        assert_eq!(0, s.r_u32(0).unwrap());
    }

    pub fn w_u32_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(8).unwrap();
        assert_eq!(
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY, 
            s.w_u32(7, u32::max_value()).unwrap_err().description()
        );
        assert!(s.w_u32(8, u32::max_value()).is_ok());
        s.set_txn_boundary(16).unwrap();
        assert_eq!(0, s.r_u32(4).unwrap());
        assert_eq!(u32::max_value(), s.r_u32(8).unwrap());
    }

    pub fn w_u32_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
        assert!(s.w_u32(256, u32::max_value()).is_ok());
        assert_eq!(512, s.get_capacity().unwrap());
        assert_eq!(u32::max_value(), s.r_u32(256).unwrap());
    }

    // w_u64() tests
    pub fn w_u64_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        let res = s.w_u64(0, u64::max_value());
        assert_eq!(binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED, res.unwrap_err().description());
    }

    pub fn w_u64_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.w_u64(0, u64::max_value()).is_ok());
    }

    pub fn w_u64_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        s.w_u64(0, u64::max_value()).unwrap_err();
        s.open().unwrap();
        assert_eq!(0, s.r_u64(0).unwrap());
    }

    pub fn w_u64_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(8).unwrap();
        assert_eq!(
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY, 
            s.w_u64(7, u64::max_value()).unwrap_err().description()
        );
        assert!(s.w_u64(8, u64::max_value()).is_ok());
        s.set_txn_boundary(16).unwrap();
        assert_eq!(0, s.r_u64(0).unwrap());
        assert_eq!(u64::max_value(), s.r_u64(8).unwrap());
    }

    pub fn w_u64_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
        assert!(s.w_u64(256, u64::max_value()).is_ok());
        assert_eq!(512, s.get_capacity().unwrap());
        assert_eq!(u64::max_value(), s.r_u64(256).unwrap());
    }

    // w_f32() tests
    pub fn w_f32_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        let res = s.w_f32(0, 12345.6789);
        assert_eq!(binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED, res.unwrap_err().description());
    }

    pub fn w_f32_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.w_f32(0, 12345.6789).is_ok());
    }

    pub fn w_f32_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        s.w_f32(0, 12345.6789).unwrap_err();
        s.open().unwrap();
        assert_eq!(0.0, s.r_f32(0).unwrap());
    }

    pub fn w_f32_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(8).unwrap();
        assert_eq!(
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY, 
            s.w_f32(7, 12345.6789).unwrap_err().description()
        );
        assert!(s.w_f32(8, 12345.6789).is_ok());
        s.set_txn_boundary(16).unwrap();
        assert_eq!(0.0, s.r_f32(0).unwrap());
        assert_eq!(12345.6789, s.r_f32(8).unwrap());
    }

    pub fn w_f32_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
        assert!(s.w_f32(256, 12345.6789).is_ok());
        assert_eq!(512, s.get_capacity().unwrap());
        assert_eq!(12345.6789, s.r_f32(256).unwrap());
    }

    // w_f64() tests
    pub fn w_f64_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        let res = s.w_f64(0, 12345.6789);
        assert_eq!(binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED, res.unwrap_err().description());
    }

    pub fn w_f64_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.w_f64(0, 12345.6789).is_ok());
    }

    pub fn w_f64_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        s.w_f64(0, 12345.6789).unwrap_err();
        s.open().unwrap();
        assert_eq!(0.0, s.r_f64(0).unwrap());
    }

    pub fn w_f64_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(8).unwrap();
        assert_eq!(
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY, 
            s.w_f64(7, 12345.6789).unwrap_err().description()
        );
        assert!(s.w_f64(8, 12345.6789).is_ok());
        s.set_txn_boundary(16).unwrap();
        assert_eq!(0.0, s.r_f64(0).unwrap());
        assert_eq!(12345.6789, s.r_f64(8).unwrap());
    }

    pub fn w_f64_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
        assert!(s.w_f64(256, 12345.6789).is_ok());
        assert_eq!(512, s.get_capacity().unwrap());
        assert_eq!(12345.6789, s.r_f64(256).unwrap());
    }

    // w_bool() tests
    pub fn w_bool_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED, 
            s.w_bool(0, true).unwrap_err().description()
        );
    }

    pub fn w_bool_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.w_bool(0, true).is_ok());
    }

    pub fn w_bool_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        s.w_bool(0, true).unwrap_err();
        s.open().unwrap();
        assert_eq!(false, s.r_bool(0).unwrap());
    }

    pub fn w_bool_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(4).unwrap();
        assert_eq!(
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY,
            s.w_bool(3, true).unwrap_err().description()
        );
        assert!(s.w_bool(4, true).is_ok());
        s.set_txn_boundary(8).unwrap();
        assert_eq!(false, s.r_bool(3).unwrap());
        assert_eq!(true, s.r_bool(4).unwrap());
    }

    pub fn w_bool_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
        assert!(s.w_bool(256, true).is_ok());
        assert_eq!(512, s.get_capacity().unwrap());
        assert_eq!(true, s.r_bool(256).unwrap());
    }

    // w_bytes() tests
    pub fn w_bytes_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED, 
            s.w_bytes(0, &[0x0, 0x1, 0x2, 0x3, 0x4]).unwrap_err().description()
        );
    }

    pub fn w_bytes_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.w_bytes(0, &[0x0, 0x1, 0x2, 0x3, 0x4]).is_ok());
    }

    pub fn w_bytes_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        s.w_bytes(0, &[0x0, 0x1, 0x2, 0x3, 0x4]).unwrap_err();
        s.open().unwrap();
        assert_eq!(vec!(0x0, 0x0, 0x0, 0x0, 0x0), s.r_bytes(0, 5).unwrap());
    }

    pub fn w_bytes_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(8).unwrap();
        assert_eq!(
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY,
            s.w_bytes(7, &[0x0, 0x1, 0x2, 0x3, 0x4]).unwrap_err().description()
        );
        assert!(s.w_bytes(8, &[0x0, 0x1, 0x2, 0x3, 0x4]).is_ok());
        s.set_txn_boundary(16).unwrap();
        assert_eq!(vec!(0x0, 0x0, 0x0, 0x0, 0x0), s.r_bytes(3, 5).unwrap());
        assert_eq!(vec!(0x0, 0x1, 0x2, 0x3, 0x4), s.r_bytes(8, 5).unwrap());
    }

    pub fn w_bytes_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
        assert!(s.w_bytes(255, &[0x0, 0x1]).is_ok());
        assert_eq!(512, s.get_capacity().unwrap());
        assert_eq!(vec!(0x0, 0x1), s.r_bytes(255, 2).unwrap());
    }

    pub fn w_bytes_over_capacity_expands_storage_multiple_times<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
        assert!(s.w_bytes(255, &[0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6]).is_ok());
        assert_eq!(264, s.get_capacity().unwrap());
        assert_eq!(vec!(0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6), s.r_bytes(255, 7).unwrap());
    }

    // w_str() tests
    pub fn w_str_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.w_str(0, "I \u{2661} Rust").unwrap_err().description()
        );
    }

    pub fn w_str_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.w_str(0, "I \u{2661} Rust").is_ok());
    }

    pub fn w_str_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        s.w_str(0, "I \u{2661} Rust").unwrap_err();
        s.open().unwrap();
        assert_eq!(
            str::from_utf8(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0]).unwrap(), 
            s.r_str(0, 10).unwrap()
        );
    }

    pub fn w_str_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(16).unwrap();
        assert_eq!(
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY,
            s.w_str(15, "I \u{2661} Rust").unwrap_err().description()
        );
        assert!(s.w_str(16, "I \u{2661} Rust").is_ok());
        s.set_txn_boundary(32).unwrap();
        assert_eq!(
            str::from_utf8(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0]).unwrap(), 
            s.r_str(6, 10).unwrap()
        );
        assert_eq!("I \u{2661} Rust", s.r_str(16, 10).unwrap());
    }

    pub fn w_str_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
        assert!(s.w_str(255, "I \u{2661} Rust").is_ok());
        assert_eq!(512, s.get_capacity().unwrap());
        assert_eq!("I \u{2661} Rust", s.r_str(255, 10).unwrap());
    }

    pub fn w_str_over_capacity_expands_storage_multiple_times<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
        assert!(s.w_str(255, "I \u{2661} Rust").is_ok());
        assert_eq!(268, s.get_capacity().unwrap());
        assert_eq!("I \u{2661} Rust", s.r_str(255, 10).unwrap());
    }

    // r_i8() tests
    pub fn r_i8_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.r_i8(0).unwrap_err().description()
        );
    }

    pub fn r_i8_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.r_i8(0).unwrap();
        assert!(s.r_i8(0).is_ok());
    }

    pub fn r_i8_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(0, s.r_i8(0).unwrap());
    }

    pub fn r_i8_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_i8(0, i8::max_value()).unwrap();
        assert_eq!(i8::max_value(), s.r_i8(0).unwrap());
        s.w_i8(32, i8::min_value()).unwrap();
        assert_eq!(i8::min_value(), s.r_i8(32).unwrap());
    }

    pub fn r_i8_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(4).unwrap();
        assert!(s.r_i8(3).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            s.r_i8(4).unwrap_err().description()
        );
    }

    pub fn r_i8_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_i8(255).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_PAST_END,
            s.r_i8(256).unwrap_err().description()
        );
    }

    pub fn r_i8_result_is_not_mutated_on_subsequent_write<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_i8(0, i8::max_value()).unwrap();
        let res1 = s.r_i8(0).unwrap();
        assert_eq!(i8::max_value(), res1);
        s.w_i8(0, i8::max_value() - 10).unwrap();
        let res2 = s.r_i8(0).unwrap();
        assert_eq!(i8::max_value(), res1);
        assert_eq!(i8::max_value() - 10, res2);
    }

    // r_i16() tests
    pub fn r_i16_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.r_i16(0).unwrap_err().description()
        );
    }

    pub fn r_i16_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_i16(0).is_ok());
    }

    pub fn r_i16_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(0, s.r_i16(0).unwrap());
    }

    pub fn r_i16_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_i16(0, i16::max_value()).unwrap();
        assert_eq!(i16::max_value(), s.r_i16(0).unwrap());
        s.w_i16(32, i16::max_value()).unwrap();
        assert_eq!(i16::max_value(), s.r_i16(32).unwrap());
    }

    pub fn r_i16_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(4).unwrap();
        assert!(s.r_i16(2).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            s.r_i16(3).unwrap_err().description()
        );
    }

    pub fn r_i16_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_i16(254).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_PAST_END,
            s.r_i16(256).unwrap_err().description()
        );
    }

    pub fn r_i16_result_is_not_mutated_on_subsequent_write<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_i16(0, i16::max_value()).unwrap();
        let res1 = s.r_i16(0).unwrap();
        assert_eq!(i16::max_value(), res1);
        s.w_i16(0, i16::max_value() - 10).unwrap();
        let res2 = s.r_i16(0).unwrap();
        assert_eq!(i16::max_value(), res1);
        assert_eq!(i16::max_value() - 10, res2);
    }

    // r_i32() tests
    pub fn r_i32_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.r_i32(0).unwrap_err().description()
        );
    }

    pub fn r_i32_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_i32(0).is_ok());
    }

    pub fn r_i32_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(0, s.r_i32(0).unwrap());
    }

    pub fn r_i32_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_i32(0, i32::max_value()).unwrap();
        assert_eq!(i32::max_value(), s.r_i32(0).unwrap());
        s.w_i32(32, i32::max_value()).unwrap();
        assert_eq!(i32::max_value(), s.r_i32(32).unwrap());
    }

    pub fn r_i32_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(8).unwrap();
        assert!(s.r_i32(4).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            s.r_i32(5).unwrap_err().description()
        );
    }

    pub fn r_i32_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_i32(252).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_PAST_END,
            s.r_i32(256).unwrap_err().description()
        );
    }

    pub fn r_i32_result_is_not_mutated_on_subsequent_write<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_i32(0, i32::max_value()).unwrap();
        let res1 = s.r_i32(0).unwrap();
        assert_eq!(i32::max_value(), res1);
        s.w_i32(0, i32::max_value() - 10).unwrap();
        let res2 = s.r_i32(0).unwrap();
        assert_eq!(i32::max_value(), res1);
        assert_eq!(i32::max_value() - 10, res2);
    }

    // r_i64() tests
    pub fn r_i64_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.r_i64(0).unwrap_err().description()
        );
    }

    pub fn r_i64_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_i64(0).is_ok());
    }

    pub fn r_i64_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(0, s.r_i64(0).unwrap());
    }

    pub fn r_i64_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_i64(0, i64::max_value()).unwrap();
        assert_eq!(i64::max_value(), s.r_i64(0).unwrap());
        s.w_i64(32, i64::max_value()).unwrap();
        assert_eq!(i64::max_value(), s.r_i64(32).unwrap());
    }

    pub fn r_i64_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(8).unwrap();
        assert!(s.r_i64(0).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            s.r_i64(1).unwrap_err().description()
        );
    }

    pub fn r_i64_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_i64(248).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_PAST_END,
            s.r_i64(256).unwrap_err().description()
        );
    }

    pub fn r_i64_result_is_not_mutated_on_subsequent_write<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_i64(0, i64::max_value()).unwrap();
        let res1 = s.r_i64(0).unwrap();
        assert_eq!(i64::max_value(), res1);
        s.w_i64(0, i64::max_value() - 10).unwrap();
        let res2 = s.r_i64(0).unwrap();
        assert_eq!(i64::max_value(), res1);
        assert_eq!(i64::max_value() - 10, res2);
    }

    // r_u8() tests
    pub fn r_u8_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.r_u8(0).unwrap_err().description()
        );
    }

    pub fn r_u8_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_u8(0).is_ok());
    }

    pub fn r_u8_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(0, s.r_u8(0).unwrap());
    }

    pub fn r_u8_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_u8(0, u8::max_value()).unwrap();
        assert_eq!(u8::max_value(), s.r_u8(0).unwrap());
        s.w_u8(32, u8::max_value()).unwrap();
        assert_eq!(u8::max_value(), s.r_u8(32).unwrap());
    }

    pub fn r_u8_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(8).unwrap();
        assert!(s.r_u8(7).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            s.r_u8(8).unwrap_err().description()
        );
    }

    pub fn r_u8_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_u8(255).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_PAST_END,
            s.r_u8(256).unwrap_err().description()
        );
    }

    pub fn r_u8_result_is_not_mutated_on_subsequent_write<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_u8(0, u8::max_value()).unwrap();
        let res1 = s.r_u8(0).unwrap();
        assert_eq!(u8::max_value(), res1);
        s.w_u8(0, u8::max_value() - 10).unwrap();
        let res2 = s.r_u8(0).unwrap();
        assert_eq!(u8::max_value(), res1);
        assert_eq!(u8::max_value() - 10, res2);
    }

    // r_u16() tests
    pub fn r_u16_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.r_u16(0).unwrap_err().description()
        );
    }

    pub fn r_u16_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_u16(0).is_ok());
    }

    pub fn r_u16_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(0, s.r_u16(0).unwrap());
    }

    pub fn r_u16_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_u16(0, u16::max_value()).unwrap();
        assert_eq!(u16::max_value(), s.r_u16(0).unwrap());
        s.w_u16(32, u16::max_value()).unwrap();
        assert_eq!(u16::max_value(), s.r_u16(32).unwrap());
    }

    pub fn r_u16_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(8).unwrap();
        assert!(s.r_u16(6).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            s.r_u16(8).unwrap_err().description()
        );
    }

    pub fn r_u16_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_u16(254).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_PAST_END,
            s.r_u16(256).unwrap_err().description()
        );
    }

    pub fn r_u16_result_is_not_mutated_on_subsequent_write<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_u16(0, u16::max_value()).unwrap();
        let res1 = s.r_u16(0).unwrap();
        assert_eq!(u16::max_value(), res1);
        s.w_u16(0, u16::max_value() - 10).unwrap();
        let res2 = s.r_u16(0).unwrap();
        assert_eq!(u16::max_value(), res1);
        assert_eq!(u16::max_value() - 10, res2);
    }

    // r_u32() tests
    pub fn r_u32_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.r_u32(0).unwrap_err().description()
        );
    }

    pub fn r_u32_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_u32(0).is_ok());
    }

    pub fn r_u32_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(0, s.r_u32(0).unwrap());
    }

    pub fn r_u32_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_u32(0, u32::max_value()).unwrap();
        assert_eq!(u32::max_value(), s.r_u32(0).unwrap());
        s.w_u32(32, u32::max_value()).unwrap();
        assert_eq!(u32::max_value(), s.r_u32(32).unwrap());
    }

    pub fn r_u32_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(8).unwrap();
        assert!(s.r_u32(4).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            s.r_u32(8).unwrap_err().description()
        );
    }

    pub fn r_u32_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_u32(252).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_PAST_END,
            s.r_u32(256).unwrap_err().description()
        );
    }

    pub fn r_u32_result_is_not_mutated_on_subsequent_write<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_u32(0, u32::max_value()).unwrap();
        let res1 = s.r_u32(0).unwrap();
        assert_eq!(u32::max_value(), res1);
        s.w_u32(0, u32::max_value() - 10).unwrap();
        let res2 = s.r_u32(0).unwrap();
        assert_eq!(u32::max_value(), res1);
        assert_eq!(u32::max_value() - 10, res2);
    }

    // r_u64() tests
    pub fn r_u64_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.r_u64(0).unwrap_err().description()
        );
    }

    pub fn r_u64_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_u64(0).is_ok());
    }

    pub fn r_u64_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(0, s.r_u64(0).unwrap());
    }

    pub fn r_u64_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_u64(0, u64::max_value()).unwrap();
        assert_eq!(u64::max_value(), s.r_u64(0).unwrap());
        s.w_u64(32, u64::max_value()).unwrap();
        assert_eq!(u64::max_value(), s.r_u64(32).unwrap());
    }

    pub fn r_u64_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(8).unwrap();
        assert!(s.r_u64(0).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            s.r_u64(8).unwrap_err().description()
        );
    }

    pub fn r_u64_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_u64(248).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_PAST_END,
            s.r_u64(256).unwrap_err().description()
        );
    }

    pub fn r_u64_result_is_not_mutated_on_subsequent_write<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_u64(0, u64::max_value()).unwrap();
        let res1 = s.r_u64(0).unwrap();
        assert_eq!(u64::max_value(), res1);
        s.w_u64(0, u64::max_value() - 10).unwrap();
        let res2 = s.r_u64(0).unwrap();
        assert_eq!(u64::max_value(), res1);
        assert_eq!(u64::max_value() - 10, res2);
    }

    // r_f32() tests
    pub fn r_f32_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.r_f32(0).unwrap_err().description()
        );
    }

    pub fn r_f32_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_f32(0).is_ok());
    }

    pub fn r_f32_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(0.0, s.r_f32(0).unwrap());
    }

    pub fn r_f32_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_f32(0, 12345.6789).unwrap();
        assert_eq!(12345.6789, s.r_f32(0).unwrap());
        s.w_f32(32, 12345.6789).unwrap();
        assert_eq!(12345.6789, s.r_f32(32).unwrap());
    }

    pub fn r_f32_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(8).unwrap();
        assert!(s.r_f32(4).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            s.r_f32(8).unwrap_err().description()
        );
    }

    pub fn r_f32_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_f32(252).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_PAST_END,
            s.r_f32(256).unwrap_err().description()
        );
    }

    pub fn r_f32_result_is_not_mutated_on_subsequent_write<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_f32(0, 12345.6789).unwrap();
        let res1 = s.r_f32(0).unwrap();
        assert_eq!(12345.6789, res1);
        s.w_f32(0, 12345.6789 - 10.0).unwrap();
        let res2 = s.r_f32(0).unwrap();
        assert_eq!(12345.6789, res1);
        assert_eq!(12345.6789 - 10.0, res2);
    }

    // r_f64() tests
    pub fn r_f64_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.r_f64(0).unwrap_err().description()
        );
    }

    pub fn r_f64_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_f64(0).is_ok());
    }

    pub fn r_f64_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(0.0, s.r_f64(0).unwrap());
    }

    pub fn r_f64_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_f64(0, 12345.6789).unwrap();
        assert_eq!(12345.6789, s.r_f64(0).unwrap());
        s.w_f64(32, 12345.6789).unwrap();
        assert_eq!(12345.6789, s.r_f64(32).unwrap());
    }

    pub fn r_f64_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(8).unwrap();
        assert!(s.r_f64(0).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            s.r_f64(8).unwrap_err().description()
        );
    }

    pub fn r_f64_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_f64(248).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_PAST_END,
            s.r_f64(256).unwrap_err().description()
        );
    }

    pub fn r_f64_result_is_not_mutated_on_subsequent_write<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_f64(0, 12345.6789).unwrap();
        let res1 = s.r_f64(0).unwrap();
        assert_eq!(12345.6789, res1);
        s.w_f64(0, 12345.6789 - 10.0).unwrap();
        let res2 = s.r_f64(0).unwrap();
        assert_eq!(12345.6789, res1);
        assert_eq!(12345.6789 - 10.0, res2);
    }

    // r_bool() tests
    pub fn r_bool_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.r_bool(0).unwrap_err().description()
        );
    }

    pub fn r_bool_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_bool(0).is_ok());
    }

    pub fn r_bool_reads_false_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(false, s.r_bool(0).unwrap());
    }

    pub fn r_bool_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_bool(0, true).unwrap();
        assert_eq!(true, s.r_bool(0).unwrap());
        s.w_bool(32, true).unwrap();
        assert_eq!(true, s.r_bool(32).unwrap());
    }

    pub fn r_bool_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(8).unwrap();
        assert!(s.r_bool(7).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            s.r_bool(8).unwrap_err().description()
        );
    }

    pub fn r_bool_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_bool(255).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_PAST_END,
            s.r_bool(256).unwrap_err().description()
        );
    }

    pub fn r_bool_result_is_not_mutated_on_subsequent_write<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_bool(0, true).unwrap();
        let res1 = s.r_bool(0).unwrap();
        assert_eq!(true, res1);
        s.w_bool(0, false).unwrap();
        let res2 = s.r_bool(0).unwrap();
        assert_eq!(true, res1);
        assert_eq!(false, res2);
    }

    // r_bytes() tests
    pub fn r_bytes_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.r_bytes(0, 5).unwrap_err().description()
        );
    }

    pub fn r_bytes_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_bytes(0, 5).is_ok());
    }

    pub fn r_bytes_reads_zeros_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(vec!(0x0, 0x0, 0x0, 0x0, 0x0), s.r_bytes(0, 5).unwrap());
    }

    pub fn r_bytes_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_bytes(0, &[0x0, 0x1, 0x2, 0x3, 0x4]).unwrap();
        assert_eq!(vec!(0x0, 0x1, 0x2, 0x3, 0x4), s.r_bytes(0, 5).unwrap());
        s.w_bytes(32, &[0x5, 0x6, 0x7, 0x8, 0x9]).unwrap();
        assert_eq!(vec!(0x5, 0x6, 0x7, 0x8, 0x9), s.r_bytes(32, 5).unwrap());
    }

    pub fn r_bytes_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(8).unwrap();
        assert!(s.r_bytes(6, 2).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            s.r_bytes(7, 2).unwrap_err().description()
        );
    }

    pub fn r_bytes_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_bytes(254, 2).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_PAST_END,
            s.r_bytes(255, 2).unwrap_err().description()
        );
    }

    pub fn r_bytes_result_is_not_mutated_on_subsequent_write<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_bytes(0, &[0x0, 0x1, 0x2]).unwrap();
        let res1 = s.r_bytes(0, 3).unwrap();
        assert_eq!(vec!(0x0, 0x1, 0x2), res1);
        s.w_bytes(0, &[0x4, 0x5, 0x6]).unwrap();
        let res2 = s.r_bytes(0, 3).unwrap();
        assert_eq!(vec!(0x0, 0x1, 0x2), res1);
        assert_eq!(vec!(0x4, 0x5, 0x6), res2);
    }

    // r_str() tests
    pub fn r_str_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.r_str(0, 5).unwrap_err().description()
        );
    }

    pub fn r_str_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_str(0, 5).is_ok());
    }

    pub fn r_str_reads_nulls_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(str::from_utf8(&[0x0, 0x0, 0x0, 0x0, 0x0]).unwrap(), s.r_str(0, 5).unwrap());
    }

    pub fn r_str_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_str(0, "foobar").unwrap();
        assert_eq!("foobar", s.r_str(0, 6).unwrap());
        s.w_str(32, "I \u{2661} Rust").unwrap();
        assert_eq!("I \u{2661} Rust", s.r_str(32, 10).unwrap());
    }

    pub fn r_str_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        s.open().unwrap();
        s.set_txn_boundary(8).unwrap();
        assert!(s.r_str(6, 2).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            s.r_str(7, 2).unwrap_err().description()
        );
    }

    pub fn r_str_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.r_str(254, 2).is_ok());
        assert_eq!(
            binary_storage::ERR_READ_PAST_END,
            s.r_str(255, 2).unwrap_err().description()
        );
    }

    pub fn r_str_result_is_not_mutated_on_subsequent_write<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_str(0, "foobar").unwrap();
        let res1 = s.r_str(0, 6).unwrap();
        assert_eq!("foobar", res1);
        s.w_str(0, "barbaz").unwrap();
        let res2 = s.r_str(0, 6).unwrap();
        assert_eq!("foobar", res1);
        assert_eq!("barbaz", res2);
    }

    // fill() tests
    pub fn fill_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.fill(None, None, 0x1).unwrap_err().description()
        );
    }

    pub fn fill_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        s.fill(None, None, 0x1).unwrap_err();
        s.open().unwrap();
        assert_eq!(true, s.is_filled(None, None, 0x0).unwrap());
    }

    pub fn fill_returns_ok_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.fill(None, None, 0x1).is_ok());
    }

    pub fn fill_repeats_byte_in_storage_range<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.fill(Some(10), Some(20), 0x1).is_ok());
        assert_eq!(true, s.is_filled(Some(0), Some(10), 0x0).unwrap());
        assert!(s.is_filled(Some(10), Some(20), 0x1).unwrap());
        assert!(s.is_filled(Some(20), None, 0x0).unwrap());
    }

    pub fn fill_starts_from_beginning_when_start_offset_is_none<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(true, s.fill(None, Some(20), 0x1).is_ok());
        assert!(s.is_filled(Some(0), Some(20), 0x1).unwrap());
        assert!(s.is_filled(Some(20), None, 0x0).unwrap());
    }

    pub fn fill_goes_to_end_when_end_offset_is_none<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.fill(Some(10), None, 0x1).is_ok());
        assert!(s.is_filled(None, Some(10), 0x0).unwrap());
        assert!(s.is_filled(Some(10), None, 0x1).unwrap());
    }

    pub fn fill_returns_err_when_end_offset_is_before_start_offset<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(
            binary_storage::ERR_WRITE_NOTHING,
            s.fill(Some(20), Some(10), 0x1).unwrap_err().description()
        );
    }

    pub fn fill_does_not_write_when_end_offset_is_before_start_offset<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.fill(Some(20), Some(10), 0x1).unwrap_err();
        assert!(s.is_filled(None, None, 0x0).unwrap());
    }

    pub fn fill_returns_err_when_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.set_use_txn_boundary(true);
        s.set_txn_boundary(10).unwrap();
        assert_eq!(
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY,
            s.fill(Some(9), None, 0x1).unwrap_err().description()
        );
    }

    pub fn fill_does_not_write_when_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.set_use_txn_boundary(true);
        s.set_txn_boundary(10).unwrap();
        s.fill(Some(9), None, 0x1).unwrap_err();
        assert!(s.is_filled(None, None, 0x0).unwrap());
    }

    pub fn fill_returns_ok_when_after_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.set_use_txn_boundary(true);
        s.set_txn_boundary(10).unwrap();
        assert!(s.fill(Some(10), None, 0x1).is_ok());
    }

    pub fn fill_writes_when_after_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.set_use_txn_boundary(true);
        s.set_txn_boundary(10).unwrap();
        s.fill(Some(10), None, 0x1).unwrap();
        assert!(s.is_filled(None, Some(10), 0x0).unwrap());
        assert!(s.is_filled(Some(10), None, 0x1).unwrap());
    }

    pub fn fill_returns_err_when_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
        assert_eq!(
            binary_storage::ERR_WRITE_PAST_END,
            s.fill(Some(9), Some(257), 0x1).unwrap_err().description()
        );
    }

    pub fn fill_does_not_write_when_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
        s.fill(Some(9), Some(257), 0x1).unwrap_err();
        assert!(s.is_filled(None, None, 0x0).unwrap());
    }

    pub fn fill_does_not_expand_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
        s.fill(Some(9), Some(257), 0x1).unwrap_err();
        assert_eq!(256, s.get_capacity().unwrap());
    }

    // is_filled() tests
    pub fn is_filled_retuns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.is_filled(None, None, 0x0).unwrap_err().description()
        );
    }

    pub fn is_filled_returns_err_when_start_offset_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.is_filled(Some(255), None, 0x0).unwrap());
        assert_eq!(
            binary_storage::ERR_READ_PAST_END,
            s.is_filled(Some(256), None, 0x0).unwrap_err().description()
        );
    }

    pub fn is_filled_returns_err_when_end_offset_at_or_before_start_offset<T: BinaryStorage>(
        mut s: T
    ) {
        s.open().unwrap();
        assert!(s.is_filled(Some(10), Some(11), 0x0).unwrap());
        assert_eq!(
            binary_storage::ERR_READ_NOTHING,
            s.is_filled(Some(10), Some(10), 0x0).unwrap_err().description()
        );
        assert_eq!(
            binary_storage::ERR_READ_NOTHING,
            s.is_filled(Some(10), Some(9), 0x0).unwrap_err().description()
        );
    }

    pub fn is_filled_returns_err_when_end_offset_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.is_filled(Some(10), Some(256), 0x0).unwrap());
        assert_eq!(
            binary_storage::ERR_READ_PAST_END,
            s.is_filled(Some(10), Some(257), 0x0).unwrap_err().description()
        );
    }

    pub fn is_filled_checks_whether_all_bytes_in_range_match_value<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.fill(Some(10), Some(20), 0x1).unwrap();
        assert!(s.is_filled(None, Some(10), 0x0).unwrap());
        assert!(!s.is_filled(None, Some(11), 0x0).unwrap());
        assert!(s.is_filled(Some(10), Some(20), 0x1).unwrap());
        assert!(!s.is_filled(Some(9), Some(20), 0x1).unwrap());
        assert!(!s.is_filled(Some(10), Some(21), 0x1).unwrap());
        assert!(s.is_filled(Some(20), None, 0x0).unwrap());
        assert!(!s.is_filled(Some(19), None, 0x0).unwrap());
    }

    pub fn is_filled_starts_from_start_offset<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.fill(Some(0), Some(10), 0x1).unwrap();
        assert!(s.is_filled(Some(10), None, 0x0).unwrap());
        assert!(!s.is_filled(Some(9), None, 0x0).unwrap());
    }

    pub fn is_filled_starts_from_beginning_when_start_offset_is_none<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.fill(Some(1), None, 0x1).unwrap();
        assert!(s.is_filled(None, Some(1), 0x0).unwrap());
        assert!(!s.is_filled(Some(1), Some(2), 0x0).unwrap());
    }

    pub fn is_filled_goes_to_end_offset<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.fill(Some(250), None, 0x1).unwrap();
        assert!(s.is_filled(None, Some(250), 0x0).unwrap());
        assert!(!s.is_filled(None, Some(251), 0x0).unwrap());
    }

    pub fn is_filled_goes_to_end_when_end_offset_is_none<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.fill(Some(255), None, 0x1).unwrap();
        assert!(s.is_filled(None, Some(255), 0x0).unwrap());
        assert!(!s.is_filled(None, None, 0x0).unwrap());
    }

    // get_use_txn_boundary(), set_use_txn_boundary(), get_txn_boundary(), and set_txn_boundary() tests
    pub fn set_use_txn_boundary_changes_value<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        assert!(s.get_use_txn_boundary());
        s.set_use_txn_boundary(false);
        assert!(!s.get_use_txn_boundary());
    }

    pub fn set_use_txn_boundary_resets_boundary_to_zero_when_txn_boundary_turned_off<T: BinaryStorage>(
        mut s: T
    ) {
        s.open().unwrap();
        s.set_use_txn_boundary(true);
        s.set_txn_boundary(10).unwrap();
        assert_eq!(10, s.get_txn_boundary().unwrap());
        s.set_use_txn_boundary(false);
        s.set_use_txn_boundary(true);
        assert_eq!(0, s.get_txn_boundary().unwrap());
    }

    pub fn get_txn_boundary_returns_err_when_closed<T: BinaryStorage>(s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.get_txn_boundary().unwrap_err().description()
        );
    }

    pub fn get_txn_boundary_returns_err_when_not_using_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.set_use_txn_boundary(false);
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_NOT_USING_TXN_BOUNDARY,
            s.get_txn_boundary().unwrap_err().description()
        );
    }

    pub fn get_txn_boundary_starts_at_0<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.set_use_txn_boundary(true);
        assert_eq!(0, s.get_txn_boundary().unwrap());
    }

    pub fn set_txn_boundary_returns_err_when_not_using_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.set_use_txn_boundary(false);
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_NOT_USING_TXN_BOUNDARY,
            s.set_txn_boundary(10).unwrap_err().description()
        );
    }

    pub fn set_txn_boundary_does_not_change_boundary_when_not_using_txn_boundary<T: BinaryStorage>(
        mut s: T
    ) {
        s.open().unwrap();
        s.set_use_txn_boundary(false);
        s.set_txn_boundary(10).unwrap_err();
        s.set_use_txn_boundary(true);
        assert_eq!(0, s.get_txn_boundary().unwrap());
    }

    pub fn set_txn_boundary_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.set_txn_boundary(10).unwrap_err().description()
        );
    }

    pub fn set_txn_boundary_does_not_change_boundary_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        s.set_txn_boundary(10).unwrap_err();
        s.open().unwrap();
        s.set_use_txn_boundary(true);
        assert_eq!(0, s.get_txn_boundary().unwrap());
    }

    pub fn set_txn_boundary_returns_err_when_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.set_use_txn_boundary(true);
        assert_eq!(
            binary_storage::ERR_SET_TXN_BOUNDARY_PAST_END,
            s.set_txn_boundary(257).unwrap_err().description()
        );

        assert!(s.set_txn_boundary(256).is_ok());
    }

    pub fn set_txn_boundary_does_not_change_boundary_when_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.set_use_txn_boundary(true);
        s.set_txn_boundary(257).unwrap_err();
        assert_eq!(0, s.get_txn_boundary().unwrap());
    }

    pub fn set_txn_boundary_does_not_expand_capacity_when_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.set_use_txn_boundary(true);
        assert_eq!(256, s.get_capacity().unwrap());
        s.set_txn_boundary(257).unwrap_err();
        assert_eq!(256, s.get_capacity().unwrap());
    }

    pub fn set_txn_boundary_changes_boundary<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.set_use_txn_boundary(true);
        s.set_txn_boundary(50).unwrap();
        assert_eq!(50, s.get_txn_boundary().unwrap());
        s.set_txn_boundary(25).unwrap();
        assert_eq!(25, s.get_txn_boundary().unwrap());
        s.set_txn_boundary(200).unwrap();
        assert_eq!(200, s.get_txn_boundary().unwrap());
    }

    // get_expand_size() and set_expand_size() tests
    pub fn get_expand_size_returns_initial_expand_size<T: BinaryStorage>(s: T) {
        assert_eq!(512, s.get_expand_size());
    }

    pub fn set_expand_size_returns_err_when_expand_size_is_zero<T: BinaryStorage>(mut s: T) {
        assert_eq!(
            binary_storage::ERR_EXPAND_SIZE_TOO_SMALL, 
            s.set_expand_size(0).unwrap_err().description()
        );
    }

    pub fn set_expand_size_does_not_change_expand_size_when_expand_size_is_zero<T: BinaryStorage>(mut s: T) {
        s.set_expand_size(0).unwrap_err();
        assert_eq!(512, s.get_expand_size());
    }

    pub fn set_expand_size_returns_err_when_expand_size_is_not_power_of_2<T: BinaryStorage>(mut s: T) {
        assert_eq!(
            binary_storage::ERR_EXPAND_SIZE_NOT_POWER_OF_2, 
            s.set_expand_size(513).unwrap_err().description()
        );
    }

    pub fn set_expand_size_does_not_change_expand_size_when_expand_size_is_not_power_of_2<T: BinaryStorage>(
        mut s: T
    ) {
        s.set_expand_size(513).unwrap_err();
        assert_eq!(512, s.get_expand_size());
    }

    pub fn set_expand_size_returns_true_when_checks_pass<T: BinaryStorage>(mut s: T) {
        assert!(s.set_expand_size(1024).is_ok());
    }

    pub fn set_expand_size_changes_expand_size_when_checks_pass<T: BinaryStorage>(mut s: T) {
        s.set_expand_size(1024).unwrap();
        assert_eq!(1024, s.get_expand_size());
    }

    pub fn capacity_increases_to_increments_of_last_set_expand_size<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_u8(256, 0x1).unwrap();
        /*
        assert_eq!(512, s.get_capacity().unwrap());
        s.set_expand_size(8).unwrap();
        s.w_u8(512, 0x1).unwrap();
        assert_eq!(520, s.get_capacity().unwrap());
        */
    }

    // get_capacity() tests
    pub fn get_capacity_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED, 
            s.get_capacity().unwrap_err().description()
        );
        s.open().unwrap();
        s.close().unwrap();
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED, 
            s.get_capacity().unwrap_err().description()
        );
    }

    pub fn get_capacity_returns_initial_capacity_when_open<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
    }

    pub fn get_capacity_returns_new_capacity_after_expansion<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.w_u8(256, 0x1).unwrap();
        assert_eq!(512, s.get_capacity().unwrap());
    }

    // expand() tests
    pub fn expand_returns_err_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.is_open());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            s.expand(10000).unwrap_err().description()
        );
    }

    pub fn expand_does_not_change_capacity_when_closed<T: BinaryStorage>(mut s: T) {
        s.expand(10000).unwrap_err();
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
    }

    pub fn expand_returns_ok_when_already_has_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.set_expand_size(16).unwrap();
        assert!(s.expand(50).is_ok());
    }

    pub fn expand_does_not_change_capacity_when_already_has_capacity<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.set_expand_size(16).unwrap();
        s.expand(50).unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
    }

    pub fn expand_returns_err_when_allocation_arithmetic_overflows<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(
            binary_storage::ERR_ARITHMETIC_OVERFLOW,
            s.expand(u64::max_value()).unwrap_err().description()
        );
    }

    pub fn expand_does_not_change_capacity_when_allocation_arithmetic_overflows<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.expand(u64::max_value()).unwrap_err();
        assert_eq!(256, s.get_capacity().unwrap());
    }

    pub fn expand_returns_err_when_allocation_fails<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert_eq!(
            binary_storage::ERR_STORAGE_ALLOC,
            s.expand(u64::max_value() - 1024).unwrap_err().description()
        );
    }

    pub fn expand_does_not_change_capacity_when_allocation_fails<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.expand(u64::max_value() - 1024).unwrap_err();
        assert_eq!(256, s.get_capacity().unwrap());
    }

    pub fn expand_returns_ok_when_successful<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        assert!(s.expand(300).is_ok());
    }

    pub fn expand_changes_capacity_by_expand_size_when_successful<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.expand(300).unwrap();
        assert_eq!(512, s.get_capacity().unwrap());
    }

    pub fn expand_changes_capacity_by_multiples_of_expand_size_when_successful<T: BinaryStorage>(mut s: T) {
        s.open().unwrap();
        s.expand(3000).unwrap();
        assert_eq!(3072, s.get_capacity().unwrap());
    }


}
