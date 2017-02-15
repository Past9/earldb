extern crate alloc;
extern crate core;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::iter::FromIterator;
use std::collections::{ HashMap, VecDeque };

use error::{ Error };
use storage::file_page::FilePage;


pub struct FileSyncedBuffer {
  file: File,
  page_size: u32,
  max_pages: u32,
  pages: HashMap<u64, FilePage>,
  page_insertions: VecDeque<u64>
}
impl FileSyncedBuffer {

  pub fn new(
    file: File,
    page_size: u32,
    max_pages: u32,
  ) -> FileSyncedBuffer {
    FileSyncedBuffer {
      file: file,
      page_size: page_size,
      max_pages: max_pages,
      pages: HashMap::new(),
      page_insertions: VecDeque::new()
    }
  }


  fn calc_page_range(&self, offset: u64, length: usize) -> (u64, u64) {
    let page_size = self.page_size as u64;
    let len = length as u64;
    let start = offset / page_size; 
    let end = (offset + len - 1) / page_size;
    (start as u64, end as u64)
  }

  fn calc_page_section(
    &self, 
    page_index: u64, 
    offset: u64, 
    length: usize
  ) -> (u32, u32) {
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

  fn read_from_page(
    &mut self, 
    index: u64, 
    start: u32, 
    len: u32
  ) -> Result<Vec<u8>, Error> {

    match self.pages.get(&index) {
      Some(p) => return Ok(p.read(start, len)),
      None => ()
    };

    let seek_pos = index * self.page_size as u64;

    try!(self.file.seek(SeekFrom::Start(seek_pos)));

    let mut buf = vec![0; self.page_size as usize];
    
    let read_len = try!(self.file.read(buf.as_mut_slice()));
    buf.truncate(read_len);

    let mut page = FilePage::new(self.page_size).unwrap();
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
      let (start_in_page, len_in_page) = self.calc_page_section(
        i, 
        offset, 
        data.len()
      );
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
    self.remove_oldest_pages(0);
  }

  pub fn get_num_current_pages(&self) -> u32 {
    self.pages.len() as u32
  }

  pub fn get_current_page_insertions(&self) -> Vec<u64> {
    Vec::from_iter(self.page_insertions.iter().map(|&x| x))
  }

}


