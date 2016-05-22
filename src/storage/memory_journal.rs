#![feature(alloc, heap_api)]

extern crate alloc;
extern crate core;

use alloc::heap;
use std::{mem, ptr, slice};
use storage::journal::Journal;
use storage::journal_reader::JournalReader;
use storage::journal_writer::JournalWriter;


pub struct MemoryJournal {
    reader: JournalReader,
    writer: JournalWriter,
    is_open: bool
}
impl MemoryJournal {

    pub fn new(
        initial_capacity: usize,
        expand_size: usize
    ) -> MemoryJournal {
        let writer = JournalWriter::new(initial_capacity, expand_size, 1024);
        let reader = JournalReader::new(writer.storage_origin(), writer.capacity(), 1024);

        MemoryJournal {
            reader: reader,
            writer: writer,
            is_open: false
        }
    }

}
impl Journal for MemoryJournal {


    fn open(&mut self) {
        self.is_open = true;
    }

    fn close(&mut self) {
        self.is_open = false;
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn reset(&mut self) -> bool {
        if !self.is_open { return false }
        self.reader.reset();
        true
    }

    fn write(&mut self, data: &[u8]) -> bool {
        if !self.is_open { return false }
        let old_storage_origin = self.writer.storage_origin();
        let res = self.writer.write(data);
        if self.writer.storage_origin() != old_storage_origin {
            self.reader.storage_reallocated(self.writer.storage_origin(), self.writer.capacity());
        }
        res
    }

    fn commit(&mut self) -> bool {
        if !self.is_open { return false }
        self.writer.commit()
    }

    fn discard(&mut self) -> bool {
        if !self.is_open { return false }
        self.writer.discard()
    }

    fn next(&mut self) -> bool {
        if !self.is_open { return false }
        self.reader.next()
    }

    fn size(&self) -> u32 {
        if !self.is_open { return 0 }
        self.reader.size()
    }

    fn read(&self) -> Option<Vec<u8>> {
        if !self.is_open { return None }
        self.reader.read()
    }

    fn is_writing(&self) -> bool {
        if !self.is_open { return false }
        self.writer.is_writing()
    }

    fn capacity(&self) -> usize {
        if !self.is_open { return 0 }
        self.writer.capacity()
    }

}
impl Drop for MemoryJournal {

    fn drop(&mut self) {
        let storage_origin = self.writer.storage_origin();
        unsafe { 
            heap::deallocate(
                storage_origin as *mut u8, 
                self.writer.capacity(), 
                self.writer.align()
            ); 
        }
        self.reader.forget();
        self.writer.forget();
    }

}


/*
#[cfg(test)]
mod tests {

    #![feature(alloc, heap_api)]

    extern crate alloc;
    extern crate core;

    use alloc::heap;
    use std::{mem, ptr, slice};
    use storage::journal::Journal;
    use storage::memory_journal::MemoryJournal;

    #[test]
    fn commits_and_reads_one_item() {
        let mut journal = MemoryJournal::new(1024, 1024);

        let data: [u8; 3] = [1, 2, 3];
        journal.open();
        journal.write(&data);
        journal.commit();

        let r = journal.read().unwrap();

        assert_eq!(3, r.len());
        assert_eq!(1, r[0]);
        assert_eq!(2, r[1]);
        assert_eq!(3, r[2]);
    }

    #[test]
    fn returns_correct_record_size() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data: [u8; 5] = [0, 1, 2, 3, 4];
        journal.open();
        journal.write(&data);
        journal.commit();
        assert_eq!(5, journal.size());
    }

    #[test]
    fn is_complete_when_started() {
        let mut journal = MemoryJournal::new(1024, 1024);
        journal.open();
        assert!(!journal.is_writing());
    }

    #[test]
    fn is_not_complete_after_write() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data: [u8; 5] = [0, 1, 2, 3, 4];
        journal.open();
        journal.write(&data);
        assert!(journal.is_writing());
    }

    #[test]
    fn is_complete_after_commit() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data: [u8; 5] = [0, 1, 2, 3, 4];
        journal.open();
        journal.write(&data);
        journal.commit();
        assert!(!journal.is_writing());
    }

    #[test]
    fn is_complete_after_discard() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data: [u8; 5] = [0, 1, 2, 3, 4];
        journal.open();
        journal.write(&data);
        journal.discard();
        assert!(!journal.is_writing());
    }

    #[test]
    fn read_returns_none_when_reading_discarded_record() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data: [u8; 5] = [0, 1, 2, 3, 4];
        journal.open();
        journal.write(&data);
        journal.discard();
        assert_eq!(None, journal.read());
    }

    #[test]
    fn write_adds_record_in_place_of_discarded_record() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data1: [u8; 2] = [0, 1];
        journal.open();
        journal.write(&data1);
        journal.commit();

        let data2: [u8; 3] = [7, 8, 9];
        journal.write(&data2);
        journal.discard();

        let data3: [u8; 4] = [6, 5, 4, 3];
        journal.write(&data3);
        journal.commit();

        assert_eq!(2, journal.size());
        journal.next();
        assert_eq!(4, journal.size());
    }

    #[test]
    fn read_returns_none_when_reading_incomplete_record() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data: [u8; 5] = [0, 1, 2, 3, 4];
        journal.open();
        journal.write(&data);
        assert_eq!(None, journal.read());
    }

    #[test]
    fn commits_multiple_items_then_reads_from_beginning() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data1: [u8; 2] = [0, 1];
        journal.open();
        journal.write(&data1);
        journal.commit();

        let data2: [u8; 3] = [7, 8, 9];
        journal.write(&data2);
        journal.commit();

        let data3: [u8; 4] = [6, 5, 4, 3];
        journal.write(&data3);
        journal.commit();

        assert_eq!(2, journal.size());
        journal.next();
        assert_eq!(3, journal.size());
        journal.next();
        assert_eq!(4, journal.size());
    }

    #[test]
    fn read_returns_none_when_no_more_elements() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data: [u8; 5] = [0, 1, 2, 3, 4];
        journal.open();
        journal.write(&data);
        journal.commit();
        assert!(journal.read().is_some());
        journal.next();
        assert_eq!(None, journal.read());
    }

    #[test]
    fn allocates_initial_capacity_of_start_capacity() {
        let mut journal = MemoryJournal::new(1024, 2048);
        journal.open();
        assert_eq!(1024, journal.capacity());
    }

    #[test]
    fn allocates_more_capacity_by_given_size_increment() {
        let mut journal = MemoryJournal::new(4096, 2048);
        journal.open();
        assert_eq!(4096, journal.capacity());
        let data: [u8; 1500] = unsafe { mem::uninitialized() };
        journal.write(&data);
        journal.commit();
        journal.write(&data);
        journal.commit();
        journal.write(&data);
        journal.commit();
        assert_eq!(6144, journal.capacity());
        journal.write(&data);
        journal.commit();
        assert_eq!(6144, journal.capacity());
        journal.write(&data);
        journal.commit();
        assert_eq!(8192, journal.capacity());
        journal.write(&data);
        journal.commit();
        assert_eq!(10240, journal.capacity());
    }

    #[test]
    fn allocates_enough_capacity_for_record_when_record_larger_than_size_increment() {
        let mut journal = MemoryJournal::new(1024, 512);
        journal.open();
        assert_eq!(1024, journal.capacity());
        let data: [u8; 4000] = unsafe { mem::uninitialized() };
        journal.write(&data);
        journal.commit();
        assert_eq!(4096, journal.capacity());
    }

    #[test]
    fn reset_starts_reading_from_beginning() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data1: [u8; 2] = [0, 1];
        journal.open();
        journal.write(&data1);
        journal.commit();

        let data2: [u8; 3] = [7, 8, 9];
        journal.write(&data2);
        journal.commit();

        let data3: [u8; 4] = [6, 5, 4, 3];
        journal.write(&data3);
        journal.commit();

        journal.next();
        journal.next();
        journal.reset();

        assert_eq!(2, journal.size());
        journal.next();
        assert_eq!(3, journal.size());
        journal.next();
        assert_eq!(4, journal.size());
    }

    #[test]
    fn read_without_next_reads_same_record() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data1: [u8; 2] = [0, 1];
        journal.open();
        journal.write(&data1);
        journal.commit();

        let data2: [u8; 3] = [7, 8, 9];
        journal.write(&data2);
        journal.commit();

        assert_eq!(2, journal.size());
        assert_eq!(2, journal.size());
        journal.next();
        assert_eq!(3, journal.size());
    }

}
*/
