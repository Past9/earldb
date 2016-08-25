use std::str;
use std::error::Error;

use test::storage::binary_storage_tests;
use storage::binary_storage;
use storage::binary_storage::BinaryStorage;
use storage::memory_binary_storage::MemoryBinaryStorage;

// open(), close(), and is_open() tests 
#[test]
pub fn open_returns_err_when_already_open() {
    binary_storage_tests::open_returns_err_when_already_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
pub fn close_returns_err_when_already_closed() {
    binary_storage_tests::close_returns_err_when_already_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
pub fn open_returns_ok_when_previously_closed() {
    binary_storage_tests::open_returns_ok_when_previously_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
pub fn close_returns_ok_when_previously_open() {
    binary_storage_tests::close_returns_ok_when_previously_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn is_closed_when_new() {
    binary_storage_tests::is_closed_when_new(MemoryBinaryStorage::new(256, 256, false).unwrap());
}

#[test]
fn is_open_after_open() {
    binary_storage_tests::is_open_after_open(MemoryBinaryStorage::new(256, 256, false).unwrap());
}

#[test]
fn is_closed_after_open_and_close() {
    binary_storage_tests::is_closed_after_open_and_close(MemoryBinaryStorage::new(256, 256, false).unwrap());
}

// new() tests
#[test]
fn new_sets_initial_capacity() {
    let mut s = MemoryBinaryStorage::new(256, 512, false).unwrap();
    s.open().unwrap();
    assert_eq!(256, s.get_capacity().unwrap());
}

#[test]
fn new_sets_expand_size() {
    let s = MemoryBinaryStorage::new(256, 512, false).unwrap();
    assert_eq!(512, s.get_expand_size());
}

#[test]
fn new_sets_use_txn_boundary() {
    let s1 = MemoryBinaryStorage::new(256, 512, false).unwrap();
    assert!(!s1.get_use_txn_boundary());
    let s2 = MemoryBinaryStorage::new(256, 512, true).unwrap();
    assert!(s2.get_use_txn_boundary());
}

#[test]
fn new_requires_initial_capacity_greater_than_0() {
    let s = MemoryBinaryStorage::new(0, 512, false);
    assert!(s.is_err());
    assert_eq!(binary_storage::ERR_INITIAL_CAP_TOO_SMALL, s.unwrap_err().description());
}

#[test]
fn new_requires_expand_size_greater_than_0() {
    let s = MemoryBinaryStorage::new(256, 0, false);
    assert!(s.is_err());
    assert_eq!(binary_storage::ERR_EXPAND_SIZE_TOO_SMALL, s.unwrap_err().description());
}

#[test]
fn new_requires_initial_capacity_is_power_of_2() {
    let s1 = MemoryBinaryStorage::new(256, 512, false);
    assert!(s1.is_ok());

    let s2 = MemoryBinaryStorage::new(257, 512, false);
    assert!(s2.is_err());
    assert_eq!(binary_storage::ERR_INITIAL_CAP_NOT_POWER_OF_2, s2.unwrap_err().description());

    let s3 = MemoryBinaryStorage::new(384, 512, false);
    assert!(s3.is_err());
    assert_eq!(binary_storage::ERR_INITIAL_CAP_NOT_POWER_OF_2, s3.unwrap_err().description());

    let s4 = MemoryBinaryStorage::new(512, 512, false);
    assert!(s4.is_ok());
}

#[test]
fn new_requires_expand_size_is_power_of_2() {
    let s1 = MemoryBinaryStorage::new(256, 512, false);
    assert!(s1.is_ok());

    let s2 = MemoryBinaryStorage::new(256, 513, false);
    assert!(s2.is_err());
    assert_eq!(binary_storage::ERR_EXPAND_SIZE_NOT_POWER_OF_2, s2.unwrap_err().description());

    let s3 = MemoryBinaryStorage::new(256, 768, false);
    assert!(s3.is_err());
    assert_eq!(binary_storage::ERR_EXPAND_SIZE_NOT_POWER_OF_2, s3.unwrap_err().description());

    let s4 = MemoryBinaryStorage::new(256, 1024, false);
    assert!(s4.is_ok());
}

#[test]
fn new_initializes_memory_to_zeros() {
    let mut s = MemoryBinaryStorage::new(256, 512, false).unwrap();
    s.open().unwrap();
    assert!(s.is_filled(None, None, 0x0).unwrap());
}

// w_i8() tests
#[test]
fn w_i8_returns_err_when_closed() {
    binary_storage_tests::w_i8_returns_err_when_closed(MemoryBinaryStorage::new(256, 256, false).unwrap());
}

#[test]
fn w_i8_returns_ok_when_open() {
    binary_storage_tests::w_i8_returns_ok_when_open(MemoryBinaryStorage::new(256, 256, false).unwrap());
}

#[test]
fn w_i8_does_not_write_when_closed() {
    binary_storage_tests::w_i8_does_not_write_when_closed(MemoryBinaryStorage::new(256, 256, false).unwrap());
}

#[test]
fn w_i8_does_not_write_before_txn_boundary() {
    binary_storage_tests::w_i8_does_not_write_before_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_i8_over_capacity_expands_storage() {
    binary_storage_tests::w_i8_over_capacity_expands_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// w_i16() tests
#[test]
fn w_i16_returns_err_when_closed() {
    binary_storage_tests::w_i16_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_i16_returns_ok_when_open() {
    binary_storage_tests::w_i16_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_i16_does_not_write_when_closed() {
    binary_storage_tests::w_i16_does_not_write_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_i16_does_not_write_before_txn_boundary() {
    binary_storage_tests::w_i16_does_not_write_before_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_i16_over_capacity_expands_storage() {
    binary_storage_tests::w_i16_over_capacity_expands_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// w_i32() tests
#[test]
fn w_i32_returns_err_when_closed() {
    binary_storage_tests::w_i32_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_i32_returns_ok_when_open() {
    binary_storage_tests::w_i32_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_i32_does_not_write_when_closed() {
    binary_storage_tests::w_i32_does_not_write_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_i32_does_not_write_before_txn_boundary() {
    binary_storage_tests::w_i32_does_not_write_before_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_i32_over_capacity_expands_storage() {
    binary_storage_tests::w_i32_over_capacity_expands_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// w_i64() tests
#[test]
fn w_i64_returns_err_when_closed() {
    binary_storage_tests::w_i64_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_i64_returns_ok_when_open() {
    binary_storage_tests::w_i64_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_i64_does_not_write_when_closed() {
    binary_storage_tests::w_i64_does_not_write_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_i64_does_not_write_before_txn_boundary() {
    binary_storage_tests::w_i64_does_not_write_before_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_i64_over_capacity_expands_storage() {
    binary_storage_tests::w_i64_over_capacity_expands_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// w_u8() tests
#[test]
fn w_u8_returns_err_when_closed() {
    binary_storage_tests::w_u8_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_u8_returns_ok_when_open() {
    binary_storage_tests::w_u8_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_u8_does_not_write_when_closed() {
    binary_storage_tests::w_u8_does_not_write_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_u8_does_not_write_before_txn_boundary() {
    binary_storage_tests::w_u8_does_not_write_before_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_u8_over_capacity_expands_storage() {
    binary_storage_tests::w_u8_over_capacity_expands_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// w_u16() tests
#[test]
fn w_u16_returns_err_when_closed() {
    binary_storage_tests::w_u16_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_u16_returns_ok_when_open() {
    binary_storage_tests::w_u16_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_u16_does_not_write_when_closed() {
    binary_storage_tests::w_u16_does_not_write_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_u16_does_not_write_before_txn_boundary() {
    binary_storage_tests::w_u16_does_not_write_before_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_u16_over_capacity_expands_storage() {
    binary_storage_tests::w_u16_over_capacity_expands_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// w_u32() tests
#[test]
fn w_u32_returns_err_when_closed() {
    binary_storage_tests::w_u32_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_u32_returns_ok_when_open() {
    binary_storage_tests::w_u32_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_u32_does_not_write_when_closed() {
    binary_storage_tests::w_u32_does_not_write_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_u32_does_not_write_before_txn_boundary() {
    binary_storage_tests::w_u32_does_not_write_before_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_u32_over_capacity_expands_storage() {
    binary_storage_tests::w_u32_over_capacity_expands_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// w_u64() tests
#[test]
fn w_u64_returns_err_when_closed() {
    binary_storage_tests::w_u64_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_u64_returns_ok_when_open() {
    binary_storage_tests::w_u64_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_u64_does_not_write_when_closed() {
    binary_storage_tests::w_u64_does_not_write_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_u64_does_not_write_before_txn_boundary() {
    binary_storage_tests::w_u64_does_not_write_before_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_u64_over_capacity_expands_storage() {
    binary_storage_tests::w_u64_over_capacity_expands_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// w_f32() tests
#[test]
fn w_f32_returns_err_when_closed() {
    binary_storage_tests::w_f32_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_f32_returns_ok_when_open() {
    binary_storage_tests::w_f32_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_f32_does_not_write_when_closed() {
    binary_storage_tests::w_f32_does_not_write_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_f32_does_not_write_before_txn_boundary() {
    binary_storage_tests::w_f32_does_not_write_before_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_f32_over_capacity_expands_storage() {
    binary_storage_tests::w_f32_over_capacity_expands_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// w_f64() tests
#[test]
fn w_f64_returns_err_when_closed() {
    binary_storage_tests::w_f64_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_f64_returns_ok_when_open() {
    binary_storage_tests::w_f64_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_f64_does_not_write_when_closed() {
    binary_storage_tests::w_f64_does_not_write_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_f64_does_not_write_before_txn_boundary() {
    binary_storage_tests::w_f64_does_not_write_before_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_f64_over_capacity_expands_storage() {
    binary_storage_tests::w_f64_over_capacity_expands_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// w_bool() tests
#[test]
fn w_bool_returns_err_when_closed() {
    binary_storage_tests::w_bool_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_bool_returns_ok_when_open() {
    binary_storage_tests::w_bool_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_bool_does_not_write_when_closed() {
    binary_storage_tests::w_bool_does_not_write_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_bool_does_not_write_before_txn_boundary() {
    binary_storage_tests::w_bool_does_not_write_before_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_bool_over_capacity_expands_storage() {
    binary_storage_tests::w_bool_over_capacity_expands_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// w_bytes() tests
#[test]
fn w_bytes_returns_err_when_closed() {
    binary_storage_tests::w_bytes_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_bytes_returns_ok_when_open() {
    binary_storage_tests::w_bytes_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_bytes_does_not_write_when_closed() {
    binary_storage_tests::w_bytes_does_not_write_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_bytes_does_not_write_before_txn_boundary() {
    binary_storage_tests::w_bytes_does_not_write_before_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_bytes_over_capacity_expands_storage() {
    binary_storage_tests::w_bytes_over_capacity_expands_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_bytes_over_capacity_expands_storage_multiple_times() {
    binary_storage_tests::w_bytes_over_capacity_expands_storage_multiple_times(
        MemoryBinaryStorage::new(256, 4, false).unwrap()
    );
}

// w_str() tests
#[test]
fn w_str_returns_err_when_closed() {
    binary_storage_tests::w_str_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_str_returns_ok_when_open() {
    binary_storage_tests::w_str_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_str_does_not_write_when_closed() {
    binary_storage_tests::w_str_does_not_write_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_str_does_not_write_before_txn_boundary() {
    binary_storage_tests::w_str_does_not_write_before_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_str_over_capacity_expands_storage() {
    binary_storage_tests::w_str_over_capacity_expands_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn w_str_over_capacity_expands_storage_multiple_times() {
    binary_storage_tests::w_str_over_capacity_expands_storage_multiple_times(
        MemoryBinaryStorage::new(256, 4, false).unwrap()
    );
}

// r_i8() tests
#[test]
fn r_i8_returns_err_when_closed() {
    binary_storage_tests::r_i8_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i8_returns_ok_when_open() {
    binary_storage_tests::r_i8_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i8_reads_zero_from_unwritten_storage() {
    binary_storage_tests::r_i8_reads_zero_from_unwritten_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i8_reads_written_data() {
    binary_storage_tests::r_i8_reads_written_data(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i8_does_not_read_past_txn_boundary() {
    binary_storage_tests::r_i8_does_not_read_past_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i8_does_not_read_past_capacity() {
    binary_storage_tests::r_i8_does_not_read_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i8_result_is_not_mutated_on_subsequent_write() {
    binary_storage_tests::r_i8_result_is_not_mutated_on_subsequent_write(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// r_i16() tests
#[test]
fn r_i16_returns_err_when_closed() {
    binary_storage_tests::r_i16_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i16_returns_ok_when_open() {
    binary_storage_tests::r_i16_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i16_reads_zero_from_unwritten_storage() {
    binary_storage_tests::r_i16_reads_zero_from_unwritten_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i16_reads_written_data() {
    binary_storage_tests::r_i16_reads_written_data(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i16_does_not_read_past_txn_boundary() {
    binary_storage_tests::r_i16_does_not_read_past_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i16_does_not_read_past_capacity() {
    binary_storage_tests::r_i16_does_not_read_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i16_result_is_not_mutated_on_subsequent_write() {
    binary_storage_tests::r_i16_result_is_not_mutated_on_subsequent_write(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// r_i32() tests
#[test]
fn r_i32_returns_err_when_closed() {
    binary_storage_tests::r_i32_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i32_returns_ok_when_open() {
    binary_storage_tests::r_i32_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i32_reads_zero_from_unwritten_storage() {
    binary_storage_tests::r_i32_reads_zero_from_unwritten_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i32_reads_written_data() {
    binary_storage_tests::r_i32_reads_written_data(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i32_does_not_read_past_txn_boundary() {
    binary_storage_tests::r_i32_does_not_read_past_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i32_does_not_read_past_capacity() {
    binary_storage_tests::r_i32_does_not_read_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i32_result_is_not_mutated_on_subsequent_write() {
    binary_storage_tests::r_i32_result_is_not_mutated_on_subsequent_write(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// r_i64() tests
#[test]
fn r_i64_returns_err_when_closed() {
    binary_storage_tests::r_i64_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i64_returns_ok_when_open() {
    binary_storage_tests::r_i64_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i64_reads_zero_from_unwritten_storage() {
    binary_storage_tests::r_i64_reads_zero_from_unwritten_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i64_reads_written_data() {
    binary_storage_tests::r_i64_reads_written_data(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i64_does_not_read_past_txn_boundary() {
    binary_storage_tests::r_i64_does_not_read_past_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i64_does_not_read_past_capacity() {
    binary_storage_tests::r_i64_does_not_read_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_i64_result_is_not_mutated_on_subsequent_write() {
    binary_storage_tests::r_i64_result_is_not_mutated_on_subsequent_write(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// r_u8() tests
#[test]
fn r_u8_returns_err_when_closed() {
    binary_storage_tests::r_u8_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u8_returns_ok_when_open() {
    binary_storage_tests::r_u8_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u8_reads_zero_from_unwritten_storage() {
    binary_storage_tests::r_u8_reads_zero_from_unwritten_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u8_reads_written_data() {
    binary_storage_tests::r_u8_reads_written_data(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u8_does_not_read_past_txn_boundary() {
    binary_storage_tests::r_u8_does_not_read_past_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u8_does_not_read_past_capacity() {
    binary_storage_tests::r_u8_does_not_read_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u8_result_is_not_mutated_on_subsequent_write() {
    binary_storage_tests::r_u8_result_is_not_mutated_on_subsequent_write(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// r_u16() tests
#[test]
fn r_u16_returns_err_when_closed() {
    binary_storage_tests::r_u16_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u16_returns_ok_when_open() {
    binary_storage_tests::r_u16_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u16_reads_zero_from_unwritten_storage() {
    binary_storage_tests::r_u16_reads_zero_from_unwritten_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u16_reads_written_data() {
    binary_storage_tests::r_u16_reads_written_data(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u16_does_not_read_past_txn_boundary() {
    binary_storage_tests::r_u16_does_not_read_past_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u16_does_not_read_past_capacity() {
    binary_storage_tests::r_u16_does_not_read_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u16_result_is_not_mutated_on_subsequent_write() {
    binary_storage_tests::r_u16_result_is_not_mutated_on_subsequent_write(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// r_u32() tests
#[test]
fn r_u32_returns_err_when_closed() {
    binary_storage_tests::r_u32_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u32_returns_ok_when_open() {
    binary_storage_tests::r_u32_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u32_reads_zero_from_unwritten_storage() {
    binary_storage_tests::r_u32_reads_zero_from_unwritten_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u32_reads_written_data() {
    binary_storage_tests::r_u32_reads_written_data(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u32_does_not_read_past_txn_boundary() {
    binary_storage_tests::r_u32_does_not_read_past_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u32_does_not_read_past_capacity() {
    binary_storage_tests::r_u32_does_not_read_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u32_result_is_not_mutated_on_subsequent_write() {
    binary_storage_tests::r_u32_result_is_not_mutated_on_subsequent_write(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// r_u64() tests
#[test]
fn r_u64_returns_err_when_closed() {
    binary_storage_tests::r_u64_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u64_returns_ok_when_open() {
    binary_storage_tests::r_u64_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u64_reads_zero_from_unwritten_storage() {
    binary_storage_tests::r_u64_reads_zero_from_unwritten_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u64_reads_written_data() {
    binary_storage_tests::r_u64_reads_written_data(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u64_does_not_read_past_txn_boundary() {
    binary_storage_tests::r_u64_does_not_read_past_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u64_does_not_read_past_capacity() {
    binary_storage_tests::r_u64_does_not_read_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_u64_result_is_not_mutated_on_subsequent_write() {
    binary_storage_tests::r_u64_result_is_not_mutated_on_subsequent_write(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// r_f32() tests
#[test]
fn r_f32_returns_err_when_closed() {
    binary_storage_tests::r_f32_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_f32_returns_ok_when_open() {
    binary_storage_tests::r_f32_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_f32_reads_zero_from_unwritten_storage() {
    binary_storage_tests::r_f32_reads_zero_from_unwritten_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_f32_reads_written_data() {
    binary_storage_tests::r_f32_reads_written_data(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_f32_does_not_read_past_txn_boundary() {
    binary_storage_tests::r_f32_does_not_read_past_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_f32_does_not_read_past_capacity() {
    binary_storage_tests::r_f32_does_not_read_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_f32_result_is_not_mutated_on_subsequent_write() {
    binary_storage_tests::r_f32_result_is_not_mutated_on_subsequent_write(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// r_f64() tests
#[test]
fn r_f64_returns_err_when_closed() {
    binary_storage_tests::r_f64_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_f64_returns_ok_when_open() {
    binary_storage_tests::r_f64_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_f64_reads_zero_from_unwritten_storage() {
    binary_storage_tests::r_f64_reads_zero_from_unwritten_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_f64_reads_written_data() {
    binary_storage_tests::r_f64_reads_written_data(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_f64_does_not_read_past_txn_boundary() {
    binary_storage_tests::r_f64_does_not_read_past_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_f64_does_not_read_past_capacity() {
    binary_storage_tests::r_f64_does_not_read_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_f64_result_is_not_mutated_on_subsequent_write() {
    binary_storage_tests::r_f64_result_is_not_mutated_on_subsequent_write(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// r_bool() tests
#[test]
fn r_bool_returns_err_when_closed() {
    binary_storage_tests::r_bool_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_bool_returns_ok_when_open() {
    binary_storage_tests::r_bool_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_bool_reads_false_from_unwritten_storage() {
    binary_storage_tests::r_bool_reads_false_from_unwritten_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_bool_reads_written_data() {
    binary_storage_tests::r_bool_reads_written_data(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_bool_does_not_read_past_txn_boundary() {
    binary_storage_tests::r_bool_does_not_read_past_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_bool_does_not_read_past_capacity() {
    binary_storage_tests::r_bool_does_not_read_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_bool_result_is_not_mutated_on_subsequent_write() {
    binary_storage_tests::r_bool_result_is_not_mutated_on_subsequent_write(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// r_bytes() tests
#[test]
fn r_bytes_returns_err_when_closed() {
    binary_storage_tests::r_bytes_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_bytes_returns_ok_when_open() {
    binary_storage_tests::r_bytes_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_bytes_reads_zeros_from_unwritten_storage() {
    binary_storage_tests::r_bytes_reads_zeros_from_unwritten_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_bytes_reads_written_data() {
    binary_storage_tests::r_bytes_reads_written_data(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_bytes_does_not_read_past_txn_boundary() {
    binary_storage_tests::r_bytes_does_not_read_past_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_bytes_does_not_read_past_capacity() {
    binary_storage_tests::r_bytes_does_not_read_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_bytes_result_is_not_mutated_on_subsequent_write() {
    binary_storage_tests::r_bytes_result_is_not_mutated_on_subsequent_write(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// r_str() tests
#[test]
fn r_str_returns_err_when_closed() {
    binary_storage_tests::r_str_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_str_returns_ok_when_open() {
    binary_storage_tests::r_str_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_str_reads_nulls_from_unwritten_storage() {
    binary_storage_tests::r_str_reads_nulls_from_unwritten_storage(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_str_reads_written_data() {
    binary_storage_tests::r_str_reads_written_data(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_str_does_not_read_past_txn_boundary() {
    binary_storage_tests::r_str_does_not_read_past_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_str_does_not_read_past_capacity() {
    binary_storage_tests::r_str_does_not_read_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn r_str_result_is_not_mutated_on_subsequent_write() {
    binary_storage_tests::r_str_result_is_not_mutated_on_subsequent_write(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// fill() tests
#[test]
fn fill_returns_err_when_closed() {
    binary_storage_tests::fill_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn fill_does_not_write_when_closed() {
    binary_storage_tests::fill_does_not_write_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn fill_returns_ok_when_open() {
    binary_storage_tests::fill_returns_ok_when_open(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn fill_repeats_byte_in_storage_range() {
    binary_storage_tests::fill_repeats_byte_in_storage_range(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn fill_starts_from_beginning_when_start_offset_is_none() {
    binary_storage_tests::fill_starts_from_beginning_when_start_offset_is_none(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn fill_goes_to_end_when_end_offset_is_none() {
    binary_storage_tests::fill_goes_to_end_when_end_offset_is_none(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn fill_returns_err_when_end_offset_is_before_start_offset() {
    binary_storage_tests::fill_returns_err_when_end_offset_is_before_start_offset(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn fill_does_not_write_when_end_offset_is_before_start_offset() {
    binary_storage_tests::fill_does_not_write_when_end_offset_is_before_start_offset(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn fill_returns_err_when_before_txn_boundary() {
    binary_storage_tests::fill_returns_err_when_before_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn fill_does_not_write_when_before_txn_boundary() {
    binary_storage_tests::fill_does_not_write_when_before_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn fill_returns_ok_when_after_txn_boundary() {
    binary_storage_tests::fill_returns_ok_when_after_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn fill_writes_when_after_txn_boundary() {
    binary_storage_tests::fill_writes_when_after_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn fill_returns_err_when_past_capacity() {
    binary_storage_tests::fill_returns_err_when_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn fill_does_not_write_when_past_capacity() {
    binary_storage_tests::fill_does_not_write_when_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn fill_does_not_expand_capacity() {
    binary_storage_tests::fill_does_not_expand_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// assert_filled() tests
#[test]
fn is_filled_retuns_err_when_closed() {
    binary_storage_tests::is_filled_retuns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn is_filled_returns_err_when_start_offset_past_capacity() {
    binary_storage_tests::is_filled_returns_err_when_start_offset_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn is_filled_returns_err_when_end_offset_at_or_before_start_offset() {
    binary_storage_tests::is_filled_returns_err_when_end_offset_at_or_before_start_offset(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn is_filled_returns_err_when_end_offset_past_capacity() {
    binary_storage_tests::is_filled_returns_err_when_end_offset_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn is_filled_checks_whether_all_bytes_in_range_match_value() {
    binary_storage_tests::is_filled_checks_whether_all_bytes_in_range_match_value(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn is_filled_starts_from_start_offset() {
    binary_storage_tests::is_filled_starts_from_start_offset(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn is_filled_starts_from_beginning_when_start_offset_is_none() {
    binary_storage_tests::is_filled_starts_from_beginning_when_start_offset_is_none(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn is_filled_goes_to_end_offset() {
    binary_storage_tests::is_filled_goes_to_end_offset(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn is_filled_goes_to_end_when_end_offset_is_none() {
    binary_storage_tests::is_filled_goes_to_end_when_end_offset_is_none(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// get_use_txn_boundary(), set_use_txn_boundary(), get_txn_boundary(), and set_txn_boundary() tests
#[test]
fn set_use_txn_boundary_changes_value() {
    binary_storage_tests::set_use_txn_boundary_changes_value(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn set_use_txn_boundary_resets_boundary_to_zero_when_txn_boundary_turned_off() {
    binary_storage_tests::set_use_txn_boundary_resets_boundary_to_zero_when_txn_boundary_turned_off(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn get_txn_boundary_returns_err_when_closed() {
    binary_storage_tests::get_txn_boundary_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn get_txn_boundary_returns_err_when_not_using_txn_boundary() {
    binary_storage_tests::get_txn_boundary_returns_err_when_not_using_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn get_txn_boundary_starts_at_0() {
    binary_storage_tests::get_txn_boundary_starts_at_0(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn set_txn_boundary_returns_err_when_not_using_txn_boundary() {
    binary_storage_tests::set_txn_boundary_returns_err_when_not_using_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn set_txn_boundary_does_not_change_boundary_when_not_using_txn_boundary() {
    binary_storage_tests::set_txn_boundary_does_not_change_boundary_when_not_using_txn_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn set_txn_boundary_returns_err_when_closed() {
    binary_storage_tests::set_txn_boundary_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn set_txn_boundary_does_not_change_boundary_when_closed() {
    binary_storage_tests::set_txn_boundary_does_not_change_boundary_when_closed(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn set_txn_boundary_returns_err_when_past_capacity() {
    binary_storage_tests::set_txn_boundary_returns_err_when_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn set_txn_boundary_does_not_change_boundary_when_past_capacity() {
    binary_storage_tests::set_txn_boundary_does_not_change_boundary_when_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn set_txn_boundary_does_not_expand_capacity_when_past_capacity() {
    binary_storage_tests::set_txn_boundary_does_not_expand_capacity_when_past_capacity(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

#[test]
fn set_txn_boundary_changes_boundary() {
    binary_storage_tests::set_txn_boundary_changes_boundary(
        MemoryBinaryStorage::new(256, 256, false).unwrap()
    );
}

// get_expand_size() and set_expand_size() tests
#[test]
fn get_expand_size_returns_initial_expand_size() {
    binary_storage_tests::get_expand_size_returns_initial_expand_size(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn set_expand_size_returns_err_when_expand_size_is_zero() {
    binary_storage_tests::set_expand_size_returns_err_when_expand_size_is_zero(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn set_expand_size_does_not_change_expand_size_when_expand_size_is_zero() {
    binary_storage_tests::set_expand_size_does_not_change_expand_size_when_expand_size_is_zero(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn set_expand_size_returns_err_when_expand_size_is_not_power_of_2() {
    binary_storage_tests::set_expand_size_returns_err_when_expand_size_is_not_power_of_2(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn set_expand_size_does_not_change_expand_size_when_expand_size_is_not_power_of_2() {
    binary_storage_tests::set_expand_size_does_not_change_expand_size_when_expand_size_is_not_power_of_2(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn set_expand_size_returns_true_when_checks_pass() {
    binary_storage_tests::set_expand_size_returns_true_when_checks_pass(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn set_expand_size_changes_expand_size_when_checks_pass() {
    binary_storage_tests::set_expand_size_changes_expand_size_when_checks_pass(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn capacity_increases_to_increments_of_last_set_expand_size() {
    binary_storage_tests::capacity_increases_to_increments_of_last_set_expand_size(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

// get_capacity() tests
#[test]
fn get_capacity_returns_err_when_closed() {
    binary_storage_tests::get_capacity_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn get_capacity_returns_initial_capacity_when_open() {
    binary_storage_tests::get_capacity_returns_initial_capacity_when_open(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn get_capacity_returns_new_capacity_after_expansion() {
    binary_storage_tests::get_capacity_returns_new_capacity_after_expansion(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

// expand() tests
#[test]
fn expand_returns_err_when_closed() {
    binary_storage_tests::expand_returns_err_when_closed(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn expand_does_not_change_capacity_when_closed() {
    binary_storage_tests::expand_does_not_change_capacity_when_closed(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn expand_returns_ok_when_already_has_capacity() {
    binary_storage_tests::expand_returns_ok_when_already_has_capacity(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn expand_does_not_change_capacity_when_already_has_capacity() {
    binary_storage_tests::expand_does_not_change_capacity_when_already_has_capacity(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn expand_returns_err_when_allocation_arithmetic_overflows() {
    binary_storage_tests::expand_returns_err_when_allocation_arithmetic_overflows(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn expand_does_not_change_capacity_when_allocation_arithmetic_overflows() {
    binary_storage_tests::expand_does_not_change_capacity_when_allocation_arithmetic_overflows(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn expand_returns_err_when_allocation_fails() {
    binary_storage_tests::expand_returns_err_when_allocation_fails(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn expand_does_not_change_capacity_when_allocation_fails() {
    binary_storage_tests::expand_does_not_change_capacity_when_allocation_fails(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn expand_returns_ok_when_successful() {
    binary_storage_tests::expand_returns_ok_when_successful(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn expand_changes_capacity_by_expand_size_when_successful() {
    binary_storage_tests::expand_changes_capacity_by_expand_size_when_successful(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}

#[test]
fn expand_changes_capacity_by_multiples_of_expand_size_when_successful() {
    binary_storage_tests::expand_changes_capacity_by_multiples_of_expand_size_when_successful(
        MemoryBinaryStorage::new(256, 512, false).unwrap()
    );
}
