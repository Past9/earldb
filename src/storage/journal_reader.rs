#![feature(alloc, heap_api)]

extern crate alloc;
extern crate core;

use alloc::heap;
use std::{mem, ptr, slice};
use storage::journal::Journal;


pub struct JournalReader {
    storage_origin: Option<*const u8>,
    record_offset: usize,
    capacity: usize,
    align: usize
}
impl JournalReader {
    
    pub fn new(
        storage_origin: *const u8,
        initial_capacity: usize,
        align: usize
    ) -> JournalReader {
        JournalReader {
            storage_origin: Some(storage_origin),
            record_offset: 0,
            capacity: initial_capacity,
            align: align
        }
    }

    pub fn forget(&mut self) {
        self.storage_origin = None;
    }

    fn storage_origin(&self) -> *const u8 {
        self.storage_origin.unwrap()
    }

    fn ptr_at_offset(&self, offset: usize) -> *const u8 {
        return (
            self.storage_origin() as usize + 
            self.record_offset +
            offset
        ) as *const u8;
    }

    fn start_byte_ref(&self) -> &u8 {
        unsafe { &*self.ptr_at_offset(0) } 
    }

    fn size_ref(&self) -> &u32 {
        unsafe { &*(self.ptr_at_offset(1) as *const u32) }
    }

    fn data_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr_at_offset(5), self.size() as usize) }
    }

    fn end_byte_ref(&self) -> &u8 {
        unsafe { &*(self.ptr_at_offset(1 + 4 + self.size() as usize) as *const u8) }
    }

    fn has_start(&self) -> bool {
        *self.start_byte_ref() == 0x02
    }

    fn has_end(&self) -> bool {
        *self.end_byte_ref() == 0x03
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn align(&self) -> usize {
        self.align
    }

    pub fn storage_reallocated(&mut self, new_storage_origin: *const u8, new_capacity: usize) {
        self.storage_origin = Some(new_storage_origin);
        self.capacity = new_capacity;
    }

    pub fn reset(&mut self) {
        self.record_offset = 0;
    }

    pub fn jump_to(&mut self, offset: usize, back_on_fail: bool) -> bool {
        // Move to the requested offset, but remember the old one
        // so we can fall back to it if the record is incomplete
        let old_offset = self.record_offset; 
        self.record_offset = offset;

        // Check to make sure the start and end bytes are present
        // (this means the record is complete and committed). If
        // so, return true to signify that the record can be read.
        if self.has_start() && self.has_end() {
            true
        } else {
            // Otherwise return false to indicate that the data at the
            // current position is not a valid and complete record.
            // Fall back to the original offset if requested.
            if back_on_fail {
                self.record_offset = old_offset;
            }
            false
        }
    }

    pub fn size(&self) -> u32 {
        if !self.has_start() { return 0 }
        *self.size_ref()
    }


}
impl Iterator for JournalReader {

    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Vec<u8>> {
        if !self.has_start() || !self.has_end() { return None };

        let res = Some(self.data_slice().to_vec());

        let new_offset = self.record_offset + 1 + 4 + self.size() as usize + 1;
        self.jump_to(new_offset, false);

        res
    }

}
impl Drop for JournalReader {

    fn drop(&mut self) {
        match self.storage_origin {
            Some(s) => unsafe {
                heap::deallocate(
                    s as *mut u8,
                    self.capacity,
                    self.align
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
    use storage::journal_reader::JournalReader;

    fn get_mem(size: usize, align: usize, data: &[u8]) -> *const u8 {
        let ptr = unsafe { heap::allocate(size, align) as *mut u8 };
        unsafe {
            ptr::write_bytes(ptr, 0, size);
            ptr::copy(data.as_ptr(), ptr, data.len());
        }
        ptr as *const u8
    }

    #[test]
    fn new_sets_properties() {
        let reader = JournalReader::new(get_mem(256, 1024, &[]), 256, 1024);
        assert_eq!(256, reader.capacity());
        assert_eq!(1024, reader.align());
    }

    #[test]
    fn storage_reallocated_changes_capacity() {
        let mut reader = JournalReader::new(get_mem(256, 1024, &[]), 256, 1024);
        let new_mem = get_mem(4096, 1024, &[]);
        reader.storage_reallocated(new_mem, 4096);
        assert_eq!(4096, reader.capacity());
    }

    #[test]
    fn next_returns_none_when_first_record_is_empty() {
        let mut reader = JournalReader::new(get_mem(256, 1024, &[]), 256, 1024);
        assert_eq!(None, reader.next());
    }

    #[test]
    fn next_returns_none_when_only_start_marker_is_present_on_first_record() {
        let mut reader = JournalReader::new(get_mem(256, 1024, &[0x02]), 256, 1024);
        assert_eq!(None, reader.next());
    }

    #[test]
    fn next_returns_none_when_only_start_marker_and_size_are_present_on_first_record() {
        let mut reader = JournalReader::new(get_mem(256, 1024, &[0x02, 0x03, 0x00, 0x00, 0x00]), 256, 1024);
        assert_eq!(None, reader.next());
    }

    #[test]
    fn next_returns_none_when_no_end_marker_is_present_on_first_record() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x00]), 
            256, 
            1024
        );
        assert_eq!(None, reader.next());
    }

    #[test]
    fn next_returns_data_when_end_marker_is_present_on_first_record() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03]), 
            256, 
            1024
        );
        assert_eq!(Some(vec!(0x01, 0x02, 0x03)), reader.next());
    }

    #[test]
    fn next_returns_none_when_nth_record_is_empty() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[
                0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
            ]), 
            256, 
            1024
        );
        assert_eq!(Some(vec!(0x01, 0x02, 0x03)), reader.next());
        assert_eq!(None, reader.next());
    }

    #[test]
    fn next_returns_none_when_only_start_marker_is_present_on_nth_record() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[
                0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03,
                0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
            ]), 
            256, 
            1024
        );
        assert_eq!(Some(vec!(0x01, 0x02, 0x03)), reader.next());
        assert_eq!(None, reader.next());
    }

    #[test]
    fn next_returns_none_when_only_start_marker_and_size_are_present_on_nth_record() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[
                0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03,
                0x02, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
            ]), 
            256, 
            1024
        );
        assert_eq!(Some(vec!(0x01, 0x02, 0x03)), reader.next());
        assert_eq!(None, reader.next());
    }

    #[test]
    fn next_returns_none_when_no_end_marker_is_present_on_nth_record() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[
                0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03,
                0x02, 0x03, 0x00, 0x00, 0x00, 0x04, 0x05, 0x06, 0x00
            ]), 
            256, 
            1024
        );
        assert_eq!(Some(vec!(0x01, 0x02, 0x03)), reader.next());
        assert_eq!(None, reader.next());
    }

    #[test]
    fn next_returns_data_when_end_marker_is_present_on_nth_record() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[
                0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03,
                0x02, 0x03, 0x00, 0x00, 0x00, 0x04, 0x05, 0x06, 0x03
            ]), 
            256, 
            1024
        );
        assert_eq!(Some(vec!(0x01, 0x02, 0x03)), reader.next());
        assert_eq!(Some(vec!(0x04, 0x05, 0x06)), reader.next());
        assert_eq!(None, reader.next());
    }

    // Iterator trait ssanity check
    #[test]
    fn iterator_returns_correct_count() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[
                0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03,
                0x02, 0x03, 0x00, 0x00, 0x00, 0x04, 0x05, 0x06, 0x03
            ]), 
            256, 
            1024
        );
        assert_eq!(2, reader.count());
    }

    // Iterator trait ssanity check
    #[test]
    fn iterator_returns_correct_last() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[
                0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03,
                0x02, 0x03, 0x00, 0x00, 0x00, 0x04, 0x05, 0x06, 0x03
            ]), 
            256, 
            1024
        );
        assert_eq!(Some(vec!(0x04, 0x05, 0x06)), reader.last());
    }

    #[test]
    fn reset_allows_iteration_from_beginning() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[
                0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03,
                0x02, 0x03, 0x00, 0x00, 0x00, 0x04, 0x05, 0x06, 0x03
            ]), 
            256, 
            1024
        );
        assert_eq!(Some(vec!(0x01, 0x02, 0x03)), reader.next());
        assert_eq!(Some(vec!(0x04, 0x05, 0x06)), reader.next());
        assert_eq!(None, reader.next());
        reader.reset();
        assert_eq!(Some(vec!(0x01, 0x02, 0x03)), reader.next());
        assert_eq!(Some(vec!(0x04, 0x05, 0x06)), reader.next());
        assert_eq!(None, reader.next());
    }

    #[test]
    fn jump_to_moves_to_record() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[
                0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03,
                0x02, 0x03, 0x00, 0x00, 0x00, 0x04, 0x05, 0x06, 0x03
            ]), 
            256, 
            1024
        );
        reader.jump_to(9, false);
        assert_eq!(Some(vec!(0x04, 0x05, 0x06)), reader.next());
        assert_eq!(None, reader.next());
    }

    #[test]
    fn jump_to_bad_position_without_back_on_fail_does_not_go_back() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[
                0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03,
                0x02, 0x03, 0x00, 0x00, 0x00, 0x04, 0x05, 0x06, 0x03
            ]), 
            256, 
            1024
        );
        reader.jump_to(10, false);
        assert_eq!(None, reader.next());
    }

    #[test]
    fn jump_to_bad_position_with_back_on_fail_does_not_go_back() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[
                0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03,
                0x02, 0x03, 0x00, 0x00, 0x00, 0x04, 0x05, 0x06, 0x03
            ]), 
            256, 
            1024
        );
        reader.jump_to(10, true);
        assert_eq!(Some(vec!(0x01, 0x02, 0x03)), reader.next());
        assert_eq!(Some(vec!(0x04, 0x05, 0x06)), reader.next());
        assert_eq!(None, reader.next());
    }

    #[test]
    fn size_returns_size_of_record_data() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[
                0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03,
                0x02, 0x04, 0x00, 0x00, 0x00, 0x04, 0x05, 0x06, 0x07, 0x03
            ]), 
            256, 
            1024
        );
        assert_eq!(3, reader.size());
        reader.next();
        assert_eq!(4, reader.size());
    }

    #[test]
    fn has_start_returns_true_when_start_byte_exists() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[
                0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
            ]), 
            256, 
            1024
        );
        assert!(reader.has_start());
    }

    #[test]
    fn has_start_returns_false_when_no_start_byte() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
            ]), 
            256, 
            1024
        );
        assert!(!reader.has_start());
    }

    #[test]
    fn size_returns_zero_when_no_start_byte() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[
                0x00, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03
            ]), 
            256, 
            1024
        );
        assert_eq!(0, reader.size());
    }

    #[test]
    fn size_returns_size_when_start_byte_is_present() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[
                0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03
            ]), 
            256, 
            1024
        );
        assert_eq!(3, reader.size());
    }

    #[test]
    fn has_end_returns_true_when_end_byte_exists() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[
                0x02, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03
            ]), 
            256, 
            1024
        );
        assert!(reader.has_end());
    }

    #[test]
    fn has_end_returns_false_when_no_end_byte() {
        let mut reader = JournalReader::new(
            get_mem(256, 1024, &[
                0x02, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
            ]), 
            256, 
            1024
        );
        assert!(!reader.has_end());
    }



}
