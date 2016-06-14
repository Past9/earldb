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
        let mut end = (offset + len - 1) / page_size;
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

    fn insert_page(&mut self, index: u64, page: FilePage) {
        self.pages.insert(index, page); 
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
        let data = page.read(start, len);

        self.insert_page(index, page);

        Ok(data)
    }

    pub fn read(&mut self, offset: u64, len: usize) -> Result<Vec<u8>, Error> {
        let (start, end) = self.calc_page_range(offset, len);

        let mut data = Vec::new();
        let mut total_len: usize = 0;

        for i in start..(end + 1) {
            let (start_in_page, len_in_page) = self.calc_page_section(i, offset, len);
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
        let page_size = self.page_size as u64;
        let (start, end) = self.calc_page_range(offset, data.len());

        println!("(start, end) = ({}, {})", start, end);

        let mut data_offset: u64 = 0;

        for i in start..(end + 1) {
            let (start_in_page, len_in_page) = self.calc_page_section(i, offset, data.len());
            if i == start { data_offset = (start * page_size) + start_in_page as u64 }
            let mut page = match self.pages.get_mut(&i) {
                Some(p) => p,
                None => continue 
            };

            let start_in_data = i * page_size + start_in_page as u64 - data_offset;
            let end_in_data = start_in_data + (len_in_page as u64);
            println!("(start_in_data, end_in_data) = ({}, {})", start_in_data, end_in_data);
            page.write(
                start_in_page,
                &data[(start_in_data as usize)..(end_in_data as usize)]
            );
        }
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

    use std::thread;
    use std::time::Duration;
    use std::str;
    use std::fs;
    use std::panic;
    use std::fs::{ File, OpenOptions };
    use std::io::{ Read, Write };

    use uuid::{ Uuid, UuidVersion };

    use storage::file_synced_buffer::FileSyncedBuffer;
    use error::Error;


    pub static BASE_PATH: &'static str = "./test_data/storage/file_synced_buffer/";

    fn rnd_path() -> String {
        BASE_PATH.to_string() 
            + Uuid::new_v4().simple().to_string().as_str()
            + ".tmp"
    }

    fn path(filename: &str) -> String {
        BASE_PATH.to_string() + filename
    }

    fn file_r(filename: &str) -> File {
        OpenOptions::new()
            .read(true)
            .open(path(filename))
            .unwrap()
    }

    fn file_tmp_rw() -> (File, String) {
        let path = rnd_path();
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .open(path.clone())
            .unwrap();
        (file, path)
    }

    fn rm_tmp(filename: String) {
        fs::remove_file(filename).unwrap();
    }

    // read() tests
    /*
    #[test]
    fn read_bubbles_io_errors() {
        let (mut f, p) = file_tmp_rw(); 
        println!("before len: {}", f.metadata().unwrap().len());
        f.write(&[0x1, 0x2, 0x3, 0x4]);
        f.sync_all().unwrap();
        rm_tmp(p);
        f.sync_all().unwrap();
        f.write(&[0x5, 0x6, 0x7]);
        f.sync_all().unwrap();
        println!("after len: {}", f.metadata().unwrap().len());
        let mut b = FileSyncedBuffer::new(f, 16, 16, 1);
        let res = b.read(0, 10);
        println!("DATA: {:?}", res.unwrap());

        assert!(false);
        //assert!(res.is_err());
        //assert!(match res.unwrap_err() {
        //    Error::Io(_) => true,
        //    _ => false
        //});
    }
    */

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
    fn read_only_returns_data_present_in_file() {
        let mut b = FileSyncedBuffer::new(file_r("10.txt"), 4, 4, 1);
        let res = b.read(0, 100).unwrap();
        assert_eq!(10, res.len());
        assert_eq!("Lorem ips\n", str::from_utf8(res.as_slice()).unwrap());
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
    fn read_reads_data_across_multiple_page_boundaries() {
        let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16, 1);
        let res = b.read(40, 35).unwrap();
        assert_eq!(35, res.len());
        assert_eq!("adipiscing elit. Integer ut imperdi", str::from_utf8(res.as_slice()).unwrap());
    }

    // update() tests
    #[test]
    fn update_writes_to_subset_of_first_page() {
        let (mut f, p) = file_tmp_rw();

        f.write(&[0x1, 0x2, 0x3, 0x4]).unwrap();
        let mut b = FileSyncedBuffer::new(f, 4, 16, 1);
        assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), b.read(0, 4).unwrap());
        b.update(1, &[0x5, 0x6]); 
        assert_eq!(vec!(0x1, 0x5, 0x6, 0x4), b.read(0, 4).unwrap());

        rm_tmp(p);
    }

    #[test]
    fn update_writes_to_subset_of_nth_page() {
        let (mut f, p) = file_tmp_rw();

        f.write(&[0x0, 0x0, 0x0, 0x0, 0x1, 0x2, 0x3, 0x4]).unwrap();
        let mut b = FileSyncedBuffer::new(f, 4, 16, 1);
        assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), b.read(4, 4).unwrap());
        b.update(5, &[0x5, 0x6]); 
        assert_eq!(vec!(0x1, 0x5, 0x6, 0x4), b.read(4, 4).unwrap());

        rm_tmp(p);
    }

    #[test]
    fn update_writes_to_whole_first_page() {
        let (mut f, p) = file_tmp_rw();

        f.write(&[0x1, 0x2, 0x3, 0x4]).unwrap();
        let mut b = FileSyncedBuffer::new(f, 4, 16, 1);
        assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), b.read(0, 4).unwrap());
        b.update(0, &[0x5, 0x6, 0x7, 0x8]); 
        assert_eq!(vec!(0x5, 0x6, 0x7, 0x8), b.read(0, 4).unwrap());

        rm_tmp(p);
    }

    #[test]
    fn update_writes_to_whole_nth_page() {
        let (mut f, p) = file_tmp_rw();

        f.write(&[0x0, 0x0, 0x0, 0x0, 0x1, 0x2, 0x3, 0x4]).unwrap();
        let mut b = FileSyncedBuffer::new(f, 4, 16, 1);
        assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), b.read(4, 4).unwrap());
        b.update(4, &[0x5, 0x6, 0x7, 0x8]); 
        assert_eq!(vec!(0x5, 0x6, 0x7, 0x8), b.read(4, 4).unwrap());

        rm_tmp(p);
    }

    #[test]
    fn update_writes_across_page_boundaries_from_first_page() {
        let (mut f, p) = file_tmp_rw();

        f.write(&[0x0, 0x0, 0x1, 0x2, 0x3, 0x4]).unwrap();
        let mut b = FileSyncedBuffer::new(f, 4, 16, 1);
        assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), b.read(2, 4).unwrap());
        b.update(2, &[0x5, 0x6, 0x7, 0x8]); 
        assert_eq!(vec!(0x5, 0x6, 0x7, 0x8), b.read(2, 4).unwrap());

        rm_tmp(p);
    }

    #[test]
    fn update_writes_across_page_boundaries_from_nth_page() {
        let (mut f, p) = file_tmp_rw();

        f.write(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x2, 0x3, 0x4]).unwrap();
        let mut b = FileSyncedBuffer::new(f, 4, 16, 1);
        assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), b.read(6, 4).unwrap());
        b.update(6, &[0x5, 0x6, 0x7, 0x8]); 
        assert_eq!(vec!(0x5, 0x6, 0x7, 0x8), b.read(6, 4).unwrap());

        rm_tmp(p);
    }

    #[test]
    fn update_writes_across_multiple_page_boundaries_from_first_page() {
        let (mut f, p) = file_tmp_rw();

        f.write(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]).unwrap();
        let mut b = FileSyncedBuffer::new(f, 4, 16, 1);
        assert_eq!(vec!(0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8), b.read(6, 8).unwrap());
        b.update(6, &[0x8, 0x7, 0x6, 0x5, 0x4, 0x3, 0x2, 0x1]); 
        assert_eq!(vec!(0x8, 0x7, 0x6, 0x5, 0x4, 0x3, 0x2, 0x1), b.read(6, 8).unwrap());

        rm_tmp(p);
    }

    #[test]
    fn update_writes_across_multiple_page_boundaries_from_nth_page() {
        let (mut f, p) = file_tmp_rw();

        f.write(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]).unwrap();
        let mut b = FileSyncedBuffer::new(f, 4, 16, 1);
        assert_eq!(vec!(0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8), b.read(6, 8).unwrap());
        b.update(6, &[0x8, 0x7, 0x6, 0x5, 0x4, 0x3, 0x2, 0x1]); 
        assert_eq!(vec!(0x8, 0x7, 0x6, 0x5, 0x4, 0x3, 0x2, 0x1), b.read(6, 8).unwrap());

        rm_tmp(p);
    }

    #[test]
    fn update_only_writes_to_cached_pages() {
        let (mut f, p) = file_tmp_rw();

        f.write(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]).unwrap();
        let mut b = FileSyncedBuffer::new(f, 4, 16, 1);
        assert_eq!(vec!(0x0, 0x0, 0x1, 0x2), b.read(4, 4).unwrap());
        assert_eq!(vec!(0x7, 0x8), b.read(12, 4).unwrap());
        b.update(6, &[0x8, 0x7, 0x6, 0x5, 0x4, 0x3, 0x2, 0x1]); 
        assert_eq!(vec!(0x0, 0x0, 0x8, 0x7), b.read(4, 4).unwrap());
        assert_eq!(vec!(0x3, 0x4, 0x5, 0x6), b.read(8, 4).unwrap());
        assert_eq!(vec!(0x2, 0x1), b.read(12, 4).unwrap());

        rm_tmp(p);
    }




}
