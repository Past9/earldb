extern crate alloc;
extern crate core;

use std::cmp;
use alloc::heap;
use std::{mem, ptr, slice};


pub struct FilePage {
    origin: *const u8,
    max_size: u32,
    actual_size: u32
}
impl FilePage {

    pub fn new(
        max_size: u32
    ) -> Option<FilePage> {
        if !FilePage::check_mem_params(max_size) { return None }; 
        
        let origin = unsafe { heap::allocate(max_size as usize, mem::size_of::<usize>()) };

        if origin.is_null() { return None }

        unsafe { ptr::write_bytes::<u8>(origin, 0x0, max_size as usize) };
         
        Some(FilePage {
            origin: origin,
            max_size: max_size,
            actual_size: 0
        })
    }

    fn ptr(&self, offset: u32) -> *const u8 {
        (self.origin as usize + offset as usize) as *const u8
    }

    fn ptr_mut(&mut self, offset: u32) -> *mut u8 {
        (self.origin as usize + offset as usize) as *mut u8
    }

    fn check_mem_params(
        max_size: u32
    ) -> bool {
        // Max size must be a power of 2 
        if !(max_size as usize).is_power_of_two() { return false }
        // If all checks pass, return true
        true
    }

    pub fn write(&mut self, offset: u32, data: &[u8]) {
        let c_offset = offset as usize;
        let c_max_size = self.max_size as usize;

        let end_offset = cmp::min(c_offset + data.len(), c_max_size);

        if c_offset > end_offset { return }

        let trunc_len = end_offset - c_offset;

        let dest = unsafe { slice::from_raw_parts_mut(self.ptr_mut(offset), trunc_len) };
        dest.clone_from_slice(&data[0..trunc_len]);

        let new_size = end_offset as u32;
        if new_size > self.actual_size { self.actual_size = new_size }
    }

    pub fn read(&self, offset: u32, len: u32) -> Vec<u8> {
        if offset >= self.actual_size { return Vec::new() }

        let end_offset = cmp::min(offset + len, self.actual_size);

        if offset > end_offset { return Vec::new() }

        let trunc_len = end_offset - offset;

        let src = unsafe { slice::from_raw_parts(self.ptr(offset), trunc_len as usize) };
        let mut dst = vec![0; trunc_len as usize];
        dst.copy_from_slice(src);
        dst
    }

    pub fn truncate(&mut self, len: u32) {
        if len >= self.actual_size { return }
        if len >= self.max_size { return }
        self.actual_size = len;
        unsafe { ptr::write_bytes::<u8>(self.ptr_mut(len), 0x0, (self.max_size - len) as usize) };
    }

    pub fn get_max_size(&self) -> u32 {
        self.max_size
    }

    pub fn get_actual_size(&self) -> u32 {
        self.actual_size
    }

}




