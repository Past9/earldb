#![feature(alloc, heap_api)]

extern crate alloc;
extern crate core;

use std::cmp;
use alloc::heap;
use std::{mem, ptr, slice};
use std::collections::HashMap;
use storage::util;


pub struct FilePage {
    origin: *const u8,
    max_size: u32,
    actual_size: u32,
    align: usize
}
impl FilePage {

    pub fn new(
        max_size: u32,
        align: usize
    ) -> Option<FilePage> {
        if !FilePage::check_mem_params(align, max_size) { return None }; 
        
        let origin = unsafe { heap::allocate(max_size as usize, align) };

        if origin.is_null() { return None }

        unsafe { ptr::write_bytes::<u8>(origin, 0x0, max_size as usize) };
         
        Some(FilePage {
            origin: origin,
            max_size: max_size,
            actual_size: 0,
            align: align
        })
    }

    fn ptr(&self, offset: u32) -> *const u8 {
        (self.origin as usize + offset as usize) as *const u8
    }

    fn ptr_mut(&mut self, offset: u32) -> *mut u8 {
        (self.origin as usize + offset as usize) as *mut u8
    }

    fn check_mem_params(
        align: usize,
        max_size: u32,
    ) -> bool {
        // alignment must be greater than zero
        if align < 1 { return false }
        // Max size must be a power of 2 
        if !util::is_power_of_two(max_size as usize) { return false }
        // Alignment must be a power of 2
        if !util::is_power_of_two(align) { return false }
        // Alignment must be no larger than max size
        if align > (max_size as usize) { return false }
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

        self.actual_size = end_offset as u32;
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

    pub fn get_max_size(&self) -> u32 {
        self.max_size
    }

    pub fn get_actual_size(&self) -> u32 {
        self.actual_size
    }

    pub fn get_align(&self) -> usize {
        self.align
    }

}




#[cfg(test)]
mod file_page_tests {

    use storage::file_page::FilePage;

    // FilePage::new() tests
    #[test]
    fn new_returns_none_when_align_is_zero() {
        let p = FilePage::new(256, 0);
        assert!(p.is_none());
    }

    #[test]
    fn new_returns_none_when_max_size_not_power_of_2() {
        let p = FilePage::new(257, 128);
        assert!(p.is_none());
    }

    #[test]
    fn new_returns_none_when_align_not_power_of_2() {
        let p = FilePage::new(256, 129);
        assert!(p.is_none());
    }

    #[test]
    fn new_returns_none_when_align_larger_than_max_size() {
        let p = FilePage::new(256, 512);
        assert!(p.is_none());
    }

    #[test]
    fn new_returns_file_page_instance_when_checks_pass() {
        let p = FilePage::new(256, 256);
        assert!(p.is_some());
    }

    #[test]
    fn new_sets_max_size() {
        let p = FilePage::new(512, 256).unwrap();
        assert_eq!(512, p.get_max_size());
    }

    #[test]
    fn new_sets_align() {
        let mut p = FilePage::new(256, 256).unwrap();
        assert_eq!(256, p.get_align());
    }

    #[test]
    fn new_inits_memory_to_zeros() {
        let mut p = FilePage::new(256, 256).unwrap();
        p.write(255, &[0x0]);
        let data = p.read(0, 256);
        assert_eq!(256, data.len());
        for b in data {
            assert_eq!(b, 0x0);
        }
    }

    // FilePage::read() tests
    #[test]
    fn read_returns_empty_when_new() {
        let mut p = FilePage::new(256, 256).unwrap();
        let data = p.read(0, 4);
        assert_eq!(0, data.len());
    }

    #[test]
    fn read_returns_empty_when_reading_from_past_actual_size() {
        let mut p = FilePage::new(256, 256).unwrap();
        p.write(0, &[0x0, 0x1, 0x2, 0x3]);
        let data = p.read(4, 4);
        assert_eq!(0, data.len());
    }

    #[test]
    fn read_returns_remaining_data_when_past_actual_size() {
        let mut p = FilePage::new(256, 256).unwrap();
        p.write(0, &[0x0, 0x1, 0x2, 0x3]);
        let data = p.read(2, 4);
        assert_eq!(2, data.len());
        assert_eq!(vec!(0x2, 0x3), data);
    }

    #[test]
    fn read_past_actual_size_does_not_increase_actual_size() {
        let mut p = FilePage::new(256, 256).unwrap();
        p.write(0, &[0x0, 0x1, 0x2, 0x3]);
        assert_eq!(4, p.get_actual_size());
        p.read(2, 4);
        assert_eq!(4, p.get_actual_size());
    }

    #[test]
    fn read_returns_zeros_for_unwritten_data() {
        let mut p = FilePage::new(256, 256).unwrap();
        p.write(255, &[0x1]);
        assert_eq!(vec!(0x0, 0x0, 0x0, 0x0), p.read(0, 4));
        assert_eq!(vec!(0x0, 0x0, 0x0, 0x0), p.read(64, 4));
        assert_eq!(vec!(0x0, 0x0, 0x0, 0x0), p.read(128, 4));
        assert_eq!(vec!(0x0, 0x0, 0x0, 0x1), p.read(252, 4));
    }

    #[test]
    fn read_returns_written_data() {
        let mut p = FilePage::new(256, 256).unwrap();
        p.write(10, &[0x1, 0x2, 0x3, 0x4]);
        p.write(20, &[0x5, 0x6, 0x7, 0x8]);
        assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), p.read(10, 4));
        assert_eq!(vec!(0x5, 0x6, 0x7, 0x8), p.read(20, 4));
    }

    // FilePage::write() tests
    #[test] 
    fn write_writes_data_at_beginning() {
        let mut p = FilePage::new(256, 256).unwrap();
        p.write(0, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), p.read(0, 4)); 
    }

    #[test]
    fn write_writes_data_at_offset() {
        let mut p = FilePage::new(256, 256).unwrap();
        p.write(10, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), p.read(10, 4)); 
    }

    #[test]
    fn write_writes_remaining_data_until_end_when_writing_past_max_size() {
        let mut p = FilePage::new(256, 256).unwrap();
        p.write(254, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(vec!(0x1, 0x2), p.read(254, 2)); 
    }

    #[test]
    fn writes_nothing_when_starting_after_max_size() {
        let mut p = FilePage::new(256, 256).unwrap();
        p.write(256, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(vec!(0x0, 0x0), p.read(254, 2)); 
    }

    #[test]
    fn write_increases_actual_size() {
        let mut p = FilePage::new(256, 256).unwrap();
        assert_eq!(0, p.get_actual_size());
        p.write(0, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(4, p.get_actual_size());
        p.write(100, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(104, p.get_actual_size());
    }

    #[test]
    fn write_does_not_increase_actual_size_past_max_size() {
        let mut p = FilePage::new(256, 256).unwrap();
        assert_eq!(0, p.get_actual_size());
        p.write(0, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(4, p.get_actual_size());
        p.write(252, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(256, p.get_actual_size());
        p.write(400, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(256, p.get_actual_size());
    }

    // FilePage::get_max_size() tests
    #[test]
    fn get_max_size_returns_max_size() {
        let mut p = FilePage::new(256, 128).unwrap();
        assert_eq!(256, p.get_max_size());
    }

    #[test]
    fn get_max_size_does_not_change_on_writes() {
        let mut p = FilePage::new(256, 128).unwrap();
        assert_eq!(256, p.get_max_size());
        p.write(0, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(256, p.get_max_size());
        p.write(252, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(256, p.get_max_size());
        p.write(400, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(256, p.get_max_size());
    }

    // FilePage::get_actual_size() tests
    #[test]
    fn get_actual_size_returns_zero_when_new() {
        let mut p = FilePage::new(256, 128).unwrap();
        assert_eq!(0, p.get_actual_size());
    }

    #[test]
    fn get_actual_size_returns_actual_size() {
        let mut p = FilePage::new(256, 128).unwrap();
        assert_eq!(0, p.get_actual_size());
        p.write(0, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(4, p.get_actual_size());
        p.write(100, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(104, p.get_actual_size());
    }

    // FilePage::get_align() tests
    #[test]
    fn get_align_returns_alignment() {
        let mut p = FilePage::new(256, 128).unwrap();
        assert_eq!(128, p.get_align());
    }

}
