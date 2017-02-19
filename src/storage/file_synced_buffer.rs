extern crate alloc;
extern crate core;

use std::cell::RefCell;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::iter::FromIterator;
use std::collections::{ HashMap, VecDeque };

use error::{ Error };
use storage::util;
use storage::file_page::FilePage;


pub struct FileSyncedBuffer {
  file: RefCell<File>,
  page_size: usize,
  max_pages: u64,
  pages: RefCell<HashMap<u64, FilePage>>,
  page_insertions: RefCell<VecDeque<u64>>
}
impl FileSyncedBuffer {

  pub fn new(
    file: File,
    page_size: usize,
    max_pages: u64,
  ) -> FileSyncedBuffer {
    FileSyncedBuffer {
      file: RefCell::new(file),
      page_size: page_size,
      max_pages: max_pages,
      pages: RefCell::new(HashMap::new()),
      page_insertions: RefCell::new(VecDeque::new())
    }
  }


  fn calc_page_range(&self, offset: u64, length: u64) -> (u64, u64) {
    let page_size = self.page_size as u64;
    let start = offset / page_size; 
    let end = (offset + length - 1) / page_size;
    (start, end)
  }

  fn calc_page_section(
    &self, 
    page_index: u64, 
    offset: u64, 
    length: usize
  ) -> Result<(usize, usize), Error> {
    let page_size = self.page_size as u64;
    let len = length as u64;

    let page_start_offset = page_index * page_size;
    let page_end_offset = page_start_offset + page_size;
    let end_offset = offset + len;

    let start_offset_in_page = try!(util::u64_as_usize(
      if page_start_offset < offset {
        offset - page_start_offset
      } else {
        0
      }
    ));

    let end_offset_in_page = try!(util::u64_as_usize(
      if end_offset > page_end_offset {
        page_size
      } else {
        end_offset - page_start_offset
      }
    ));

    Ok((start_offset_in_page, (end_offset_in_page - start_offset_in_page)))
  }

  fn remove_oldest_page(&self) {
    if self.page_insertions.borrow().len() == 0 { return }
    match self.page_insertions.borrow_mut().pop_front() {
      Some(i) => { 
        self.pages.borrow_mut().remove(&i); 
        return;
      },
      _ => ()
    };
    self.remove_oldest_page();
  }

  fn remove_oldest_pages(&self, room_for: u64) {
    let rf = room_for as usize;
    let max_pages = self.max_pages as usize;
    if self.pages.borrow().len() + rf <= max_pages { return }
    let num_to_rm = self.pages.borrow().len() - max_pages + rf;
    for _ in 0..(num_to_rm) { self.remove_oldest_page() }
  }

  fn remove_page(&mut self, index: u64) {
    self.pages.borrow_mut().remove(&index);

    let mut ins = self.page_insertions.borrow_mut();

    match ins.iter().position(|&x| x == index) {
      Some(i) => ins.remove(i),
      None => return
    };
  }

  fn insert_page(&self, index: u64, page: FilePage) {
    if self.max_pages == 0 { return }
    self.remove_oldest_pages(1);
    self.pages.borrow_mut().insert(index, page); 
    self.page_insertions.borrow_mut().push_back(index);
  }

  fn read_from_page(
    &self, 
    index: u64, 
    start: usize, 
    len: usize
  ) -> Result<Vec<u8>, Error> {

    match self.pages.borrow().get(&index) {
      Some(p) => return Ok(p.read(start, len)),
      None => ()
    };

    let seek_pos = index * self.page_size as u64;

    try!(self.file.borrow_mut().seek(SeekFrom::Start(seek_pos)));

    let mut buf = vec![0; self.page_size as usize];
    
    let read_len = try!(self.file.borrow_mut().read(buf.as_mut_slice()));
    buf.truncate(read_len);

    let mut page = FilePage::new(self.page_size).unwrap();
    page.write(0, buf.as_slice());
    let data = page.read(start, len);

    self.insert_page(index, page);

    Ok(data)
  }

  pub fn read(&self, offset: u64, len: usize) -> Result<Vec<u8>, Error> {
    let (start, end) = self.calc_page_range(offset, len as u64);

    let mut data = Vec::new();
    let mut total_len: usize = 0;

    for i in start..(end + 1) {
      let (start_in_page, len_in_page) = try!(self.calc_page_section(i, offset, len));
      let partial_data = try!(self.read_from_page(i, start_in_page, len_in_page));
      let partial_len = partial_data.len();
      total_len += partial_data.len();

      data.extend(partial_data);

      if partial_len < len_in_page as usize { break };
    }

    data.truncate(total_len);
    Ok(data)

  }

  pub fn update(&mut self, offset: u64, data: &[u8]) -> Result<(), Error> {
    let (start, end) = self.calc_page_range(offset, data.len() as u64);

    let mut data_offset: usize = 0;

    for i in start..(end + 1) {
      let (start_in_page, len_in_page) = try!(self.calc_page_section(
        i, 
        offset, 
        data.len()
      ));
      if i == start { 
        data_offset = try!(util::u64_as_usize(
          (start * self.page_size as u64) + start_in_page as u64
        ));
      }
      match self.pages.borrow_mut().get_mut(&i) {
        Some(p) => {
          let start_in_data = try!(util::u64_as_usize(
            i * self.page_size as u64 + start_in_page as u64 - data_offset as u64
          ));
          let end_in_data = start_in_data + len_in_page;
          p.write(
            start_in_page,
            &data[start_in_data..end_in_data]
          );
        },
        None => continue 
      };
    }
    Ok(())
  }

  pub fn truncate(&mut self, len: usize) {
    if len == 0 { 
      self.pages.borrow_mut().clear();
      self.page_insertions.borrow_mut().clear();
      return;
    }

    let page_size = self.page_size;

    let last_page = len / page_size;         
    let last_page_len = len % page_size;
    let to_remove = Vec::from_iter(
      self.page_insertions.borrow().iter()
        .filter(|&&p| p > last_page as u64)
        .map(|&p| p.clone())
    );


    for p in to_remove {
      self.remove_page(p);
    }

    match self.pages.borrow_mut().get_mut(&(last_page as u64)) {
      Some(p) => {
        p.truncate(last_page_len);
      },
      None => ()
    }

  }

  pub fn get_page_size(&self) -> usize {
    self.page_size
  }

  pub fn get_max_pages(&self) -> u64 {
    self.max_pages
  }

  pub fn set_max_pages(&mut self, pages: u64) {
    self.max_pages = pages;
    self.remove_oldest_pages(0);
  }

  pub fn get_num_current_pages(&self) -> u64 {
    self.pages.borrow().len() as u64
  }

  pub fn get_current_page_insertions(&self) -> Vec<u64> {
    Vec::from_iter(self.page_insertions.borrow().iter().map(|&x| x))
  }

}


