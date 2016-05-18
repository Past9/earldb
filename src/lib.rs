#![feature(alloc, heap_api)]

extern crate alloc;
extern crate core;

use alloc::heap;
use std::{mem, ptr, slice};


pub trait JournalInterface {

    fn open(&mut self);
    fn close(&mut self);
    fn is_open(&self) -> bool;

    fn write(&mut self, data: &[u8]);
    fn commit(&mut self);
    fn discard(&mut self);

    fn next(&mut self);
    fn size(&self) -> Option<u32>;
    fn read(&self) -> Option<Vec<u8>>;
    fn is_complete(&self) -> bool;

    fn cap_bytes(&self) -> usize;

}

pub struct MemoryJournal {
    is_open: bool,
    is_complete: bool,
    start_size: usize,
    expand_size: usize,
    cap_bytes: usize,
    read_ptr: *mut u8,
    write_ptr: *mut u8

}
impl MemoryJournal {
    pub fn new() -> MemoryJournal {

        let init_size = 32 * 1024;

        unsafe {
            let mut raw: *mut u8 = mem::transmute(heap::allocate(init_size, 4 * 1024));
            ptr::write_bytes(raw, 0, init_size);

            MemoryJournal {
                is_open: false,
                is_complete: true,
                start_size: init_size,
                expand_size: init_size,
                cap_bytes: init_size,
                read_ptr: raw,
                write_ptr: raw
            }

        }

    }


    fn expand() {
    }

}
impl JournalInterface for MemoryJournal {


    fn open(&mut self) {
        self.is_open = true;
    }

    fn close(&mut self) {
        self.is_open = false;
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn write(&mut self, data: &[u8]) {
        if !self.is_open { return }
        if !self.is_complete { return }

        self.is_complete = false;

        unsafe {
            ptr::write(self.write_ptr, 0x02);
            self.write_ptr = self.write_ptr.offset(1);
            let len = data.len() as u32;
            let len_src: *const u32 = mem::transmute(&len);
            ptr::copy(len_src, self.write_ptr as *mut u32, 1);
            self.write_ptr = self.write_ptr.offset(4);

            let dest_slice = slice::from_raw_parts_mut(self.write_ptr, len as usize);

            dest_slice.clone_from_slice(data);



            /*
            //let data_ref = data.as_ref();
            let data_pos: usize = data as *const usize as usize;
            let data_ptr: *const u8 = mem::transmute(data);
            ptr::copy(data_ptr, self.write_ptr, len as usize);
            */
            self.write_ptr = self.write_ptr.offset(len as isize);
        }

    }

    fn commit(&mut self) {
        if !self.is_open { return }
        if self.is_complete { return }

        unsafe {
            ptr::write(self.write_ptr, 0x03);
            self.write_ptr = self.write_ptr.offset(1);
        }

        self.is_complete = true;
    }

    fn discard(&mut self) {
        if !self.is_open { return }

        unsafe {
            self.write_ptr = self.read_ptr;
            let len = self.cap_bytes - (self.write_ptr as usize);
            ptr::write_bytes(self.write_ptr, 0, len);
        }
    }

    fn next(&mut self) {
        if !self.is_open { return }
        if !self.is_complete { return }

        unsafe {
            self.read_ptr = self.read_ptr.offset(6 + self.size().unwrap() as isize);
        }
    }

    fn size(&self) -> Option<u32> {
        if !self.is_open { return None }
        if !self.is_complete { return None }

        unsafe {
            let size_ptr = self.read_ptr.offset(1) as *const u32;
            Some(*size_ptr)
        }
    }

    fn read(&self) -> Option<Vec<u8>> {
        if !self.is_open { return None }
        if !self.is_complete { return None }

        let size = self.size().unwrap() as usize;
        let mut dst: Vec<u8> = Vec::with_capacity(size);

        unsafe {

            let mut disp: Vec<u8> = Vec::with_capacity(size + 6);
            disp.set_len(size + 6);
            ptr::copy(self.read_ptr, disp.as_mut_ptr(), size + 6);

            dst.set_len(size);
            ptr::copy(self.read_ptr.offset(5), dst.as_mut_ptr(), size);

            return Some(dst);
        }
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    fn cap_bytes(&self) -> usize {
        self.cap_bytes
    }

}







#[test]
fn it_works() {
    let mut journal = MemoryJournal::new();

    let data: [u8; 3] = [1, 2, 3];
    journal.open();
    journal.write(&data);
    journal.commit();

    let r = journal.read().unwrap();

    assert_eq!(3, r.len());
    assert_eq!(1, r[0]);
    assert_eq!(2, r[1]);
    assert_eq!(3, r[2]);

}

