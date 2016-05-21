#![feature(alloc, heap_api)]

extern crate alloc;
extern crate core;

use alloc::heap;
use std::{mem, ptr, slice};
use storage::journal::Journal;


pub struct JournalReader {
    storage_origin: *const u8,
    record_offset: usize,
    capacity: usize,
    has_start: bool,
    size: u32,
    has_end: bool
}
impl JournalReader {
    
    pub fn new(
        storage_origin: *const u8,
        initial_capacity: usize
    ) -> JournalReader {
        let mut reader = JournalReader {
            storage_origin: storage_origin,
            record_offset: 0,
            capacity: initial_capacity,
            has_start: false,
            size: 0,
            has_end: false
        };
        reader.eval_current();
        reader
    }

    fn ptr_at_offset(&self, offset: usize) -> *const u8 {
        return (
            self.storage_origin as usize + 
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
        unsafe { slice::from_raw_parts(self.ptr_at_offset(5), self.size as usize) }
    }

    fn end_byte_ref(&self) -> &u8 {
        unsafe { &*(self.ptr_at_offset(1 + 4 + self.size as usize) as *const u8) }
    }

    fn eval_current(&mut self) {
        // Determine if the start/STX byte is set (is the byte at the
        // record pointer 0x02?)
        self.has_start = *self.start_byte_ref() == 0x02;

        // Determine the size of the record (the 4 bytes after the STX 
        // marker are a u32 representing the size of the record data in
        // bytes)
        self.size = *self.size_ref();

        // Determine if the end/ETX byte is set (is the byte immediately 
        // following the data 0x03?)
        self.has_end = *self.end_byte_ref() == 0x03;
    }

    fn jump_to(&mut self, offset: usize) -> bool {
        // Move to the requested offset, but remember the old one
        // so we can fall back to it if the record is incomplete
        let old_offset = self.record_offset; 
        self.record_offset = offset;

        // Get the metadata for the record (if any) at the current 
        // offset
        self.eval_current();

        // Check to make sure the start and end bytes are present
        // (this means the record is complete and committed). If
        // so, return true to signify that the record can be read.
        if self.has_start && self.has_end {
            return true;
        }

        // Otherwise, go back to the old offset and return false
        // to indicate that there is no valid record to be read
        self.record_offset = old_offset;
        false
    }

    fn next(&mut self) -> bool {
        let new_offset = self.record_offset + 1 + 4 + self.size as usize + 1;
        self.jump_to(new_offset) 
    }

    fn size(&self) -> u32 {
        self.size
    }

    fn has_start(&self) -> bool {
        self.has_start
    }

    fn has_end(&self) -> bool {
        self.has_end
    }

    fn read(&mut self) -> Option<Vec<u8>> {
        Some(self.data_slice().to_vec())
    }


}
