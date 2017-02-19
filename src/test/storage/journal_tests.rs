// TODO: Test for invalid checksums, especially their effects on verify() behavior

use std::error::Error;
use storage::journal;
use storage::journal::Journal;
use storage::binary_storage;
use storage::binary_storage::BinaryStorage;
use storage::transactional_storage;
use storage::transactional_storage::TransactionalStorage;
use storage::memory_binary_storage::MemoryBinaryStorage;

fn new_storage(
  initial_capacity: usize, 
  expand_size: usize
) -> TransactionalStorage<MemoryBinaryStorage> {
  TransactionalStorage::new(
    MemoryBinaryStorage::new(initial_capacity, expand_size).unwrap()
  )
}

// new() tests
#[test]
pub fn mem_new_reads_and_writes_from_0_when_empty_storage() {
  let j = Journal::new(new_storage(256, 256));
  assert_eq!(0, j.read_offset());
  assert_eq!(0, j.write_offset());
}


// open(), close(), verify(), and is_open() tests
#[test]
pub fn is_closed_by_default() {
  let j = Journal::new(new_storage(256, 256));
  assert!(!j.is_open());
}

#[test]
pub fn close_returns_err_when_already_closed() {
  let mut j = Journal::new(new_storage(256, 256));
  assert_eq!(
    binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
    j.close().unwrap_err().description()
  );
}

#[test]
pub fn open_returns_ok_when_previously_closed() {
  let mut j = Journal::new(new_storage(256, 256));
  assert!(j.open().is_ok());
}

#[test]
pub fn open_returns_err_when_previously_open() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  assert_eq!(
    binary_storage::ERR_OPERATION_INVALID_WHEN_OPEN,
    j.open().unwrap_err().description()
  );
}

#[test]
pub fn open_and_verify_counts_existing_records() {
  let mut s = new_storage(256, 256);
  s.open().unwrap();
  s.w_bytes(
    0, 
    &[0x2, 0x2, 0x3, 0x0, 0x0, 0x0, 0x1, 0x2, 0x3, 0x0, 0x3, 0x3]
  ).unwrap();
  s.close().unwrap();
  let mut j = Journal::new(s);
  j.open().unwrap();
  assert_eq!(1, j.record_count());
}

#[test]
pub fn open_and_verify_does_not_count_uncommitted_records() {
  let mut s = new_storage(256, 256);
  s.open().unwrap();
  s.w_bytes(
    0, 
    &[0x2, 0x2, 0x3, 0x0, 0x0, 0x0, 0x1, 0x2, 0x3, 0x0, 0x3, 0x3]
  ).unwrap();
  s.w_bytes(12, &[0x2, 0x2, 0x3, 0x0, 0x0, 0x0]).unwrap();
  s.close().unwrap();
  let mut j = Journal::new(s);
  j.open().unwrap();
  assert_eq!(1, j.record_count());
}

#[test]
pub fn open_and_verify_returns_err_on_bad_checksum() {
  let mut s = new_storage(256, 256);
  s.open().unwrap();
  s.w_bytes(
    0, 
    &[0x2, 0x2, 0x3, 0x0, 0x0, 0x0, 0x1, 0x2, 0x3, 0x1, 0x3, 0x3]
  ).unwrap();
  s.close().unwrap();
  let mut j = Journal::new(s);
  assert_eq!(
    journal::ERR_CHECKSUM_MISMATCH,
    j.open().unwrap_err().description()
  );
}

#[test]
pub fn open_and_verify_recognizes_all_committed_records() {
  let mut s = new_storage(256, 256);
  s.open().unwrap();
  s.w_bytes(
    0, 
    &[0x2, 0x2, 0x3, 0x0, 0x0, 0x0, 0x1, 0x2, 0x3, 0x0, 0x3, 0x3]
  ).unwrap();
  s.close().unwrap();
  let mut j = Journal::new(s);
  j.open().unwrap();
  assert!(!j.is_writing());
}

#[test]
pub fn open_and_verify_recognizes_uncommitted_record() {
  let mut s = new_storage(256, 256);
  s.open().unwrap();
  s.w_bytes(
    0, 
    &[0x2, 0x2, 0x3, 0x0, 0x0, 0x0, 0x1, 0x2, 0x3, 0x0, 0x3, 0x3]
  ).unwrap();
  s.close().unwrap();
  let mut j = Journal::new(s);
  j.open().unwrap();
  assert!(!j.is_writing());
}

#[test]
pub fn open_and_verify_allows_record_commit() {
  let mut s = new_storage(256, 256);
  s.open().unwrap();
  s.w_bytes(
    0, 
    &[0x2, 0x2, 0x3, 0x0, 0x0, 0x0, 0x1, 0x2, 0x3, 0x0, 0x3, 0x3]
  ).unwrap();
  s.w_bytes(12, &[0x2, 0x2, 0x3, 0x0, 0x0, 0x0]).unwrap();
  s.close().unwrap();
  let mut j = Journal::new(s);
  j.open().unwrap();
  assert_eq!(1, j.record_count());
  assert!(j.is_writing());
  assert!(j.commit().is_ok());
  assert_eq!(2, j.record_count());
  assert!(!j.is_writing());
}

#[test]
pub fn close_returns_ok_when_previously_open() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  assert!(j.close().is_ok());
}

#[test]
pub fn is_open_after_open() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  assert!(j.is_open());
}

#[test]
pub fn is_closed_after_open_and_close() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.close().unwrap();
  assert!(!j.is_open());
}

// write(), commit(), and discard() tests
#[test]
pub fn write_returns_err_when_closed() {
  let mut j = Journal::new(new_storage(256, 256));
  assert_eq!(
      binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
      j.write(&[0x0, 0x1, 0x2]).unwrap_err().description()
  );
}

#[test]
pub fn write_returns_ok_when_open() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  assert!(j.write(&[0x0, 0x1, 0x2]).is_ok());
}

#[test]
pub fn write_returns_err_when_uncommitted_data() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  assert_eq!(
    journal::ERR_WRITE_IN_PROGRESS,
    j.write(&[0x0, 0x1, 0x2]).unwrap_err().description()
  );
}

#[test]
pub fn write_returns_ok_after_commit() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert!(j.write(&[0x0, 0x1, 0x2]).is_ok());
}

#[test]
pub fn commit_returns_err_when_closed() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.close().unwrap();
  assert_eq!(
    journal::ERR_WRITE_NOT_IN_PROGRESS,
    j.commit().unwrap_err().description()
  );
}

#[test]
pub fn commit_returns_err_when_no_data() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  assert_eq!(
    journal::ERR_WRITE_NOT_IN_PROGRESS,
    j.commit().unwrap_err().description()
  );
}

#[test]
pub fn commit_returns_ok_when_uncommitted_data() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  assert!(j.commit().is_ok());
}

#[test]
pub fn discard_returns_err_when_closed() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.close().unwrap();
  assert_eq!(
    journal::ERR_WRITE_NOT_IN_PROGRESS,
    j.commit().unwrap_err().description()
  );
}

#[test]
pub fn discard_returns_ok_when_uncommitted_data() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  assert!(j.discard().is_ok());
}

#[test]
pub fn discard_returns_err_when_no_data() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  assert_eq!(
    journal::ERR_WRITE_NOT_IN_PROGRESS,
    j.discard().unwrap_err().description()
  );
}

#[test]
pub fn write_returns_ok_after_discard() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.discard().unwrap();
  assert!(j.write(&[0x0, 0x1, 0x2]).is_ok());
}


// is_writing() tests
#[test]
pub fn is_not_writing_when_new() {
  let j = Journal::new(new_storage(256, 256));
  assert!(!j.is_writing());
}

#[test]
pub fn is_not_writing_when_newly_opened() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  assert!(!j.is_writing());
}

#[test]
pub fn is_writing_after_write() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  assert!(j.is_writing());
}

#[test]
pub fn is_not_writing_when_closed() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.close().unwrap();
  assert!(!j.is_writing());
}

#[test]
pub fn is_writing_when_reopened_before_commit() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.close().unwrap();
  j.open().unwrap();
  assert!(j.is_writing());
}

#[test]
pub fn is_not_writing_after_commit() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert!(!j.is_writing());
}

#[test]
pub fn is_not_writing_after_discard() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.discard().unwrap();
  assert!(!j.is_writing());
}

// has_start() tests
#[test]
pub fn has_start_returns_err_when_closed() {
  let mut j = Journal::new(new_storage(256, 256));
  assert_eq!(
    binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
    j.has_start().unwrap_err().description()
  );
}

#[test]
pub fn has_start_returns_err_when_past_txn_boundary() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  assert_eq!(
    transactional_storage::ERR_READ_AFTER_TXN_BOUNDARY,
    j.has_start().unwrap_err().description()
  );
}

#[test]
pub fn has_start_returns_ok_when_open() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert!(j.has_start().is_ok());
}

#[test]
pub fn has_start_returns_true_when_record_exists() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert!(j.has_start().unwrap());
}

// has_end() tests
#[test]
pub fn has_end_returns_err_when_closed() {
  let mut j = Journal::new(new_storage(256, 256));
  assert_eq!(
    binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
    j.has_end().unwrap_err().description()
  );
}

#[test]
pub fn has_end_returns_err_when_past_txn_boundary() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  assert_eq!(
    transactional_storage::ERR_READ_AFTER_TXN_BOUNDARY,
    j.has_end().unwrap_err().description()
  );
}

#[test]
pub fn has_end_returns_true_when_record_is_committed() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert!(j.has_end().is_ok());
}

// read() tests
#[test]
pub fn read_returns_err_when_closed() {
  let mut j = Journal::new(new_storage(256, 256));
  assert_eq!(
    binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
    j.read().unwrap_err().description()
  );
}

#[test]
pub fn read_returns_err_when_no_data() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  assert_eq!(
    transactional_storage::ERR_READ_AFTER_TXN_BOUNDARY,
    j.read().unwrap_err().description()
  );
}

#[test]
pub fn read_returns_err_when_uncommitted_data() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  assert_eq!(
    transactional_storage::ERR_READ_AFTER_TXN_BOUNDARY,
    j.read().unwrap_err().description()
  );
}

#[test]
pub fn read_returns_ok_when_committed_data() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert!(j.read().is_ok());
}

#[test]
pub fn read_returns_first_record() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert_eq!(vec!(0x0, 0x1, 0x2), j.read().unwrap());
}

#[test]
pub fn read_returns_record_multiple_times() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert_eq!(vec!(0x0, 0x1, 0x2), j.read().unwrap());
  assert_eq!(vec!(0x0, 0x1, 0x2), j.read().unwrap());
}

// jump_to() tests
#[test]
pub fn jump_to_returns_err_when_closed() {
  let mut j = Journal::new(new_storage(256, 256));
  assert_eq!(
    binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
    j.jump_to(6).unwrap_err().description()
  );
}

#[test]
pub fn jump_to_returns_err_when_past_txn_boundary() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  assert_eq!(
    transactional_storage::ERR_READ_AFTER_TXN_BOUNDARY,
    j.jump_to(6).unwrap_err().description()
  );
}

#[test]
pub fn jump_to_returns_ok_when_at_record_start() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  j.write(&[0x3, 0x4, 0x5]).unwrap();
  j.commit().unwrap();
  assert!(j.jump_to(12).is_ok());
}

#[test]
pub fn jump_to_returns_err_when_not_at_record_start() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  j.write(&[0x3, 0x4, 0x5]).unwrap();
  j.commit().unwrap();
  assert_eq!(
    journal::ERR_NO_COMMITTED_RECORD,
    j.jump_to(13).unwrap_err().description()
  );
}

#[test]
pub fn jump_to_returns_err_when_at_uncommitted_record_start() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  j.write(&[0x3, 0x4, 0x5]).unwrap();
  assert_eq!(
    transactional_storage::ERR_READ_AFTER_TXN_BOUNDARY,
    j.jump_to(12).unwrap_err().description()
  );
}

#[test]
pub fn jump_to_still_jumps_when_err() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  j.write(&[0x3, 0x4, 0x5]).unwrap();
  j.jump_to(11).unwrap_err();
  assert_eq!(11, j.read_offset());
}

#[test]
pub fn jump_to_jumps_when_complete_record() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  j.write(&[0x3, 0x4, 0x5]).unwrap();
  j.commit().unwrap();
  j.jump_to(12).unwrap();
  assert_eq!(12, j.read_offset());
}

#[test]
pub fn jump_to_allows_record_read_at_jump_location() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert_eq!(vec!(0x0, 0x1, 0x2), j.read().unwrap());
  j.write(&[0x3, 0x4, 0x5]).unwrap();
  j.commit().unwrap();
  j.jump_to(12).unwrap();
  assert_eq!(vec!(0x3, 0x4, 0x5), j.read().unwrap());
}

// reset() tests
#[test]
pub fn reset_does_not_change_read_offset_when_already_0() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  assert_eq!(0, j.read_offset());
  j.reset();
  assert_eq!(0, j.read_offset());
}

#[test]
pub fn reset_changes_read_offset_to_0() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert_eq!(0, j.read_offset());
  j.write(&[0x3, 0x4, 0x5]).unwrap();
  j.commit().unwrap();
  j.jump_to(12).unwrap();
  assert_eq!(12, j.read_offset());
  j.reset();
  assert_eq!(0, j.read_offset());
}

#[test]
pub fn reset_allows_reading_from_first_record() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert_eq!(vec!(0x0, 0x1, 0x2), j.read().unwrap());
  j.write(&[0x3, 0x4, 0x5]).unwrap();
  j.commit().unwrap();
  j.jump_to(12).unwrap();
  assert_eq!(vec!(0x3, 0x4, 0x5), j.read().unwrap());
  j.reset();
  assert_eq!(vec!(0x0, 0x1, 0x2), j.read().unwrap());
}

// next() tests
#[test]
pub fn next_returns_none_when_closed() {
  let mut j = Journal::new(new_storage(256, 256));
  assert!(j.next().is_none());
}

#[test]
pub fn next_returns_none_when_no_records() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  assert!(j.next().is_none());
}

#[test]
pub fn next_returns_records_in_order() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert_eq!(vec!(0x0, 0x1, 0x2), j.next().unwrap());
  j.write(&[0x3, 0x4, 0x5]).unwrap();
  j.commit().unwrap();
  assert_eq!(vec!(0x3, 0x4, 0x5), j.next().unwrap());
}

#[test]
pub fn next_returns_none_when_no_more_records_available() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert_eq!(vec!(0x0, 0x1, 0x2), j.next().unwrap());
  assert!(j.next().is_none());
}

#[test]
pub fn next_returns_records_as_they_become_available() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert_eq!(vec!(0x0, 0x1, 0x2), j.next().unwrap());
  assert!(j.next().is_none());
  j.write(&[0x4, 0x5, 0x6]).unwrap();
  j.commit().unwrap();
  assert_eq!(vec!(0x4, 0x5, 0x6), j.next().unwrap());
  assert!(j.next().is_none());
}

// read_offset() tests
#[test]
pub fn read_offset_starts_at_0() {
  let j = Journal::new(new_storage(256, 256));
  assert_eq!(0, j.read_offset());
}

#[test]
pub fn read_offset_moves_on_next() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert_eq!(0, j.read_offset());
  assert_eq!(vec!(0x0, 0x1, 0x2), j.next().unwrap());
  assert_eq!(12, j.read_offset());
}

#[test]
pub fn read_offset_resets_after_reopening() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert_eq!(vec!(0x0, 0x1, 0x2), j.next().unwrap());
  assert_eq!(12, j.read_offset());
  j.close().unwrap();
  j.open().unwrap();
  assert_eq!(0, j.read_offset());
}

// write_offset() tests
#[test]
pub fn write_offset_starts_at_0() {
  let j = Journal::new(new_storage(256, 256));
  assert_eq!(0, j.read_offset());
}

#[test]
pub fn write_offset_increases_on_write() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  assert_eq!(10, j.write_offset());
}

#[test]
pub fn write_offset_increases_on_commit() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  assert_eq!(10, j.write_offset());
  j.commit().unwrap();
  assert_eq!(12, j.write_offset());
}

#[test]
pub fn write_offset_resets_on_discard() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  assert_eq!(10, j.write_offset());
  j.discard().unwrap();
  assert_eq!(0, j.write_offset());
}

#[test]
pub fn write_offset_goes_to_uncommitted_record_end_when_reopened_before_commit() {
  let mut j = Journal::new(new_storage(256, 256));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  assert_eq!(10, j.write_offset());
  j.close().unwrap();
  j.open().unwrap();
  assert_eq!(10, j.write_offset());
}

// capacity() tests
#[test]
pub fn capacity_returns_err_when_closed() {
  let j = Journal::new(new_storage(16, 16));
  assert_eq!(
    binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
    j.capacity().unwrap_err().description()
  );
}

#[test]
pub fn capacity_returns_ok_when_open() {
  let mut j = Journal::new(new_storage(16, 16));
  j.open().unwrap();
  assert!(j.capacity().is_ok());
}

#[test]
pub fn capacity_returns_initial_capacity() {
  let mut j = Journal::new(new_storage(16, 16));
  j.open().unwrap();
  assert_eq!(16, j.capacity().unwrap());
}

#[test]
pub fn capacity_returns_expanded_capacity() {
  let mut j = Journal::new(new_storage(16, 16));
  j.open().unwrap();
  assert_eq!(16, j.capacity().unwrap());
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert_eq!(16, j.capacity().unwrap());
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert_eq!(32, j.capacity().unwrap());
}

// txn_boundary() tests
#[test]
pub fn txn_boundary_returns_err_when_closed() {
  let j = Journal::new(new_storage(16, 16));
  assert_eq!(
    binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
    j.txn_boundary().unwrap_err().description()
  );
}

#[test]
pub fn txn_boundary_starts_at_0() {
  let mut j = Journal::new(new_storage(16, 16));
  j.open().unwrap();
  assert_eq!(0, j.txn_boundary().unwrap());
}

#[test]
pub fn txn_boundary_does_not_advance_on_write() {
  let mut j = Journal::new(new_storage(16, 16));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  assert_eq!(0, j.txn_boundary().unwrap());
}

#[test]
pub fn txn_boundary_advances_on_commit() {
  let mut j = Journal::new(new_storage(16, 16));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  assert_eq!(0, j.txn_boundary().unwrap());
  j.commit().unwrap();
  assert_eq!(12, j.txn_boundary().unwrap());
}

#[test]
pub fn txn_boundary_does_not_advance_on_discard() {
  let mut j = Journal::new(new_storage(16, 16));
  j.open().unwrap();
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  assert_eq!(0, j.txn_boundary().unwrap());
  j.discard().unwrap();
  assert_eq!(0, j.txn_boundary().unwrap());
  j.write(&[0x0, 0x1, 0x2]).unwrap();
  j.commit().unwrap();
  assert_eq!(12, j.txn_boundary().unwrap());
}
