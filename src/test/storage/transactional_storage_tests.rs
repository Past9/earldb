use std::str;
use std::error::Error;
use storage::transactional_storage;
use storage::binary_storage::BinaryStorage;
use storage::transactional_storage::TransactionalStorage;
use storage::memory_binary_storage::MemoryBinaryStorage;


fn new_storage() -> TransactionalStorage<MemoryBinaryStorage> {
    TransactionalStorage::new(MemoryBinaryStorage::new(256, 256).unwrap())
}


// writer tests
#[test]
pub fn w_i8_does_not_write_before_txn_boundary() {
    let mut s = new_storage();
    s.open().unwrap();
    s.set_txn_boundary(4).unwrap();
    assert_eq!(
        transactional_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY,
        s.w_i8(3, i8::max_value()).unwrap_err().description()
    );
    assert!(s.w_i8(4, i8::max_value()).is_ok());
    s.set_txn_boundary(16).unwrap();
    assert_eq!(0, s.r_i8(3).unwrap());
    assert_eq!(i8::max_value(), s.r_i8(4).unwrap()); 
}

#[test]
pub fn w_i16_does_not_write_before_txn_boundary() {
    let mut s = new_storage();
    s.open().unwrap();
    s.set_txn_boundary(4).unwrap();
    assert_eq!(
        transactional_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY,
        s.w_i16(3, i16::max_value()).unwrap_err().description()
    );
    assert!(s.w_i16(4, i16::max_value()).is_ok());
    s.set_txn_boundary(16).unwrap();
    assert_eq!(0, s.r_i16(2).unwrap());
    assert_eq!(i16::max_value(), s.r_i16(4).unwrap()); 
}

#[test]
pub fn w_i32_does_not_write_before_txn_boundary() {
    let mut s = new_storage();
    s.open().unwrap();
    s.set_txn_boundary(8).unwrap();
    assert_eq!(
        transactional_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY,
        s.w_i32(3, i32::max_value()).unwrap_err().description()
    );
    assert!(s.w_i32(8, i32::max_value()).is_ok());
    s.set_txn_boundary(16).unwrap();
    assert_eq!(0, s.r_i32(2).unwrap());
    assert_eq!(i32::max_value(), s.r_i32(8).unwrap()); 
}

#[test]
pub fn w_i64_does_not_write_before_txn_boundary() {
    let mut s = new_storage();
    s.open().unwrap();
    s.set_txn_boundary(8).unwrap();
    assert_eq!(
        transactional_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY,
        s.w_i64(7, i64::max_value()).unwrap_err().description()
    );
    assert!(s.w_i64(8, i64::max_value()).is_ok());
    s.set_txn_boundary(16).unwrap();
    assert_eq!(0, s.r_i64(0).unwrap());
    assert_eq!(i64::max_value(), s.r_i64(8).unwrap()); 
}

#[test]
pub fn w_u8_does_not_write_before_txn_boundary() {
    let mut s = new_storage();
    s.open().unwrap();
    s.set_txn_boundary(4).unwrap();
    assert_eq!(
        transactional_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY,
        s.w_u8(3, u8::max_value()).unwrap_err().description()
    );
    assert!(s.w_u8(4, u8::max_value()).is_ok());
    s.set_txn_boundary(16).unwrap();
    assert_eq!(0, s.r_u8(3).unwrap());
    assert_eq!(u8::max_value(), s.r_u8(4).unwrap()); 
}

#[test]
pub fn w_u16_does_not_write_before_txn_boundary() {
    let mut s = new_storage();
    s.open().unwrap();
    s.set_txn_boundary(4).unwrap();
    assert_eq!(
        transactional_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY,
        s.w_u16(3, u16::max_value()).unwrap_err().description()
    );
    assert!(s.w_u16(4, u16::max_value()).is_ok());
    s.set_txn_boundary(16).unwrap();
    assert_eq!(0, s.r_u16(2).unwrap());
    assert_eq!(u16::max_value(), s.r_u16(4).unwrap()); 
}

#[test]
pub fn w_u32_does_not_write_before_txn_boundary() {
    let mut s = new_storage();
    s.open().unwrap();
    s.set_txn_boundary(8).unwrap();
    assert_eq!(
        transactional_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY,
        s.w_u32(3, u32::max_value()).unwrap_err().description()
    );
    assert!(s.w_u32(8, u32::max_value()).is_ok());
    s.set_txn_boundary(16).unwrap();
    assert_eq!(0, s.r_u32(2).unwrap());
    assert_eq!(u32::max_value(), s.r_u32(8).unwrap()); 
}

#[test]
pub fn w_u64_does_not_write_before_txn_boundary() {
    let mut s = new_storage();
    s.open().unwrap();
    s.set_txn_boundary(8).unwrap();
    assert_eq!(
        transactional_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY,
        s.w_u64(7, u64::max_value()).unwrap_err().description()
    );
    assert!(s.w_u64(8, u64::max_value()).is_ok());
    s.set_txn_boundary(16).unwrap();
    assert_eq!(0, s.r_u64(0).unwrap());
    assert_eq!(u64::max_value(), s.r_u64(8).unwrap()); 
}

#[test]
pub fn w_f32_does_not_write_before_txn_boundary() {
    let mut s = new_storage();
    s.open().unwrap();
    s.set_txn_boundary(8).unwrap();
    assert_eq!(
        transactional_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY,
        s.w_f32(7, 12345.6789).unwrap_err().description()
    );
    assert!(s.w_f32(8, 12345.6789).is_ok());
    s.set_txn_boundary(16).unwrap();
    assert_eq!(0.0, s.r_f32(0).unwrap());
    assert_eq!(12345.6789, s.r_f32(8).unwrap()); 
}

#[test]
pub fn w_f64_does_not_write_before_txn_boundary() {
    let mut s = new_storage();
    s.open().unwrap();
    s.set_txn_boundary(8).unwrap();
    assert_eq!(
        transactional_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY,
        s.w_f64(7, 12345.6789).unwrap_err().description()
    );
    assert!(s.w_f64(8, 12345.6789).is_ok());
    s.set_txn_boundary(16).unwrap();
    assert_eq!(0.0, s.r_f64(0).unwrap());
    assert_eq!(12345.6789, s.r_f64(8).unwrap()); 
}

#[test]
pub fn w_bool_does_not_write_before_txn_boundary() {
    let mut s = new_storage();
    s.open().unwrap();
    s.set_txn_boundary(4).unwrap();
    assert_eq!(
        transactional_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY,
        s.w_bool(3, true).unwrap_err().description()
    );
    assert!(s.w_bool(4, true).is_ok());
    s.set_txn_boundary(8).unwrap();
    assert_eq!(false, s.r_bool(3).unwrap());
    assert_eq!(true, s.r_bool(4).unwrap()); 
}

#[test]
pub fn w_bytes_does_not_write_before_txn_boundary() {
    let mut s = new_storage();
    s.open().unwrap();
    s.set_txn_boundary(8).unwrap();
    assert_eq!(
        transactional_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY,
        s.w_bytes(7, &[0x0, 0x1, 0x2, 0x3, 0x4]).unwrap_err().description()
    );
    assert!(s.w_bytes(8, &[0x0, 0x1, 0x2, 0x3, 0x4]).is_ok());
    s.set_txn_boundary(16).unwrap();
    assert_eq!(vec!(0x0, 0x0, 0x0, 0x0, 0x0), s.r_bytes(3, 5).unwrap());
    assert_eq!(vec!(0x0, 0x1, 0x2, 0x3, 0x4), s.r_bytes(8, 5).unwrap()); 
}

#[test]
pub fn w_str_does_not_write_before_txn_boundary() {
    let mut s = new_storage();
    s.open().unwrap();
    s.set_txn_boundary(16).unwrap();
    assert_eq!(
        transactional_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY,
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


// reader tests
#[test]
pub fn r_i8_does_not_read_past_txn_boundary() {
    let mut s = new_storage();
    s.open().unwrap();
    s.set_txn_boundary(4).unwrap();
    assert!(s.r_i8(3).is_ok());
    assert_eq!(
        transactional_storage::ERR_READ_AFTER_TXN_BOUNDARY,
        s.r_i8(4).unwrap_err().description()
    );
}

#[test]
pub fn r_i16_does_not_read_past_txn_boundary() {
    let mut s = new_storage();
    s.open().unwrap();
    s.set_txn_boundary(4).unwrap();
    assert!(s.r_i16(2).is_ok());
    assert_eq!(
        transactional_storage::ERR_READ_AFTER_TXN_BOUNDARY,
        s.r_i16(3).unwrap_err().description()
    );
}

#[test]
pub fn r_i32_does_not_read_past_txn_boundary() {
    let mut s = new_storage();
    s.open().unwrap();
    s.set_txn_boundary(8).unwrap();
    assert!(s.r_i32(4).is_ok());
    assert_eq!(
        transactional_storage::ERR_READ_AFTER_TXN_BOUNDARY,
        s.r_i32(5).unwrap_err().description()
    );
}

#[test]
pub fn r_i64_does_not_read_past_txn_boundary() {
    let mut s = new_storage();
    s.open().unwrap();
    s.set_txn_boundary(8).unwrap();
    assert!(s.r_i64(0).is_ok());
    assert_eq!(
        transactional_storage::ERR_READ_AFTER_TXN_BOUNDARY,
        s.r_i64(1).unwrap_err().description()
    );
}
