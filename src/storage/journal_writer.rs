#![feature(alloc, heap_api)]

extern crate alloc;
extern crate core;

use alloc::heap;
use std::{mem, ptr, slice};
use storage::journal::Journal;


pub struct JournalWriter {
    storage_origin: *const u8,
    record_offset: usize,
    write_offset: usize,
    capacity: usize,
    expand_size: usize,
    align: usize,
    is_writing: bool,
    uncommitted_size: usize
}
impl JournalWriter {

    pub fn new(
        storage_origin: *const u8,
        capacity: usize,
        expand_size: usize,
        align: usize,
    ) -> JournalWriter {
        JournalWriter {
            storage_origin: storage_origin,
            capacity: capacity,
            expand_size: expand_size,
            align: align,
            record_offset: 0,
            write_offset: 0,
            is_writing: false,
            uncommitted_size: 0
        }
    }

    fn record_ptr(&self) -> *mut u8 {
        return (
            self.storage_origin as usize + 
            self.record_offset 
        ) as *mut u8;
    }

    fn write_ptr(&self) -> *mut u8 {
        return (
            self.storage_origin as usize + 
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
        let new_capacity = (needed_capacity as f32 / self.expand_size as f32).ceil() as usize;

        // Allocate and record the new capacity
        unsafe {
            self.capacity = heap::reallocate_inplace(
                self.storage_origin as *mut u8, 
                self.capacity, 
                new_capacity, 
                self.align
            );
        }

        // Return whether or not enough space could be allocated 
        needed_capacity >= self.capacity
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_writing(&self) -> bool {
        self.is_writing
    }

    pub fn write(&mut self, data: &[u8]) {
        if self.is_writing { return }

        // Lock the writer so that no other writes can take place until a commit
        // or discard
        self.is_writing = true;

        // Allocate more memory to make room for the new record if necessary
        self.expand_if_needed(data.len());

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

    }

    pub fn commit(&mut self) {
        if !self.is_writing { return }

        // Write the ETX (end of text) marker
        unsafe { ptr::write(self.write_ptr(), 0x03) }

        // Move the record offset to the next record
        self.record_offset = self.uncommitted_size + 1;

        // Reset state
        self.write_offset = 0;
        self.uncommitted_size = 0;
        self.is_writing = false;
    }

    pub fn discard(&mut self) {
        if !self.is_writing { return }

        // Reinitialize all the uncommitted bytes to zero
        unsafe { ptr::write_bytes(self.record_ptr(), 0, self.uncommitted_size) }

        //Reset state
        self.write_offset = 0;
        self.uncommitted_size = 0;
        self.is_writing = false;
    }

}
