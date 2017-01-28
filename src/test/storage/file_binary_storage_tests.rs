use std::fs;
use std::fs::OpenOptions;
use std::path::Path;
use uuid::Uuid;

use error::Error;
use test::storage::binary_storage_tests;
use storage::binary_storage::BinaryStorage;
use storage::file_binary_storage::FileBinaryStorage;


pub static BASE_PATH: &'static str = "./test_data/storage/file_binary_storage/";

fn rnd_path() -> String {
    BASE_PATH.to_string() 
        + Uuid::new_v4().simple().to_string().as_str()
        + ".tmp"
}

fn rm_tmp(filename: String) {
    fs::remove_file(filename).unwrap()
}

fn get_storage() -> (FileBinaryStorage, String) {
    let path = rnd_path();
    let s = FileBinaryStorage::new(
        path.clone(),
        true,
        256,
        16, 
        16,
        512
    ).unwrap();
    (s, path)
}

fn get_storage_expand_size(expand_size: u64) -> (FileBinaryStorage, String) {
    let path = rnd_path();
    let s = FileBinaryStorage::new(
        path.clone(),
        true,
        256,
        16, 
        16,
        expand_size
    ).unwrap();
    (s, path)
}


// open(), close(), and is_open() tests 
#[test]
pub fn open_returns_err_when_already_open() {
    let (s, p) = get_storage();
    binary_storage_tests::open_returns_err_when_already_open(s);
    rm_tmp(p);
}

#[test]
pub fn open_creates_file_when_allowed_and_file_does_not_exist() {
    let path = rnd_path();
    assert!(!Path::new(path.clone().as_str()).exists());
    let mut s = FileBinaryStorage::new(
        path.clone(),
        true,
        256,
        16, 
        16,
        512
    ).unwrap();
    assert!(!Path::new(path.clone().as_str()).exists());
    s.open().unwrap();
    assert!(Path::new(path.clone().as_str()).exists());
    s.close().unwrap();
    assert!(Path::new(path.clone().as_str()).exists());
    rm_tmp(path);
}

#[test]
pub fn open_creates_file_with_initial_capacity() {
    let path = rnd_path();
    let mut s = FileBinaryStorage::new(
        path.clone(),
        true,
        256,
        16, 
        16,
        512
    ).unwrap();
    s.open().unwrap();
    s.close().unwrap();
    let f = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(path.clone()).unwrap();
    assert_eq!(256, f.metadata().unwrap().len());
    rm_tmp(path);
}

#[test]
pub fn open_returns_io_err_when_file_does_not_exist_and_creation_not_allowed() {
    let path = rnd_path();
    assert!(!Path::new(path.clone().as_str()).exists());
    let mut s = FileBinaryStorage::new(
        path.clone(),
        false,
        256,
        16, 
        16,
        512
    ).unwrap();
    assert!(
        match s.open().unwrap_err() {
            Error::Io(_) => true,
            _ => false
        }
    );
}

#[test]
pub fn open_does_not_open_when_file_does_not_exist_and_creation_not_allowed() {
    let path = rnd_path();
    assert!(!Path::new(path.clone().as_str()).exists());
    let mut s = FileBinaryStorage::new(
        path.clone(),
        false,
        256,
        16, 
        16,
        512
    ).unwrap();
    s.open().unwrap_err();
    assert!(!s.is_open());
}

#[test]
pub fn open_does_not_create_file_when_not_allowed_and_file_does_not_exist() {
    let path = rnd_path();
    assert!(!Path::new(path.clone().as_str()).exists());
    let mut s = FileBinaryStorage::new(
        path.clone(),
        false,
        256,
        16, 
        16,
        512
    ).unwrap();
    assert!(!Path::new(path.clone().as_str()).exists());
    s.open().unwrap_err();
    assert!(!Path::new(path.clone().as_str()).exists());
    assert!(!Path::new(path.clone().as_str()).exists());
}

#[test]
pub fn close_returns_err_when_already_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::close_returns_err_when_already_closed(s);
}

#[test]
pub fn open_returns_ok_when_previously_closed() {
    let (s, p) = get_storage();
    binary_storage_tests::open_returns_ok_when_previously_closed(s);
    rm_tmp(p);
}

#[test]
pub fn close_returns_ok_when_previously_open() {
    let (s, p) = get_storage();
    binary_storage_tests::close_returns_ok_when_previously_open(s);
    rm_tmp(p);
}

#[test]
fn is_closed_when_new() {
    let (s, _) = get_storage();
    binary_storage_tests::is_closed_when_new(s);
}

#[test]
fn is_open_after_open() {
    let (s, p) = get_storage();
    binary_storage_tests::is_open_after_open(s);
    rm_tmp(p);
}

#[test]
fn is_closed_after_open_and_close() {
    let (s, p) = get_storage();
    binary_storage_tests::is_closed_after_open_and_close(s);
    rm_tmp(p);
}

// new() tests
// TODO: Write these

// w_i8() tests
#[test]
fn w_i8_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::w_i8_returns_err_when_closed(s);
}

#[test]
fn w_i8_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::w_i8_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn w_i8_does_not_write_when_closed() {
    let (s, p) = get_storage();
    binary_storage_tests::w_i8_does_not_write_when_closed(s);
    rm_tmp(p);
}

#[test]
fn w_i8_over_capacity_expands_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::w_i8_over_capacity_expands_storage(s);
    rm_tmp(p);
}

// w_i16() tests
#[test]
fn w_i16_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::w_i16_returns_err_when_closed(s);
}

#[test]
fn w_i16_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::w_i16_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn w_i16_does_not_write_when_closed() {
    let (s, p) = get_storage();
    binary_storage_tests::w_i16_does_not_write_when_closed(s);
    rm_tmp(p);
}

#[test]
fn w_i16_over_capacity_expands_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::w_i16_over_capacity_expands_storage(s);
    rm_tmp(p);
}

// w_i32() tests
#[test]
fn w_i32_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::w_i32_returns_err_when_closed(s);
}

#[test]
fn w_i32_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::w_i32_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn w_i32_does_not_write_when_closed() {
    let (s, p) = get_storage();
    binary_storage_tests::w_i32_does_not_write_when_closed(s);
    rm_tmp(p);
}

#[test]
fn w_i32_over_capacity_expands_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::w_i32_over_capacity_expands_storage(s);
    rm_tmp(p);
}

// w_i64() tests
#[test]
fn w_i64_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::w_i64_returns_err_when_closed(s);
}

#[test]
fn w_i64_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::w_i64_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn w_i64_does_not_write_when_closed() {
    let (s, p) = get_storage();
    binary_storage_tests::w_i64_does_not_write_when_closed(s);
    rm_tmp(p);
}

#[test]
fn w_i64_over_capacity_expands_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::w_i64_over_capacity_expands_storage(s);
    rm_tmp(p);
}

// w_u8() tests
#[test]
fn w_u8_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::w_u8_returns_err_when_closed(s);
}

#[test]
fn w_u8_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::w_u8_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn w_u8_does_not_write_when_closed() {
    let (s, p) = get_storage();
    binary_storage_tests::w_u8_does_not_write_when_closed(s);
    rm_tmp(p);
}

#[test]
fn w_u8_over_capacity_expands_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::w_u8_over_capacity_expands_storage(s);
    rm_tmp(p);
}

// w_u16() tests
#[test]
fn w_u16_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::w_u16_returns_err_when_closed(s);
}

#[test]
fn w_u16_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::w_u16_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn w_u16_does_not_write_when_closed() {
    let (s, p) = get_storage();
    binary_storage_tests::w_u16_does_not_write_when_closed(s);
    rm_tmp(p);
}

#[test]
fn w_u16_over_capacity_expands_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::w_u16_over_capacity_expands_storage(s);
    rm_tmp(p);
}

// w_u32() tests
#[test]
fn w_u32_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::w_u32_returns_err_when_closed(s);
}

#[test]
fn w_u32_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::w_u32_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn w_u32_does_not_write_when_closed() {
    let (s, p) = get_storage();
    binary_storage_tests::w_u32_does_not_write_when_closed(s);
    rm_tmp(p);
}

#[test]
fn w_u32_over_capacity_expands_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::w_u32_over_capacity_expands_storage(s);
    rm_tmp(p);
}

// w_u64() tests
#[test]
fn w_u64_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::w_u64_returns_err_when_closed(s);
}

#[test]
fn w_u64_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::w_u64_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn w_u64_does_not_write_when_closed() {
    let (s, p) = get_storage();
    binary_storage_tests::w_u64_does_not_write_when_closed(s);
    rm_tmp(p);
}

#[test]
fn w_u64_over_capacity_expands_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::w_u64_over_capacity_expands_storage(s);
    rm_tmp(p);
}

// w_f32() tests
#[test]
fn w_f32_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::w_f32_returns_err_when_closed(s);
}

#[test]
fn w_f32_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::w_f32_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn w_f32_does_not_write_when_closed() {
    let (s, p) = get_storage();
    binary_storage_tests::w_f32_does_not_write_when_closed(s);
    rm_tmp(p);
}

#[test]
fn w_f32_over_capacity_expands_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::w_f32_over_capacity_expands_storage(s);
    rm_tmp(p);
}

// w_f64() tests
#[test]
fn w_f64_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::w_f64_returns_err_when_closed(s);
}

#[test]
fn w_f64_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::w_f64_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn w_f64_does_not_write_when_closed() {
    let (s, p) = get_storage();
    binary_storage_tests::w_f64_does_not_write_when_closed(s);
    rm_tmp(p);
}

#[test]
fn w_f64_over_capacity_expands_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::w_f64_over_capacity_expands_storage(s);
    rm_tmp(p);
}

// w_bool() tests
#[test]
fn w_bool_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::w_bool_returns_err_when_closed(s);
}

#[test]
fn w_bool_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::w_bool_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn w_bool_does_not_write_when_closed() {
    let (s, p) = get_storage();
    binary_storage_tests::w_bool_does_not_write_when_closed(s);
    rm_tmp(p);
}

#[test]
fn w_bool_over_capacity_expands_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::w_bool_over_capacity_expands_storage(s);
    rm_tmp(p);
}

// w_bytes() tests
#[test]
fn w_bytes_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::w_bytes_returns_err_when_closed(s);
}

#[test]
fn w_bytes_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::w_bytes_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn w_bytes_does_not_write_when_closed() {
    let (s, p) = get_storage();
    binary_storage_tests::w_bytes_does_not_write_when_closed(s);
    rm_tmp(p);
}

#[test]
fn w_bytes_over_capacity_expands_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::w_bytes_over_capacity_expands_storage(s);
    rm_tmp(p);
}

#[test]
fn w_bytes_over_capacity_expands_storage_multiple_times() {
    let (s, p) = get_storage_expand_size(4);
    binary_storage_tests::w_bytes_over_capacity_expands_storage_multiple_times(s);
    rm_tmp(p);
}

// w_str() tests
#[test]
fn w_str_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::w_str_returns_err_when_closed(s);
}

#[test]
fn w_str_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::w_str_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn w_str_does_not_write_when_closed() {
    let (s, p) = get_storage();
    binary_storage_tests::w_str_does_not_write_when_closed(s);
    rm_tmp(p);
}

#[test]
fn w_str_over_capacity_expands_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::w_str_over_capacity_expands_storage(s);
    rm_tmp(p);
}

#[test]
fn w_str_over_capacity_expands_storage_multiple_times() {
    let (s, p) = get_storage_expand_size(4);
    binary_storage_tests::w_str_over_capacity_expands_storage_multiple_times(s);
    rm_tmp(p);
}

// r_i8() tests
#[test]
fn r_i8_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::r_i8_returns_err_when_closed(s);
}

#[test]
fn r_i8_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i8_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn r_i8_reads_zero_from_unwritten_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i8_reads_zero_from_unwritten_storage(s);
    rm_tmp(p);
}

#[test]
fn r_i8_reads_written_data() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i8_reads_written_data(s);
    rm_tmp(p);
}

#[test]
fn r_i8_does_not_read_past_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i8_does_not_read_past_capacity(s);
    rm_tmp(p);
}

#[test]
fn r_i8_result_is_not_mutated_on_subsequent_write() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i8_result_is_not_mutated_on_subsequent_write(s);
    rm_tmp(p);
}

// r_i16() tests
#[test]
fn r_i16_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::r_i16_returns_err_when_closed(s);
}

#[test]
fn r_i16_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i16_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn r_i16_reads_zero_from_unwritten_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i16_reads_zero_from_unwritten_storage(s);
    rm_tmp(p);
}

#[test]
fn r_i16_reads_written_data() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i16_reads_written_data(s);
    rm_tmp(p);
}

#[test]
fn r_i16_does_not_read_past_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i16_does_not_read_past_capacity(s);
    rm_tmp(p);
}

#[test]
fn r_i16_result_is_not_mutated_on_subsequent_write() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i16_result_is_not_mutated_on_subsequent_write(s);
    rm_tmp(p);
}

// r_i32() tests
#[test]
fn r_i32_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::r_i32_returns_err_when_closed(s);
}

#[test]
fn r_i32_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i32_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn r_i32_reads_zero_from_unwritten_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i32_reads_zero_from_unwritten_storage(s);
    rm_tmp(p);
}

#[test]
fn r_i32_reads_written_data() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i32_reads_written_data(s);
    rm_tmp(p);
}

#[test]
fn r_i32_does_not_read_past_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i32_does_not_read_past_capacity(s);
    rm_tmp(p);
}

#[test]
fn r_i32_result_is_not_mutated_on_subsequent_write() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i32_result_is_not_mutated_on_subsequent_write(s);
    rm_tmp(p);
}

// r_i64() tests
#[test]
fn r_i64_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::r_i64_returns_err_when_closed(s);
}

#[test]
fn r_i64_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i64_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn r_i64_reads_zero_from_unwritten_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i64_reads_zero_from_unwritten_storage(s);
    rm_tmp(p);
}

#[test]
fn r_i64_reads_written_data() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i64_reads_written_data(s);
    rm_tmp(p);
}

#[test]
fn r_i64_does_not_read_past_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i64_does_not_read_past_capacity(s);
    rm_tmp(p);
}

#[test]
fn r_i64_result_is_not_mutated_on_subsequent_write() {
    let (s, p) = get_storage();
    binary_storage_tests::r_i64_result_is_not_mutated_on_subsequent_write(s);
    rm_tmp(p);
}

// r_u8() tests
#[test]
fn r_u8_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::r_u8_returns_err_when_closed(s);
}

#[test]
fn r_u8_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u8_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn r_u8_reads_zero_from_unwritten_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u8_reads_zero_from_unwritten_storage(s);
    rm_tmp(p);
}

#[test]
fn r_u8_reads_written_data() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u8_reads_written_data(s);
    rm_tmp(p);
}

#[test]
fn r_u8_does_not_read_past_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u8_does_not_read_past_capacity(s);
    rm_tmp(p);
}

#[test]
fn r_u8_result_is_not_mutated_on_subsequent_write() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u8_result_is_not_mutated_on_subsequent_write(s);
    rm_tmp(p);
}

// r_u16() tests
#[test]
fn r_u16_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::r_u16_returns_err_when_closed(s);
}

#[test]
fn r_u16_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u16_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn r_u16_reads_zero_from_unwritten_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u16_reads_zero_from_unwritten_storage(s);
    rm_tmp(p);
}

#[test]
fn r_u16_reads_written_data() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u16_reads_written_data(s);
    rm_tmp(p);
}

#[test]
fn r_u16_does_not_read_past_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u16_does_not_read_past_capacity(s);
    rm_tmp(p);
}

#[test]
fn r_u16_result_is_not_mutated_on_subsequent_write() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u16_result_is_not_mutated_on_subsequent_write(s);
    rm_tmp(p);
}

// r_u32() tests
#[test]
fn r_u32_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::r_u32_returns_err_when_closed(s);
}

#[test]
fn r_u32_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u32_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn r_u32_reads_zero_from_unwritten_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u32_reads_zero_from_unwritten_storage(s);
    rm_tmp(p);
}

#[test]
fn r_u32_reads_written_data() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u32_reads_written_data(s);
    rm_tmp(p);
}

#[test]
fn r_u32_does_not_read_past_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u32_does_not_read_past_capacity(s);
    rm_tmp(p);
}

#[test]
fn r_u32_result_is_not_mutated_on_subsequent_write() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u32_result_is_not_mutated_on_subsequent_write(s);
    rm_tmp(p);
}

// r_u64() tests
#[test]
fn r_u64_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::r_u64_returns_err_when_closed(s);
}

#[test]
fn r_u64_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u64_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn r_u64_reads_zero_from_unwritten_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u64_reads_zero_from_unwritten_storage(s);
    rm_tmp(p);
}

#[test]
fn r_u64_reads_written_data() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u64_reads_written_data(s);
    rm_tmp(p);
}

#[test]
fn r_u64_does_not_read_past_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u64_does_not_read_past_capacity(s);
    rm_tmp(p);
}

#[test]
fn r_u64_result_is_not_mutated_on_subsequent_write() {
    let (s, p) = get_storage();
    binary_storage_tests::r_u64_result_is_not_mutated_on_subsequent_write(s);
    rm_tmp(p);
}

// r_f32() tests
#[test]
fn r_f32_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::r_f32_returns_err_when_closed(s);
}

#[test]
fn r_f32_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::r_f32_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn r_f32_reads_zero_from_unwritten_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::r_f32_reads_zero_from_unwritten_storage(s);
    rm_tmp(p);
}

#[test]
fn r_f32_reads_written_data() {
    let (s, p) = get_storage();
    binary_storage_tests::r_f32_reads_written_data(s);
    rm_tmp(p);
}

#[test]
fn r_f32_does_not_read_past_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::r_f32_does_not_read_past_capacity(s);
    rm_tmp(p);
}

#[test]
fn r_f32_result_is_not_mutated_on_subsequent_write() {
    let (s, p) = get_storage();
    binary_storage_tests::r_f32_result_is_not_mutated_on_subsequent_write(s);
    rm_tmp(p);
}

// r_f64() tests
#[test]
fn r_f64_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::r_f64_returns_err_when_closed(s);
}

#[test]
fn r_f64_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::r_f64_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn r_f64_reads_zero_from_unwritten_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::r_f64_reads_zero_from_unwritten_storage(s);
    rm_tmp(p);
}

#[test]
fn r_f64_reads_written_data() {
    let (s, p) = get_storage();
    binary_storage_tests::r_f64_reads_written_data(s);
    rm_tmp(p);
}

#[test]
fn r_f64_does_not_read_past_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::r_f64_does_not_read_past_capacity(s);
    rm_tmp(p);
}

#[test]
fn r_f64_result_is_not_mutated_on_subsequent_write() {
    let (s, p) = get_storage();
    binary_storage_tests::r_f64_result_is_not_mutated_on_subsequent_write(s);
    rm_tmp(p);
}

// r_bool() tests
#[test]
fn r_bool_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::r_bool_returns_err_when_closed(s);
}

#[test]
fn r_bool_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::r_bool_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn r_bool_reads_false_from_unwritten_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::r_bool_reads_false_from_unwritten_storage(s);
    rm_tmp(p);
}

#[test]
fn r_bool_reads_written_data() {
    let (s, p) = get_storage();
    binary_storage_tests::r_bool_reads_written_data(s);
    rm_tmp(p);
}

#[test]
fn r_bool_does_not_read_past_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::r_bool_does_not_read_past_capacity(s);
    rm_tmp(p);
}

#[test]
fn r_bool_result_is_not_mutated_on_subsequent_write() {
    let (s, p) = get_storage();
    binary_storage_tests::r_bool_result_is_not_mutated_on_subsequent_write(s);
    rm_tmp(p);
}

// r_bytes() tests
#[test]
fn r_bytes_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::r_bytes_returns_err_when_closed(s);
}

#[test]
fn r_bytes_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::r_bytes_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn r_bytes_reads_zeros_from_unwritten_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::r_bytes_reads_zeros_from_unwritten_storage(s);
    rm_tmp(p);
}

#[test]
fn r_bytes_reads_written_data() {
    let (s, p) = get_storage();
    binary_storage_tests::r_bytes_reads_written_data(s);
    rm_tmp(p);
}

#[test]
fn r_bytes_does_not_read_past_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::r_bytes_does_not_read_past_capacity(s);
    rm_tmp(p);
}

#[test]
fn r_bytes_result_is_not_mutated_on_subsequent_write() {
    let (s, p) = get_storage();
    binary_storage_tests::r_bytes_result_is_not_mutated_on_subsequent_write(s);
    rm_tmp(p);
}

// r_str() tests
#[test]
fn r_str_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::r_str_returns_err_when_closed(s);
}

#[test]
fn r_str_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::r_str_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn r_str_reads_nulls_from_unwritten_storage() {
    let (s, p) = get_storage();
    binary_storage_tests::r_str_reads_nulls_from_unwritten_storage(s);
    rm_tmp(p);
}

#[test]
fn r_str_reads_written_data() {
    let (s, p) = get_storage();
    binary_storage_tests::r_str_reads_written_data(s);
    rm_tmp(p);
}

#[test]
fn r_str_does_not_read_past_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::r_str_does_not_read_past_capacity(s);
    rm_tmp(p);
}

#[test]
fn r_str_result_is_not_mutated_on_subsequent_write() {
    let (s, p) = get_storage();
    binary_storage_tests::r_str_result_is_not_mutated_on_subsequent_write(s);
    rm_tmp(p);
}

// fill() tests
#[test]
fn fill_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::fill_returns_err_when_closed(s);
}

#[test]
fn fill_does_not_write_when_closed() {
    let (s, p) = get_storage();
    binary_storage_tests::fill_does_not_write_when_closed(s);
    rm_tmp(p);
}

#[test]
fn fill_returns_ok_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::fill_returns_ok_when_open(s);
    rm_tmp(p);
}

#[test]
fn fill_repeats_byte_in_storage_range() {
    let (s, p) = get_storage();
    binary_storage_tests::fill_repeats_byte_in_storage_range(s);
    rm_tmp(p);
}

#[test]
fn fill_starts_from_beginning_when_start_offset_is_none() {
    let (s, p) = get_storage();
    binary_storage_tests::fill_starts_from_beginning_when_start_offset_is_none(s);
    rm_tmp(p);
}

#[test]
fn fill_goes_to_end_when_end_offset_is_none() {
    let (s, p) = get_storage();
    binary_storage_tests::fill_goes_to_end_when_end_offset_is_none(s);
    rm_tmp(p);
}

#[test]
fn fill_returns_err_when_end_offset_is_before_start_offset() {
    let (s, p) = get_storage();
    binary_storage_tests::fill_returns_err_when_end_offset_is_before_start_offset(s);
    rm_tmp(p);
}

#[test]
fn fill_does_not_write_when_end_offset_is_before_start_offset() {
    let (s, p) = get_storage();
    binary_storage_tests::fill_does_not_write_when_end_offset_is_before_start_offset(s);
    rm_tmp(p);
}

#[test]
fn fill_returns_err_when_past_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::fill_returns_err_when_past_capacity(s);
    rm_tmp(p);
}

#[test]
fn fill_does_not_write_when_past_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::fill_does_not_write_when_past_capacity(s);
    rm_tmp(p);
}

#[test]
fn fill_does_not_expand_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::fill_does_not_expand_capacity(s);
    rm_tmp(p);
}

// assert_filled() tests
#[test]
fn is_filled_retuns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::is_filled_retuns_err_when_closed(s);
}

#[test]
fn is_filled_returns_err_when_start_offset_past_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::is_filled_returns_err_when_start_offset_past_capacity(s);
    rm_tmp(p);
}

#[test]
fn is_filled_returns_err_when_end_offset_at_or_before_start_offset() {
    let (s, p) = get_storage();
    binary_storage_tests::is_filled_returns_err_when_end_offset_at_or_before_start_offset(s);
    rm_tmp(p);
}

#[test]
fn is_filled_returns_err_when_end_offset_past_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::is_filled_returns_err_when_end_offset_past_capacity(s);
    rm_tmp(p);
}

#[test]
fn is_filled_checks_whether_all_bytes_in_range_match_value() {
    let (s, p) = get_storage();
    binary_storage_tests::is_filled_checks_whether_all_bytes_in_range_match_value(s);
    rm_tmp(p);
}

#[test]
fn is_filled_starts_from_start_offset() {
    let (s, p) = get_storage();
    binary_storage_tests::is_filled_starts_from_start_offset(s);
    rm_tmp(p);
}

#[test]
fn is_filled_starts_from_beginning_when_start_offset_is_none() {
    let (s, p) = get_storage();
    binary_storage_tests::is_filled_starts_from_beginning_when_start_offset_is_none(s);
    rm_tmp(p);
}

#[test]
fn is_filled_goes_to_end_offset() {
    let (s, p) = get_storage();
    binary_storage_tests::is_filled_goes_to_end_offset(s);
    rm_tmp(p);
}

#[test]
fn is_filled_goes_to_end_when_end_offset_is_none() {
    let (s, p) = get_storage();
    binary_storage_tests::is_filled_goes_to_end_when_end_offset_is_none(s);
    rm_tmp(p);
}

// get_expand_size() and set_expand_size() tests
#[test]
fn get_expand_size_returns_initial_expand_size() {
    let (s, _) = get_storage();
    binary_storage_tests::get_expand_size_returns_initial_expand_size(s);
}

#[test]
fn set_expand_size_returns_err_when_expand_size_is_zero() {
    let (s, _) = get_storage();
    binary_storage_tests::set_expand_size_returns_err_when_expand_size_is_zero(s);
}

#[test]
fn set_expand_size_does_not_change_expand_size_when_expand_size_is_zero() {
    let (s, _) = get_storage();
    binary_storage_tests::set_expand_size_does_not_change_expand_size_when_expand_size_is_zero(s);
}

#[test]
fn set_expand_size_returns_err_when_expand_size_is_not_power_of_2() {
    let (s, _) = get_storage();
    binary_storage_tests::set_expand_size_returns_err_when_expand_size_is_not_power_of_2(s);
}

#[test]
fn set_expand_size_does_not_change_expand_size_when_expand_size_is_not_power_of_2() {
    let (s, _) = get_storage();
    binary_storage_tests::set_expand_size_does_not_change_expand_size_when_expand_size_is_not_power_of_2(s);
}

#[test]
fn set_expand_size_returns_true_when_checks_pass() {
    let (s, _) = get_storage();
    binary_storage_tests::set_expand_size_returns_true_when_checks_pass(s);
}

#[test]
fn set_expand_size_changes_expand_size_when_checks_pass() {
    let (s, _) = get_storage();
    binary_storage_tests::set_expand_size_changes_expand_size_when_checks_pass(s);
}

#[test]
fn capacity_increases_to_increments_of_last_set_expand_size() {
    let (s, p) = get_storage();
    binary_storage_tests::capacity_increases_to_increments_of_last_set_expand_size(s);
    rm_tmp(p);
}

// get_capacity() tests
#[test]
fn get_capacity_returns_err_when_closed() {
    let (s, p) = get_storage();
    binary_storage_tests::get_capacity_returns_err_when_closed(s);
    rm_tmp(p);
}

#[test]
fn get_capacity_returns_initial_capacity_when_open() {
    let (s, p) = get_storage();
    binary_storage_tests::get_capacity_returns_initial_capacity_when_open(s);
    rm_tmp(p);
}

#[test]
fn get_capacity_returns_new_capacity_after_expansion() {
    let (s, p) = get_storage();
    binary_storage_tests::get_capacity_returns_new_capacity_after_expansion(s);
    rm_tmp(p);
}

// expand() tests
#[test]
fn expand_returns_err_when_closed() {
    let (s, _) = get_storage();
    binary_storage_tests::expand_returns_err_when_closed(s);
}

#[test]
fn expand_does_not_change_capacity_when_closed() {
    let (s, p) = get_storage();
    binary_storage_tests::expand_does_not_change_capacity_when_closed(s);
    rm_tmp(p);
}

#[test]
fn expand_returns_ok_when_already_has_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::expand_returns_ok_when_already_has_capacity(s);
    rm_tmp(p);
}

#[test]
fn expand_does_not_change_capacity_when_already_has_capacity() {
    let (s, p) = get_storage();
    binary_storage_tests::expand_does_not_change_capacity_when_already_has_capacity(s);
    rm_tmp(p);
}

#[test]
fn expand_returns_err_when_allocation_arithmetic_overflows() {
    let (s, p) = get_storage();
    binary_storage_tests::expand_returns_err_when_allocation_arithmetic_overflows(s);
    rm_tmp(p);
}

#[test]
fn expand_does_not_change_capacity_when_allocation_arithmetic_overflows() {
    let (s, p) = get_storage();
    binary_storage_tests::expand_does_not_change_capacity_when_allocation_arithmetic_overflows(s);
    rm_tmp(p);
}

#[test]
fn expand_returns_err_when_allocation_fails() {
    let (s, p) = get_storage();
    binary_storage_tests::expand_returns_err_when_allocation_fails(s);
    rm_tmp(p);
}

#[test]
fn expand_does_not_change_capacity_when_allocation_fails() {
    let (s, p) = get_storage();
    binary_storage_tests::expand_does_not_change_capacity_when_allocation_fails(s);
    rm_tmp(p);
}

#[test]
fn expand_returns_ok_when_successful() {
    let (s, p) = get_storage();
    binary_storage_tests::expand_returns_ok_when_successful(s);
    rm_tmp(p);
}

#[test]
fn expand_changes_capacity_by_expand_size_when_successful() {
    let (s, p) = get_storage();
    binary_storage_tests::expand_changes_capacity_by_expand_size_when_successful(s);
    rm_tmp(p);
}

#[test]
fn expand_changes_capacity_by_multiples_of_expand_size_when_successful() {
    let (s, p) = get_storage();
    binary_storage_tests::expand_changes_capacity_by_multiples_of_expand_size_when_successful(s);
    rm_tmp(p);
}
