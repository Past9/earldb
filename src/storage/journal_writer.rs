#![feature(alloc, heap_api)]

extern crate alloc;
extern crate core;

use alloc::heap;
use std::{mem, ptr, slice};
use storage::journal::Journal;


pub struct JournalWriter {
    storage_origin: Option<*const u8>,
    record_offset: usize,
    write_offset: usize,
    capacity: usize,
    expand_size: usize,
    align: usize,
    is_writing: bool,
    uncommitted_size: usize
}
impl JournalWriter {

    /// Creates a new JournalWriter object
    pub fn new(
        initial_capacity: usize,
        expand_size: usize,
        align: usize,
    ) -> JournalWriter {

        let storage_origin = unsafe { heap::allocate(initial_capacity, align) as *const u8 };

        let writer = JournalWriter {
            storage_origin: Some(storage_origin),
            capacity: initial_capacity,
            expand_size: expand_size,
            align: align,
            record_offset: 0,
            write_offset: 0,
            is_writing: false,
            uncommitted_size: 0
        };

        writer.init_mem_from(0);

        writer
    }

    fn init_mem_from(&self, start: usize) {
        let ptr = (self.storage_origin() as usize + start) as *mut u8;
        let len = self.capacity - start;
        unsafe { ptr::write_bytes(ptr, 0, len) }
    }

    fn record_ptr(&self) -> *mut u8 {
        return (
            self.storage_origin() as usize + 
            self.record_offset 
        ) as *mut u8;
    }

    fn write_ptr(&self) -> *mut u8 {
        return (
            self.storage_origin() as usize + 
            self.record_offset + 
            self.write_offset
        ) as *mut u8;
    }

    fn expand_if_needed(&mut self, data_size: usize) -> bool {
        // The size of the full record is:
        // STX byte (1 byte) +
        // u32 data length (4 bytes) +
        // Record data (data_size bytes) +
        // ETX byte(1 byte)
        let record_size = 1 + 4 + data_size + 1;

        // Determine the minimum size that the journal needs to be in
        // order to hold the new record
        let needed_capacity = self.record_offset + record_size;

        // Return if there is already enough room 
        if self.capacity >= needed_capacity { return true }

        // Determine the new size of the journal in multiples of expand_size
        let new_capacity = 
            (needed_capacity as f32 / self.expand_size as f32).ceil() as usize 
            * self.expand_size;

        // Allocate and record the new capacity
        let ptr = unsafe {
            heap::reallocate(
                self.storage_origin() as *mut u8, 
                self.capacity, 
                new_capacity, 
                self.align
            )
        };

        // Return false if not enough storage could be allocated
        if ptr.is_null() {
            return false;
        } else {
            // Set the new capacity and pointer, remembering the old capacity
            let old_capacity = self.capacity;
            self.storage_origin = Some(ptr as *const u8);
            self.capacity = new_capacity;
            // Initialize the new storage (set all bytes to 0x00)
            self.init_mem_from(old_capacity);
            // Return true to indicate that allocation was successful
            return true;
        }

    }

    pub fn as_slice(&self) -> &[u8] {
        match self.storage_origin {
            Some(x) => unsafe { slice::from_raw_parts(x, self.capacity) },
            None => &[]
        }
    }

    pub fn forget(&mut self) {
        self.storage_origin = None;
        self.capacity = 0;
    }

    pub fn expand_size(&self) -> usize {
        self.expand_size
    }

    pub fn align(&self) -> usize {
        self.align
    }

    pub fn storage_origin(&self) -> *const u8 {
        self.storage_origin.unwrap()
    }

    /// Returns the current size of the journal in bytes. This is the current capacity,
    /// not the size of the actual records, which is likely smaller.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns whether an uncommitted record has been written to the journal
    pub fn is_writing(&self) -> bool {
        self.is_writing
    }

    /// Appends a new record to the journal but does not mark it as committed.
    /// Returns true if the write was performed.
    /// Returns false and does nothing if there is currently another uncommitted record.
    /// Returns false if enough memory could not be allocated for the new record.
    pub fn write(&mut self, data: &[u8]) -> bool {
        if self.is_writing { return false }

        // Lock the writer so that no other writes can take place until a commit
        // or discard
        self.is_writing = true;

        // Allocate more memory to make room for the new record if necessary
        let enough_capacity = self.expand_if_needed(data.len());

        // If enough capacity could not be allocated for the new record,
        // cancel the write and return false
        if !enough_capacity {
            self.discard();
            return false;
        }

        // Write the STX (start of text) marker and advance the write offset, 
        // then record how much data is uncommitted
        unsafe { ptr::write(self.write_ptr(), 0x02) }
        self.write_offset = 1;
        self.uncommitted_size = 1;


        // Write the size of the data as a u32 and advance the write offset,
        // then record how much data is uncommitted
        let len = data.len() as u32;  
        unsafe {
            let len_ptr: *const u32 = mem::transmute(&len);
            ptr::copy(len_ptr, self.write_ptr() as *mut u32, 1);
        }
        self.write_offset = 5;
        self.uncommitted_size = 5;

        // Write the data bytes and advance the write offset, then
        // record how much data is uncommitted
        let dest_slice = unsafe { slice::from_raw_parts_mut(self.write_ptr(), data.len() as usize) };
        dest_slice.clone_from_slice(data);
        self.write_offset += data.len();
        self.uncommitted_size += data.len();

        // Return that the write was successful (though still uncommitted)
        true

    }

    /// Marks a previously written and uncommitted record as committed.
    /// Does nothing if there is not currently an uncommitted record.
    pub fn commit(&mut self) -> bool {
        if !self.is_writing { return false }

        // Write the ETX (end of text) marker
        unsafe { ptr::write(self.write_ptr(), 0x03) }

        // Move the record offset to the next record
        self.record_offset += self.uncommitted_size + 1;

        // Reset state
        self.write_offset = 0;
        self.uncommitted_size = 0;
        self.is_writing = false;

        true
    }

    /// Discards a previously written but uncommitted record.
    /// Does nothing if there is not currently an uncommitted record.
    pub fn discard(&mut self) -> bool {
        if !self.is_writing { return false }

        // Reinitialize all the uncommitted bytes to zero
        unsafe { ptr::write_bytes(self.record_ptr(), 0, self.uncommitted_size) }

        //Reset state
        self.write_offset = 0;
        self.uncommitted_size = 0;
        self.is_writing = false;

        true
    }

}
impl Drop for JournalWriter {

    fn drop(&mut self) {
        match self.storage_origin {
            Some(s) => unsafe {
                heap::deallocate(
                    s as *mut u8,
                    self.capacity(),
                    self.align()
                );
            },
            None => ()
        }
    }

}


#[cfg(test)]
mod tests {

    #![feature(alloc, heap_api)]

    extern crate alloc;
    extern crate core;

    use alloc::heap;
    use std::{mem, ptr, slice};
    use storage::journal_writer::JournalWriter;

    #[test]
    fn new_sets_properties() {
        let writer = JournalWriter::new(256, 512, 1024);
        assert_eq!(256, writer.capacity());
        assert_eq!(512, writer.expand_size());
        assert_eq!(1024, writer.align());
    }

    #[test]
    fn as_slice_returns_slice_with_capacity_length() {
        let writer = JournalWriter::new(256, 512, 1024);
        let slice = writer.as_slice();
        assert_eq!(slice.len(), writer.capacity());
    }

    #[test]
    fn new_inits_memory_to_zeroes() {
        let writer = JournalWriter::new(256, 512, 1024);
        let slice = writer.as_slice();
        for i in slice {
            assert_eq!(0x00, *i);
        }
    }

    #[test]
    fn as_slice_returns_empty_slice_after_forget() {
        let mut writer = JournalWriter::new(256, 512, 1024);
        writer.forget();
        let slice = writer.as_slice();
        assert_eq!(0, slice.len());
    }

    #[test]
    fn capacity_is_zero_after_forget() {
        let mut writer = JournalWriter::new(256, 512, 1024);
        writer.forget();
        assert_eq!(0, writer.capacity());
    }

    #[test]
    fn storage_origin_returns_non_null_pointer() {
        let writer = JournalWriter::new(256, 512, 1024);
        assert!(!writer.storage_origin().is_null());
    }

    #[test]
    #[should_panic]
    fn storage_origin_panics_after_forget() {
        let mut writer = JournalWriter::new(256, 512, 1024);
        writer.forget();
        writer.storage_origin();
    }

    #[test]
    fn is_not_writing_when_new() {
        let writer = JournalWriter::new(256, 512, 1024);
        assert!(!writer.is_writing());
    }

    #[test]
    fn commit_returns_false_when_not_writing() {
        let mut writer = JournalWriter::new(256, 512, 1024);
        assert!(!writer.commit());
    }

    #[test]
    fn commit_does_not_alter_contents_or_capacity_when_not_writing() {
        let mut writer = JournalWriter::new(256, 512, 1024);
        writer.commit();
        assert_eq!(256, writer.capacity());
        let slice = writer.as_slice();
        for i in slice {
            assert_eq!(0x00, *i);
        }
    }

    #[test]
    fn write_sets_is_writing_to_true() {
        let mut writer = JournalWriter::new(256, 512, 1024);
        writer.write(&[1, 2, 3]);
        assert!(writer.is_writing());
    }

    #[test]
    fn write_returns_true_when_not_already_writing() {
        let mut writer = JournalWriter::new(256, 512, 1024);
        assert!(writer.write(&[1, 2, 3]));
    }

    #[test]
    fn write_returns_false_when_already_writing() {
        let mut writer = JournalWriter::new(256, 512, 1024);
        writer.write(&[1, 2, 3]);
        assert!(!writer.write(&[4, 5, 6]));
    }

    #[test]
    fn write_sets_data_except_end_byte() {
        let mut writer = JournalWriter::new(256, 512, 1024);
        writer.write(&[1, 2, 3]);
        assert_eq!(
            [0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x00],
            writer.as_slice()[0..9]
        );
    }

    #[test]
    fn discard_returns_false_when_not_writing() {
        let mut writer = JournalWriter::new(256, 512, 1024);
        assert!(!writer.discard());
    }

    #[test]
    fn discard_returns_true_when_writing() {
        let mut writer = JournalWriter::new(256, 512, 1024);
        writer.write(&[1, 2, 3]);
        assert!(writer.discard());
    }

    #[test]
    fn discard_zeroes_bytes() {
        let mut writer = JournalWriter::new(256, 512, 1024);
        writer.write(&[1, 2, 3]);
        writer.discard();
        assert_eq!(
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            writer.as_slice()[0..9]
        );
    }

    #[test]
    fn commit_sets_end_byte() {
        let mut writer = JournalWriter::new(256, 512, 1024);
        writer.write(&[1, 2, 3]);
        writer.commit();
        assert_eq!(
            [0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03],
            writer.as_slice()[0..9]
        );
    }

    #[test]
    fn discard_zeroes_only_uncommitted_bytes() {
        let mut writer = JournalWriter::new(256, 512, 1024);
        writer.write(&[1, 2, 3]);
        writer.commit();
        writer.write(&[4, 5, 6]);
        assert_eq!(
            [
                0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03, 
                0x02, 0x03, 0x00, 0x00, 0x00, 0x04, 0x05, 0x06, 0x00
            ],
            writer.as_slice()[0..18]
        );
        writer.discard();
        assert_eq!(
            [
                0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03, 
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
            ],
            writer.as_slice()[0..18]
        );
    }

    #[test]
    fn write_sets_contents_in_place_of_discarded_record() {
        let mut writer = JournalWriter::new(256, 512, 1024);
        writer.write(&[1, 2, 3]);
        writer.commit();
        writer.write(&[4, 5, 6]);
        writer.discard();
        writer.write(&[7, 8, 9]);
        assert_eq!(
            [
                0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03, 
                0x02, 0x03, 0x00, 0x00, 0x00, 0x07, 0x08, 0x09, 0x00
            ],
            writer.as_slice()[0..18]
        );
    }

    #[test]
    fn commit_sets_end_bytes_on_multiple_records() {
        let mut writer = JournalWriter::new(256, 512, 1024);
        writer.write(&[1, 2, 3]);
        writer.commit();
        writer.write(&[4, 5, 6]);
        writer.commit();
        assert_eq!(
            [
                0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03, 
                0x02, 0x03, 0x00, 0x00, 0x00, 0x04, 0x05, 0x06, 0x03
            ],
            writer.as_slice()[0..18]
        );
    }


    

}
