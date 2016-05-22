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

    pub fn storage_reallocated(&mut self, new_storage_origin: *const u8, new_capacity: usize) {
        self.storage_origin = Some(new_storage_origin);
        self.capacity = new_capacity;
    }

    pub fn reset(&mut self) {
        self.record_offset = 0;
    }

    pub fn jump_to(&mut self, offset: usize) -> bool {
        // Move to the requested offset, but remember the old one
        // so we can fall back to it if the record is incomplete
        let old_offset = self.record_offset; 
        self.record_offset = offset;

        // Check to make sure the start and end bytes are present
        // (this means the record is complete and committed). If
        // so, return true to signify that the record can be read.
        if self.has_start() && self.has_end() {
            return true;
        }

        // Otherwise return false to indicate that the data at the
        // current position is not a valid and complete record
        false
    }

    pub fn next(&mut self) -> bool {
        let new_offset = self.record_offset + 1 + 4 + self.size() as usize + 1;
        self.jump_to(new_offset) 
    }

    pub fn size(&self) -> u32 {
        *self.size_ref()
    }

    pub fn read(&self) -> Option<Vec<u8>> {
        if self.has_start() && self.has_end() {
            Some(self.data_slice().to_vec())
        } else {
            None
        }
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
