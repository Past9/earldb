#![feature(alloc, heap_api)]

extern crate alloc;
extern crate core;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::iter::FromIterator;
use std::str;
use std::cmp;
use alloc::heap;
use std::{mem, ptr, slice};
use std::collections::{ HashMap, VecDeque };
use storage::util;

use error::{ Error };
use storage::file_page::FilePage;


pub struct FileSyncedBuffer {
    file: File,
    page_size: u32,
    max_pages: u32,
    page_mem_align: usize,
    pages: HashMap<u64, FilePage>,
    page_insertions: VecDeque<u64>
}
impl FileSyncedBuffer {

    pub fn new(
        file: File,
        page_size: u32,
        max_pages: u32,
        page_mem_align: usize
    ) -> FileSyncedBuffer {
        FileSyncedBuffer {
            file: file,
            page_size: page_size,
            max_pages: max_pages,
            page_mem_align: page_mem_align,
            pages: HashMap::new(),
            page_insertions: VecDeque::new()
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

    fn remove_oldest_page(&mut self) {
        if self.page_insertions.len() == 0 { return }
        match self.page_insertions.pop_front() {
            Some(i) => { 
                self.pages.remove(&i); 
            },
            None => self.remove_oldest_page()
        }
    }

    fn remove_oldest_pages(&mut self, room_for: u32) {
        let rf = room_for as usize;
        let max_pages = self.max_pages as usize;
        if self.pages.len() + rf <= max_pages { return }
        let num_to_rm = self.pages.len() - max_pages + rf;
        for _ in 0..(num_to_rm) { self.remove_oldest_page() }
    }

    fn remove_page(&mut self, index: u64) {
        self.pages.remove(&index);
        match self.page_insertions.iter().position(|&x| x == index) {
            Some(i) => self.page_insertions.remove(i),
            None => return
        };
    }

    fn insert_page(&mut self, index: u64, page: FilePage) {
        if self.max_pages == 0 { return }
        self.remove_oldest_pages(1);
        self.pages.insert(index, page); 
        self.page_insertions.push_back(index);
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
            page.write(
                start_in_page,
                &data[(start_in_data as usize)..(end_in_data as usize)]
            );
        }
    }

    pub fn truncate(&mut self, len: u64) {
        if len == 0 { 
            self.pages.clear();
            self.page_insertions.clear();
            return;
        }

        let page_size = self.page_size as u64;

        let last_page = len / page_size;         
        let last_page_len = (len % page_size) as u32;
        let to_remove = Vec::from_iter(
            self.page_insertions.iter()
                .filter(|&&p| p > last_page)
                .map(|&p| p.clone())
        );


        for p in to_remove {
            self.remove_page(p);
        }

        match self.pages.get_mut(&last_page) {
            Some(p) => {
                p.truncate(last_page_len);
            },
            None => ()
        }

    }

    pub fn get_page_size(&self) -> u32 {
        self.page_size
    }

    pub fn get_max_pages(&self) -> u32 {
        self.max_pages
    }

    pub fn set_max_pages(&mut self, pages: u32) {
        self.max_pages = pages;
        // TODO: Remove old pages
    }

    pub fn get_num_current_pages(&self) -> u32 {
        self.pages.len() as u32
    }

    pub fn get_current_page_insertions(&self) -> Vec<u64> {
        Vec::from_iter(self.page_insertions.iter().map(|&x| x))
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
        fs::remove_file(filename).unwrap()
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

    // page caching tests
    #[test]
    fn reads_1_page_when_caching_0_pages() {
        let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 0, 1);
        assert_eq!(0, b.get_num_current_pages());
        let res = b.read(4, 4).unwrap();
        assert_eq!(4, res.len());
        assert_eq!("m ip", str::from_utf8(res.as_slice()).unwrap());
        assert_eq!(0, b.get_num_current_pages());
    }

    #[test]
    fn reads_multiple_pages_when_caching_0_pages() {
        let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 0, 1);
        assert_eq!(0, b.get_num_current_pages());
        let res = b.read(4, 32).unwrap();
        assert_eq!(32, res.len());
        assert_eq!("m ipsum dolor sit amet, consecte", str::from_utf8(res.as_slice()).unwrap());
        assert_eq!(0, b.get_num_current_pages());
    }

    #[test]
    fn reads_1_page_when_caching_1_page() {
        let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 1, 1);
        assert_eq!(0, b.get_num_current_pages());
        let res = b.read(4, 4).unwrap();
        assert_eq!(4, res.len());
        assert_eq!("m ip", str::from_utf8(res.as_slice()).unwrap());
        assert_eq!(1, b.get_num_current_pages());
    }

    #[test]
    fn reads_multiple_pages_when_caching_1_page() {
        let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 1, 1);
        assert_eq!(0, b.get_num_current_pages());
        let res = b.read(4, 32).unwrap();
        assert_eq!(32, res.len());
        assert_eq!("m ipsum dolor sit amet, consecte", str::from_utf8(res.as_slice()).unwrap());
        assert_eq!(1, b.get_num_current_pages());
    }

    #[test]
    fn reads_1_page_when_caching_multiple_pages() {
        let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16, 1);
        assert_eq!(0, b.get_num_current_pages());
        let res = b.read(4, 4).unwrap();
        assert_eq!(4, res.len());
        assert_eq!("m ip", str::from_utf8(res.as_slice()).unwrap());
        assert_eq!(1, b.get_num_current_pages());
    }

    #[test]
    fn reads_multiple_pages_when_caching_multiple_pages() {
        let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16, 1);
        assert_eq!(0, b.get_num_current_pages());
        let res = b.read(4, 32).unwrap();
        assert_eq!(32, res.len());
        assert_eq!("m ipsum dolor sit amet, consecte", str::from_utf8(res.as_slice()).unwrap());
        assert_eq!(3, b.get_num_current_pages());
    }

    #[test]
    fn only_caches_up_to_max_pages() {
        let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 3, 1);
        assert_eq!(0, b.get_num_current_pages());
        let res = b.read(4, 64).unwrap();
        assert_eq!(64, res.len());
        assert_eq!(
            "m ipsum dolor sit amet, consectetur adipiscing elit. Integer ut ", 
            str::from_utf8(res.as_slice()).unwrap()
        );
        assert_eq!(3, b.get_num_current_pages());
    }

    #[test]
    fn empty_pages_are_not_cached() {
        let mut b = FileSyncedBuffer::new(file_r("10.txt"), 4, 16, 1);
        assert_eq!(0, b.get_num_current_pages());
        let res = b.read(0, 128).unwrap();
        assert_eq!(3, b.get_num_current_pages());
    }

    // truncate() tests
    #[test]
    fn truncate_to_0_removes_all_pages() {
        let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16, 1);
        assert_eq!(0, b.get_num_current_pages());
        b.read(4, 64).unwrap();
        assert_eq!(5, b.get_num_current_pages());
        assert_eq!(vec!(0, 1, 2, 3, 4), b.get_current_page_insertions());
        b.truncate(0);
        assert_eq!(0, b.get_num_current_pages());
        assert_eq!(Vec::<u64>::new(), b.get_current_page_insertions());
    }

    #[test]
    fn truncate_removes_pages_past_len() {
        let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16, 1);
        assert_eq!(0, b.get_num_current_pages());
        b.read(4, 64).unwrap();
        assert_eq!(5, b.get_num_current_pages());
        assert_eq!(vec!(0, 1, 2, 3, 4), b.get_current_page_insertions());
        b.truncate(45);
        assert_eq!(3, b.get_num_current_pages());
        assert_eq!(vec!(0, 1, 2), b.get_current_page_insertions());
    }

    #[test]
    fn truncate_truncates_page_at_len() {
        let mut b = FileSyncedBuffer::new(file_r("100.txt"), 16, 16, 1);
        assert_eq!(0, b.get_num_current_pages());
        let res1 = b.read(32, 16).unwrap();
        assert_eq!(16, res1.len());
        assert_eq!("ectetur adipisci", str::from_utf8(res1.as_slice()).unwrap());
        b.truncate(45);
        let res2 = b.read(32, 16).unwrap();
        assert_eq!(13, res2.len());
        assert_eq!("ectetur adipi", str::from_utf8(res2.as_slice()).unwrap());
    }
    

}
