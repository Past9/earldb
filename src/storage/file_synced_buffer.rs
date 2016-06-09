#![feature(alloc, heap_api)]

extern crate alloc;
extern crate core;

use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::str;
use std::cmp;
use alloc::heap;
use std::{mem, ptr, slice};
use std::collections::HashMap;
use storage::util;



struct FilePage {
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

    pub fn read(&self, offset: u32, len: u32) -> &[u8] {
        if offset >= self.actual_size { return &[] }

        let end_offset = cmp::min(offset + len, self.actual_size);

        if offset > end_offset { return &[] }

        let trunc_len = end_offset - offset;

        unsafe { slice::from_raw_parts(self.ptr(offset), trunc_len as usize) }
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


pub struct FileSyncedBuffer {
    file: File,
    page_size: u32,
    max_pages: u16,
    page_mem_align: usize,
    pages: HashMap<u64, FilePage>
}
impl FileSyncedBuffer {

    pub fn new(
        file: File,
        page_size: u32,
        max_pages: u16,
        page_mem_align: usize
    ) -> FileSyncedBuffer {
        FileSyncedBuffer {
            file: file,
            page_size: page_size,
            max_pages: max_pages,
            page_mem_align: page_mem_align,
            pages: HashMap::new()
        }
    }

    fn calc_page_range(&self, offset: u64, length: usize) -> (u64, u64) {
        let page_size = self.page_size as u64;
        let len = length as u64;
        let mut start = offset / page_size; 
        let mut end = (offset + len) / page_size;
        (start as u64, end as u64)
    }

    fn calc_page_section(&self, page_index: u64, offset: u64, length: usize) -> (u32, u32) {
        let page_size = self.page_size as u64;
        let len = length as u64;
        let offset_in_page = (page_index * page_size) % offset;
        let mut len_in_page: u64 = 0;
        if len + offset_in_page > page_size{
            len_in_page = page_size - offset_in_page;
        } else {
            len_in_page = len - offset_in_page; 
        }
        (offset_in_page as u32, len_in_page as u32)
    }

    fn get_page(&mut self, index: u64) -> Option<&FilePage> {

        match self.pages.get(&index) {
            Some(p) => return Some(p),
            None => ()
        };

        let seekPos = index * self.page_size as u64;

        match self.file.seek(SeekFrom::Start(seekPos)) {
            Ok(s) => if s != seekPos { return None },
            Err(_) => return None
        };



        None

        

    }

    /*
    fn get_page_mut(&mut self, index: u64) -> Option<&mut FilePage> {
        match self.pages.get_mut(&index) {
            Some(p) => Some(p),
            None => {
            }
        }
    }
    */

    pub fn read(&mut self, offset: u64, len: usize) -> &[u8] {
        let (start, end) = self.calc_page_range(offset, len);

        let slice_ptr = unsafe { heap::allocate(len, 8) as *mut u8 };
        let mut total_len: usize = 0;

        for i in start..(end + 1) {
            let (start_in_page, len_in_page) = self.calc_page_section(i, offset, len);
            match self.get_page(i) {
                Some(mut p) => {
                    let partial_data = p.read(start_in_page, len_in_page);                    

                    if partial_data.len() < 1 { break };

                    unsafe {
                        ptr::copy(
                            partial_data.first().unwrap(),
                            slice_ptr,
                            partial_data.len()
                        );
                    }

                    total_len += partial_data.len();

                    if partial_data.len() < len_in_page as usize { break };
                },
                None => break
            };
        }

        unsafe { slice::from_raw_parts(slice_ptr as *const u8, total_len) } 
    }

    pub fn update(&mut self, offset: u64, data: &[u8]) {
        unimplemented!();
    }

    pub fn truncate(&mut self, len: u64) {
        unimplemented!();
    }

    pub fn get_page_size(&self) -> u32 {
        self.page_size
    }

    pub fn get_max_pages(&self) -> u16 {
        self.max_pages
    }

    pub fn set_max_pages(&mut self, pages: u16) {
        self.max_pages = pages;
        // TODO: Remove old pages
    }

    pub fn get_num_current_pages(&self) -> u16 {
        unimplemented!();
    }

    pub fn get_page_mem_align(&self) -> usize {
        self.page_mem_align
    }

}



#[cfg(test)]
mod file_page_tests {

    use storage::file_synced_buffer::{
        FilePage,
        FileSyncedBuffer
    };

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
        for &b in data {
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
        assert_eq!(&[0x2, 0x3], data);
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
        assert_eq!(&[0x0, 0x0, 0x0, 0x0], p.read(0, 4));
        assert_eq!(&[0x0, 0x0, 0x0, 0x0], p.read(64, 4));
        assert_eq!(&[0x0, 0x0, 0x0, 0x0], p.read(128, 4));
        assert_eq!(&[0x0, 0x0, 0x0, 0x1], p.read(252, 4));
    }

    #[test]
    fn read_returns_written_data() {
        let mut p = FilePage::new(256, 256).unwrap();
        p.write(10, &[0x1, 0x2, 0x3, 0x4]);
        p.write(20, &[0x5, 0x6, 0x7, 0x8]);
        assert_eq!(&[0x1, 0x2, 0x3, 0x4], p.read(10, 4));
        assert_eq!(&[0x5, 0x6, 0x7, 0x8], p.read(20, 4));
    }

    // FilePage::write() tests
    #[test] 
    fn write_writes_data_at_beginning() {
        let mut p = FilePage::new(256, 256).unwrap();
        p.write(0, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(&[0x1, 0x2, 0x3, 0x4], p.read(0, 4)); 
    }

    #[test]
    fn write_writes_data_at_offset() {
        let mut p = FilePage::new(256, 256).unwrap();
        p.write(10, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(&[0x1, 0x2, 0x3, 0x4], p.read(10, 4)); 
    }

    #[test]
    fn write_writes_remaining_data_until_end_when_writing_past_max_size() {
        let mut p = FilePage::new(256, 256).unwrap();
        p.write(254, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(&[0x1, 0x2], p.read(254, 2)); 
    }

    #[test]
    fn writes_nothing_when_starting_after_max_size() {
        let mut p = FilePage::new(256, 256).unwrap();
        p.write(256, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(&[0x0, 0x0], p.read(254, 2)); 
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




#[cfg(test)]
mod file_synced_buffer_tests {

    // read() tests
    #[test]
    fn read_returns_empty_on_blank_file() {
    }

    #[test]
    fn read_returns_empty_when_reading_from_past_eof() {
    }

    #[test]
    fn read_truncates_data_when_reading_past_eof() {
    }

    #[test]
    fn read_reads_data_across_page_boundaries() {
    }

    #[test]
    fn read_reads_data_across_multiple_pages() {
    }

    #[test]
    fn read_returns_none_when_page_alloc_fails() {
    }


}

