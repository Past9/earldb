#![feature(alloc, heap_api)]

extern crate alloc;
extern crate core;

use alloc::heap;
use std::{mem, ptr, slice};
use storage::binary_storage::BinaryStorage;


pub struct MemoryBinaryStorage {
    origin: *const u8,
    is_open: bool,
    capacity: usize,
    expand_size: usize,
    use_txn_boundary: bool,
    txn_boundary: usize,
    align: usize
}
impl MemoryBinaryStorage {

    pub fn new(
        origin: *const u32,
        initial_capacity: usize, 
        expand_size: usize, 
        use_txn_boundary: bool,
        align: usize
    ) -> MemoryBinaryStorage {

        let origin = unsafe { heap::allocate(initial_capacity, align) as *const u8 };

        MemoryBinaryStorage {
            origin: origin,
            is_open: false,
            capacity: initial_capacity,
            expand_size: expand_size,
            use_txn_boundary: use_txn_boundary,
            txn_boundary: 0,
            align: align
        }
    }

    fn ptr<T>(&self, offset: usize) -> *const T {
        (self.origin as usize + offset) as *const T
    }

    fn ptr_mut<T>(&mut self, offset: usize) -> *mut T {
        (self.origin as usize + offset) as *mut T
    }

    fn write<T>(&mut self, offset: usize, data: &T) -> bool {
        if !self.is_open { return false }
        if self.use_txn_boundary && offset < self.txn_boundary { return false }
        if !self.expand(offset + mem::size_of::<T>()) { return false }

        unsafe { ptr::write(self.ptr_mut(offset), data) }

        /*
        unsafe {
            let src_ptr: *const T = mem::transmute(data);
            ptr::copy(src_ptr, self.ptr_mut(offset), 1);
        }
        */
        true
    }

    fn read<T: Copy>(&self, offset: usize) -> Option<T> {
        if !self.is_open { return None }
        if self.use_txn_boundary && (offset + mem::size_of::<T>()) >= self.txn_boundary { return None }

        unsafe { Some(ptr::read(self.ptr(offset))) }


        /*
        unsafe {
            Some(*(self.ptr(offset) as *const T))
        }
        */
    }

}
impl BinaryStorage for MemoryBinaryStorage {

    fn open(&mut self) -> bool {
        if self.is_open { return false }
        self.is_open = true;
        true
    }

    fn close(&mut self) -> bool {
        if !self.is_open { return false }
        self.is_open = false;
        true
    }

    fn w_i8(&mut self, offset: usize, data: i8) -> bool {
        self.write(offset, &data)
    }

    fn w_i16(&mut self, offset: usize, data: i16) -> bool {
        self.write(offset, &data)
    }

    fn w_i32(&mut self, offset: usize, data: i32) -> bool {
        self.write(offset, &data)
    }

    fn w_i64(&mut self, offset: usize, data: i64) -> bool {
        self.write(offset, &data)
    }


    fn w_u8(&mut self, offset: usize, data: u8) -> bool {
        self.write(offset, &data)
    }

    fn w_u16(&mut self, offset: usize, data: u16) -> bool {
        self.write(offset, &data)
    }

    fn w_u32(&mut self, offset: usize, data: u32) -> bool {
        self.write(offset, &data)
    }

    fn w_u64(&mut self, offset: usize, data: u64) -> bool {
        self.write(offset, &data)
    }


    fn w_f32(&mut self, offset: usize, data: f32) -> bool {
        self.write(offset, &data)
    }

    fn w_f64(&mut self, offset: usize, data: f64) -> bool {
        self.write(offset, &data)
    }


    fn w_bool(&mut self, offset: usize, data: bool) -> bool {
        self.write(offset, &data)
    }


    fn w_slice(&mut self, offset: usize, data: &[u8]) -> bool {
        unimplemented!();
    }

    fn w_str(&mut self, offset: usize, data: &str) -> bool {
        unimplemented!();
    }



    fn r_i8(&self, offset: usize) -> Option<i8> {
        self.read(offset)
    }

    fn r_i16(&self, offset: usize) -> Option<i16> {
        self.read(offset)
    }

    fn r_i32(&self, offset: usize) -> Option<i32> {
        self.read(offset)
    }

    fn r_i64(&self, offset: usize) -> Option<i64> {
        self.read(offset)
    }


    fn r_u8(&self, offset: usize) -> Option<u8> {
        self.read(offset)
    }

    fn r_u16(&self, offset: usize) -> Option<u16> {
        self.read(offset)
    }

    fn r_u32(&self, offset: usize) -> Option<u32> {
        self.read(offset)
    }

    fn r_u64(&self, offset: usize) -> Option<u64> {
        self.read(offset)
    }


    fn r_f32(&self, offset: usize) -> Option<f32> {
        self.read(offset)
    }

    fn r_f64(&self, offset: usize) -> Option<f64> {
        self.read(offset)
    }


    fn r_bool(&self, offset: usize) -> Option<bool> {
        self.read(offset)
    }


    fn r_slice(&self, offset: usize, len: usize) -> Option<&[u8]> {
        unimplemented!();
    }

    fn r_str(&self, offset: usize, len: usize) -> Option<&str> {
        unimplemented!();
    }


    fn fill(&mut self, start: Option<usize>, end: Option<usize>, val: u8) -> bool {
        if !self.is_open { return false }

        let start_offset = match start {
            Some(s) => s,
            None => 0
        };

        if start_offset >= self.capacity { return false }

        let end_offset = match end {
            Some(e) => e,
            None => self.capacity
        };

        if end_offset <= start_offset { return false }
        if end_offset >= self.capacity { return false }

        unsafe { ptr::write_bytes::<u8>(self.ptr_mut(start_offset), val, end_offset - start_offset) }

        true
    }

    fn assert_filled(&self, start: Option<usize>, end: Option<usize>, val: u8) -> bool {
        unimplemented!();
    }


    fn get_use_txn_boundary(&self) -> bool {
        self.use_txn_boundary
    }

    fn set_use_txn_boundary(&mut self, val: bool) {
        self.use_txn_boundary = val;
    }


    fn get_txn_boundary(&self) -> usize {
        self.txn_boundary
    }

    fn set_txn_boundary(&mut self, offset: usize) -> bool {
        if !self.is_open { return false }
        if !self.use_txn_boundary { return false }
        if offset > self.capacity { return false }

        self.txn_boundary = offset;
        true
    }


    fn get_expand_size(&self) -> usize {
        self.expand_size
    }

    fn set_expand_size(&mut self) -> usize {
        // TODO: check memory params
        unimplemented!();
    }


    fn expand(&mut self, min_capacity: usize) -> bool {
        unimplemented!();
    }

    fn capacity(&self) -> usize {
        if !self.is_open { return 0 }
        self.capacity
    }


    fn is_open(&self) -> bool {
        self.is_open
    }


}
