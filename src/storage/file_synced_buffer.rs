#![feature(alloc, heap_api)]

extern crate alloc;
extern crate core;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::str;
use std::cmp;
use alloc::heap;
use std::{mem, ptr, slice};
use std::collections::HashMap;
use storage::util;

use error::{ Error };
use storage::file_page::FilePage;





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

        let page_start_offset = page_index * page_size;
        let page_end_offset = page_start_offset + page_size;
        let end_offset = offset + len;

        let start_offset_in_page =
            if page_start_offset < offset {
                offset - page_start_offset
            } else {
                0
            };

        let end_offset_in_page =
            if end_offset > page_end_offset {
                page_size
            } else {
                end_offset - page_start_offset
            };

        (start_offset_in_page as u32, (end_offset_in_page - start_offset_in_page) as u32)
    }

    fn read_from_page(&mut self, index: u64, start: u32, len: u32) -> Result<Vec<u8>, Error> {

        match self.pages.get(&index) {
            Some(p) => return Ok(p.read(start, len)),
            None => ()
        };

        let seek_pos = index * self.page_size as u64;

        try!(self.file.seek(SeekFrom::Start(seek_pos)));

        let mut buf = vec![0; self.page_size as usize];
        
        let read_len = try!(self.file.read(buf.as_mut_slice()));
        buf.truncate(read_len);

        let mut page = FilePage::new(self.page_size, self.page_mem_align).unwrap();
        page.write(0, buf.as_slice());

        Ok(page.read(start, len))


    }

    pub fn read(&mut self, offset: u64, len: usize) -> Result<Vec<u8>, Error> {
        let (start, end) = self.calc_page_range(offset, len);

        println!("start {}", start);
        println!("end {}", end);

        let mut data = Vec::new();
        let mut total_len: usize = 0;

        for i in start..(end + 1) {
            let (start_in_page, len_in_page) = self.calc_page_section(i, offset, len);
            println!("sip {}", start_in_page);
            println!("lip {}", len_in_page);
            let partial_data = try!(self.read_from_page(i, start_in_page, len_in_page));
            let partial_len = partial_data.len();
            total_len += partial_data.len();

            data.extend(partial_data);

            if partial_len < len_in_page as usize { break };
        }

        data.truncate(total_len);
        Ok(data)

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
mod file_synced_buffer_tests {

    use std::str;
    use std::fs::{ File, OpenOptions};
    use std::io::Read;
    use storage::file_synced_buffer::FileSyncedBuffer;

    pub static BASE_PATH: &'static str = "./test_data/storage/file_synced_buffer/";

    fn path(filename: &str) -> String {
        BASE_PATH.to_string() + filename
    }

    fn file_r(filename: &str) -> File {
        OpenOptions::new()
            .read(true)
            .open(path(filename))
            .unwrap()
    }

    // read() tests
    #[test]
    fn read_returns_empty_on_blank_file() {
        let mut b = FileSyncedBuffer::new(file_r("blank.txt"), 16, 16, 1);
        assert_eq!(0, b.read(0, 128).unwrap().len());
    }

    #[test]
    fn read_returns_empty_when_reading_from_past_eof() {
        let mut b = FileSyncedBuffer::new(file_r("10.txt"), 16, 16, 1);
        assert_eq!(0, b.read(10, 10).unwrap().len());
    }

    #[test]
    fn read_truncates_data_when_reading_past_eof() {
        let mut b = FileSyncedBuffer::new(file_r("10.txt"), 16, 16, 1);
        assert_eq!(10, b.read(0, 16).unwrap().len());
    }

    #[test]
    fn read_reads_data_in_single_page() {
        let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16, 1);
        let res = b.read(35, 10).unwrap();
        assert_eq!(10, res.len());
        assert_eq!("etur adipi", str::from_utf8(res.as_slice()).unwrap());
    }

    #[test]
    fn read_reads_data_across_page_boundaries() {
        let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16, 1);
        let res = b.read(25, 10).unwrap();
        assert_eq!(10, res.len());
        assert_eq!("t, consect", str::from_utf8(res.as_slice()).unwrap());
    }

    #[test]
    fn read_reads_data_across_multiple_pages() {
        let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16, 1);
        let res = b.read(40, 35).unwrap();
        assert_eq!(35, res.len());
        assert_eq!("adipiscing elit. Integer ut imperdi", str::from_utf8(res.as_slice()).unwrap());
    }



}
