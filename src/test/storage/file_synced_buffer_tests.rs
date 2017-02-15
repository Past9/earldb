use std::str;
use std::fs;
use std::fs::{ File, OpenOptions };
use std::io::Write;

use uuid::Uuid;

use storage::file_synced_buffer::FileSyncedBuffer;


pub static BASE_PATH: &'static str = "./test_data/storage/file_synced_buffer/";

fn rnd_path() -> String {
BASE_PATH.to_string() 
  + Uuid::new_v4().simple().to_string().as_str()
  + ".tmp"
}

fn path(filename: &str) -> String {
  BASE_PATH.to_string() + filename
}

fn file_r(filename: &str) -> File {
  OpenOptions::new()
    .read(true)
    .open(path(filename))
    .unwrap()
}

fn file_tmp_rw() -> (File, String) {
  let path = rnd_path();
  let file = OpenOptions::new()
    .write(true)
    .create(true)
    .read(true)
    .open(path.clone())
    .unwrap();
  (file, path)
}

fn rm_tmp(filename: String) {
  fs::remove_file(filename).unwrap()
}

// read() tests
#[test]
fn read_returns_empty_on_blank_file() {
  let mut b = FileSyncedBuffer::new(file_r("blank.txt"), 16, 16);
  assert_eq!(0, b.read(0, 128).unwrap().len());
}

#[test]
fn read_returns_empty_when_reading_from_past_eof() {
  let mut b = FileSyncedBuffer::new(file_r("10.txt"), 16, 16);
  assert_eq!(0, b.read(10, 10).unwrap().len());
}

#[test]
fn read_only_returns_data_present_in_file() {
  let mut b = FileSyncedBuffer::new(file_r("10.txt"), 4, 4);
  let res = b.read(0, 100).unwrap();
  assert_eq!(10, res.len());
  assert_eq!("Lorem ips\n", str::from_utf8(res.as_slice()).unwrap());
}

#[test]
fn read_truncates_data_when_reading_past_eof() {
  let mut b = FileSyncedBuffer::new(file_r("10.txt"), 16, 16);
  assert_eq!(10, b.read(0, 16).unwrap().len());
}

#[test]
fn read_reads_data_in_single_page() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16);
  let res = b.read(35, 10).unwrap();
  assert_eq!(10, res.len());
  assert_eq!("etur adipi", str::from_utf8(res.as_slice()).unwrap());
}

#[test]
fn read_reads_data_across_page_boundaries() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16);
  let res = b.read(25, 10).unwrap();
  assert_eq!(10, res.len());
  assert_eq!("t, consect", str::from_utf8(res.as_slice()).unwrap());
}

#[test]
fn read_reads_data_across_multiple_page_boundaries() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16);
  let res = b.read(40, 35).unwrap();
  assert_eq!(35, res.len());
  assert_eq!(
    "adipiscing elit. Integer ut imperdi", 
    str::from_utf8(res.as_slice()).unwrap()
  );
}

// update() tests
#[test]
fn update_writes_to_subset_of_first_page() {
  let (mut f, p) = file_tmp_rw();

  f.write(&[0x1, 0x2, 0x3, 0x4]).unwrap();
  let mut b = FileSyncedBuffer::new(f, 4, 16);
  assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), b.read(0, 4).unwrap());
  b.update(1, &[0x5, 0x6]); 
  assert_eq!(vec!(0x1, 0x5, 0x6, 0x4), b.read(0, 4).unwrap());

  rm_tmp(p);
}

#[test]
fn update_writes_to_subset_of_nth_page() {
  let (mut f, p) = file_tmp_rw();

  f.write(&[0x0, 0x0, 0x0, 0x0, 0x1, 0x2, 0x3, 0x4]).unwrap();
  let mut b = FileSyncedBuffer::new(f, 4, 16);
  assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), b.read(4, 4).unwrap());
  b.update(5, &[0x5, 0x6]); 
  assert_eq!(vec!(0x1, 0x5, 0x6, 0x4), b.read(4, 4).unwrap());

  rm_tmp(p);
}

#[test]
fn update_writes_to_whole_first_page() {
  let (mut f, p) = file_tmp_rw();

  f.write(&[0x1, 0x2, 0x3, 0x4]).unwrap();
  let mut b = FileSyncedBuffer::new(f, 4, 16);
  assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), b.read(0, 4).unwrap());
  b.update(0, &[0x5, 0x6, 0x7, 0x8]); 
  assert_eq!(vec!(0x5, 0x6, 0x7, 0x8), b.read(0, 4).unwrap());

  rm_tmp(p);
}

#[test]
fn update_writes_to_whole_nth_page() {
  let (mut f, p) = file_tmp_rw();

  f.write(&[0x0, 0x0, 0x0, 0x0, 0x1, 0x2, 0x3, 0x4]).unwrap();
  let mut b = FileSyncedBuffer::new(f, 4, 16);
  assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), b.read(4, 4).unwrap());
  b.update(4, &[0x5, 0x6, 0x7, 0x8]); 
  assert_eq!(vec!(0x5, 0x6, 0x7, 0x8), b.read(4, 4).unwrap());

  rm_tmp(p);
}

#[test]
fn update_writes_across_page_boundaries_from_first_page() {
  let (mut f, p) = file_tmp_rw();

  f.write(&[0x0, 0x0, 0x1, 0x2, 0x3, 0x4]).unwrap();
  let mut b = FileSyncedBuffer::new(f, 4, 16);
  assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), b.read(2, 4).unwrap());
  b.update(2, &[0x5, 0x6, 0x7, 0x8]); 
  assert_eq!(vec!(0x5, 0x6, 0x7, 0x8), b.read(2, 4).unwrap());

  rm_tmp(p);
}

#[test]
fn update_writes_across_page_boundaries_from_nth_page() {
  let (mut f, p) = file_tmp_rw();

  f.write(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x2, 0x3, 0x4]).unwrap();
  let mut b = FileSyncedBuffer::new(f, 4, 16);
  assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), b.read(6, 4).unwrap());
  b.update(6, &[0x5, 0x6, 0x7, 0x8]); 
  assert_eq!(vec!(0x5, 0x6, 0x7, 0x8), b.read(6, 4).unwrap());

  rm_tmp(p);
}

#[test]
fn update_writes_across_multiple_page_boundaries_from_first_page() {
  let (mut f, p) = file_tmp_rw();

  f.write(
    &[0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]
  ).unwrap();
  let mut b = FileSyncedBuffer::new(f, 4, 16);
  assert_eq!(vec!(0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8), b.read(6, 8).unwrap());
  b.update(6, &[0x8, 0x7, 0x6, 0x5, 0x4, 0x3, 0x2, 0x1]); 
  assert_eq!(vec!(0x8, 0x7, 0x6, 0x5, 0x4, 0x3, 0x2, 0x1), b.read(6, 8).unwrap());

  rm_tmp(p);
}

#[test]
fn update_writes_across_multiple_page_boundaries_from_nth_page() {
  let (mut f, p) = file_tmp_rw();

  f.write(
    &[0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]
  ).unwrap();
  let mut b = FileSyncedBuffer::new(f, 4, 16);
  assert_eq!(vec!(0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8), b.read(6, 8).unwrap());
  b.update(6, &[0x8, 0x7, 0x6, 0x5, 0x4, 0x3, 0x2, 0x1]); 
  assert_eq!(vec!(0x8, 0x7, 0x6, 0x5, 0x4, 0x3, 0x2, 0x1), b.read(6, 8).unwrap());

  rm_tmp(p);
}

#[test]
fn update_only_writes_to_cached_pages() {
  let (mut f, p) = file_tmp_rw();

  f.write(
    &[0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]
  ).unwrap();
  let mut b = FileSyncedBuffer::new(f, 4, 16);
  assert_eq!(vec!(0x0, 0x0, 0x1, 0x2), b.read(4, 4).unwrap());
  assert_eq!(vec!(0x7, 0x8), b.read(12, 4).unwrap());
  b.update(6, &[0x8, 0x7, 0x6, 0x5, 0x4, 0x3, 0x2, 0x1]); 
  assert_eq!(vec!(0x0, 0x0, 0x8, 0x7), b.read(4, 4).unwrap());
  assert_eq!(vec!(0x3, 0x4, 0x5, 0x6), b.read(8, 4).unwrap());
  assert_eq!(vec!(0x2, 0x1), b.read(12, 4).unwrap());

  rm_tmp(p);
}

// truncate() tests
#[test]
fn truncate_to_0_removes_all_pages() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16);
  assert_eq!(0, b.get_num_current_pages());
  b.read(4, 64).unwrap();
  assert_eq!(5, b.get_num_current_pages());
  assert_eq!(vec!(0, 1, 2, 3, 4), b.get_current_page_insertions());
  b.truncate(0);
  assert_eq!(0, b.get_num_current_pages());
  assert_eq!(Vec::<u64>::new(), b.get_current_page_insertions());
}

#[test]
fn truncate_removes_pages_past_len() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16);
  assert_eq!(0, b.get_num_current_pages());
  b.read(4, 64).unwrap();
  assert_eq!(5, b.get_num_current_pages());
  assert_eq!(vec!(0, 1, 2, 3, 4), b.get_current_page_insertions());
  b.truncate(45);
  assert_eq!(3, b.get_num_current_pages());
  assert_eq!(vec!(0, 1, 2), b.get_current_page_insertions());
}

#[test]
fn truncate_truncates_page_at_len() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16);
  assert_eq!(0, b.get_num_current_pages());
  let res1 = b.read(32, 16).unwrap();
  assert_eq!(16, res1.len());
  assert_eq!("ectetur adipisci", str::from_utf8(res1.as_slice()).unwrap());
  b.truncate(45);
  let res2 = b.read(32, 16).unwrap();
  assert_eq!(13, res2.len());
  assert_eq!("ectetur adipi", str::from_utf8(res2.as_slice()).unwrap());
}

// get_page_size() tests
#[test]
fn get_page_size_returns_initialized_page_size() {
  let b = FileSyncedBuffer::new(file_r("100.txt"), 32, 64);
  assert_eq!(32, b.get_page_size());
}

// get_max_pages() and set_max_pages() tests
#[test]
fn get_max_pages_returns_initialized_max_pages() {
  let b = FileSyncedBuffer::new(file_r("100.txt"), 32, 64);
  assert_eq!(64, b.get_max_pages());
}

#[test]
fn get_max_pages_returns_max_pages_after_set_max_pages() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 32, 64);
  b.set_max_pages(20);
  assert_eq!(20, b.get_max_pages());
  b.set_max_pages(0);
  assert_eq!(0, b.get_max_pages());
}

#[test]
fn set_max_pages_removes_oldest_pages_when_reduced() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 4);
  assert_eq!(0, b.get_num_current_pages());
  b.read(0, 64).unwrap();
  assert_eq!(vec!(0, 1, 2, 3), b.get_current_page_insertions());
  b.set_max_pages(2);
  assert_eq!(vec!(2, 3), b.get_current_page_insertions());
}

#[test]
fn set_max_pages_allows_more_pages_when_increased() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 2);
  assert_eq!(0, b.get_num_current_pages());
  b.read(0, 64).unwrap();
  assert_eq!(vec!(2, 3), b.get_current_page_insertions());
  b.set_max_pages(4);
  b.read(0, 64).unwrap();
  assert_eq!(vec!(2, 3, 0, 1), b.get_current_page_insertions());
}

// get_num_current_pages() tests
#[test]
fn get_num_current_pages_starts_at_0() {
  let b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16);
  assert_eq!(0, b.get_num_current_pages());
}

#[test]
fn get_num_current_pages_increases_as_pages_are_read() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16);
  b.read(0, 16).unwrap();
  assert_eq!(1, b.get_num_current_pages());
  b.read(16, 16).unwrap();
  assert_eq!(2, b.get_num_current_pages());
  b.read(32, 16).unwrap();
  assert_eq!(3, b.get_num_current_pages());
}

#[test]
fn get_num_current_pages_does_not_return_more_than_max_pages() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 4);
  b.read(0, 16).unwrap();
  assert_eq!(1, b.get_num_current_pages());
  b.read(16, 16).unwrap();
  assert_eq!(2, b.get_num_current_pages());
  b.read(32, 16).unwrap();
  assert_eq!(3, b.get_num_current_pages());
  b.read(48, 16).unwrap();
  assert_eq!(4, b.get_num_current_pages());
  b.read(64, 16).unwrap();
  assert_eq!(4, b.get_num_current_pages());
}

// get_current_page_insertions() tests
#[test]
fn get_current_page_insertions_starts_empty() {
  let b = FileSyncedBuffer::new(file_r("100.txt"), 16, 4);
  assert_eq!(0, b.get_current_page_insertions().len());
}

#[test]
fn get_current_page_insertions_adds_pages_in_order_of_insertion() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 4);
  b.read(16, 16).unwrap();
  assert_eq!(vec!(1), b.get_current_page_insertions());
  b.read(48, 16).unwrap();
  assert_eq!(vec!(1, 3), b.get_current_page_insertions());
  b.read(0, 16).unwrap();
  assert_eq!(vec!(1, 3, 0), b.get_current_page_insertions());
}

#[test]
fn get_current_page_insertions_shows_oldest_pages_removed_first() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 4);
  b.read(16, 16).unwrap();
  assert_eq!(vec!(1), b.get_current_page_insertions());
  b.read(48, 16).unwrap();
  assert_eq!(vec!(1, 3), b.get_current_page_insertions());
  b.read(0, 16).unwrap();
  assert_eq!(vec!(1, 3, 0), b.get_current_page_insertions());
  b.read(32, 16).unwrap();
  assert_eq!(vec!(1, 3, 0, 2), b.get_current_page_insertions());
  b.read(64, 16).unwrap();
  assert_eq!(vec!(3, 0, 2, 4), b.get_current_page_insertions());
  b.read(96, 16).unwrap();
  assert_eq!(vec!(0, 2, 4, 6), b.get_current_page_insertions());
}

// page caching tests
#[test]
fn reads_1_page_when_caching_0_pages() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 0);
  assert_eq!(0, b.get_num_current_pages());
  let res = b.read(4, 4).unwrap();
  assert_eq!(4, res.len());
  assert_eq!("m ip", str::from_utf8(res.as_slice()).unwrap());
  assert_eq!(0, b.get_num_current_pages());
}

#[test]
fn reads_multiple_pages_when_caching_0_pages() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 0);
  assert_eq!(0, b.get_num_current_pages());
  let res = b.read(4, 32).unwrap();
  assert_eq!(32, res.len());
  assert_eq!(
    "m ipsum dolor sit amet, consecte", 
    str::from_utf8(res.as_slice()).unwrap()
  );
  assert_eq!(0, b.get_num_current_pages());
}

#[test]
fn reads_1_page_when_caching_1_page() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 1);
  assert_eq!(0, b.get_num_current_pages());
  let res = b.read(4, 4).unwrap();
  assert_eq!(4, res.len());
  assert_eq!("m ip", str::from_utf8(res.as_slice()).unwrap());
  assert_eq!(1, b.get_num_current_pages());
}

#[test]
fn reads_multiple_pages_when_caching_1_page() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 1);
  assert_eq!(0, b.get_num_current_pages());
  let res = b.read(4, 32).unwrap();
  assert_eq!(32, res.len());
  assert_eq!(
    "m ipsum dolor sit amet, consecte", 
    str::from_utf8(res.as_slice()).unwrap()
  );
  assert_eq!(1, b.get_num_current_pages());
}

#[test]
fn reads_1_page_when_caching_multiple_pages() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16);
  assert_eq!(0, b.get_num_current_pages());
  let res = b.read(4, 4).unwrap();
  assert_eq!(4, res.len());
  assert_eq!("m ip", str::from_utf8(res.as_slice()).unwrap());
  assert_eq!(1, b.get_num_current_pages());
}

#[test]
fn reads_multiple_pages_when_caching_multiple_pages() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16);
  assert_eq!(0, b.get_num_current_pages());
  let res = b.read(4, 32).unwrap();
  assert_eq!(32, res.len());
  assert_eq!(
    "m ipsum dolor sit amet, consecte", 
    str::from_utf8(res.as_slice()).unwrap()
  );
  assert_eq!(3, b.get_num_current_pages());
}

#[test]
fn only_caches_up_to_max_pages() {
  let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 3);
  assert_eq!(0, b.get_num_current_pages());
  let res = b.read(4, 64).unwrap();
  assert_eq!(64, res.len());
  assert_eq!(
    "m ipsum dolor sit amet, consectetur adipiscing elit. Integer ut ", 
    str::from_utf8(res.as_slice()).unwrap()
  );
  assert_eq!(3, b.get_num_current_pages());
}

#[test]
fn empty_pages_are_not_cached() {
  let mut b = FileSyncedBuffer::new(file_r("10.txt"), 4, 16);
  assert_eq!(0, b.get_num_current_pages());
  b.read(0, 128).unwrap();
  assert_eq!(3, b.get_num_current_pages());
}
