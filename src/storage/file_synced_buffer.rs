#![feature(alloc, heap_api)]

extern crate alloc;
extern crate core;

use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::str;
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

    pub fn update(&mut self, offset: u32, data: &[u8]) {
        let dest = unsafe { slice::from_raw_parts_mut(self.ptr_mut(offset), data.len()) };
        dest.clone_from_slice(data);
    }

    pub fn read(&self, offset: u32, len: u32) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr(offset), len as usize) }
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



