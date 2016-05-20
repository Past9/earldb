#![feature(alloc, heap_api)]

extern crate alloc;
extern crate core;

use alloc::heap;
use std::{mem, ptr, slice};
use storage::journal::Journal;


pub struct MemoryJournal {
    is_open: bool,
    is_complete: bool,
    start_size: usize,
    expand_size: usize,
    cap_bytes: usize,
    size_bytes: usize,
    start_ptr: *const u8,
    read_ptr: *mut u8,
    write_ptr: *mut u8

}
impl MemoryJournal {
    pub fn new(
        start_size: usize,
        expand_size: usize
    ) -> MemoryJournal {


        unsafe {
            let mut raw: *mut u8 = mem::transmute(heap::allocate(start_size, 1024));
            ptr::write_bytes(raw, 0, start_size);

            MemoryJournal {
                is_open: false,
                is_complete: true,
                start_size: start_size,
                expand_size: expand_size,
                cap_bytes: start_size,
                size_bytes: 0,
                start_ptr: raw as *const u8,
                read_ptr: raw,
                write_ptr: raw
            }

        }

    }

    fn as_slice(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self.start_ptr, self.cap_bytes)
        }
    }


    fn expand(&mut self, needed: usize) {
        let req_size = needed + self.size_bytes + 6; 
        println!("REQ: {}", req_size);
        if (req_size <= self.cap_bytes) { return; }

        let additional = req_size - self.cap_bytes;

        let expansion_size = (additional as f64 / self.expand_size as f64).ceil() as usize * self.expand_size;

        println!("{}, {}, {}", needed, additional, expansion_size);
        
        let new_size = self.cap_bytes + expansion_size;
        unsafe {
            let ptr = self.start_ptr as *mut u8;
            heap::reallocate(ptr, self.cap_bytes, new_size, 1024);
            self.cap_bytes = new_size;
        }
    }

}
impl Journal for MemoryJournal {


    fn open(&mut self) {
        self.is_open = true;
    }

    fn close(&mut self) {
        self.is_open = false;
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn reset(&mut self) {
        self.read_ptr = self.start_ptr as *mut u8;
    }

    fn write(&mut self, data: &[u8]) {
        if !self.is_open { return }
        if !self.is_complete { return }

        self.is_complete = false;

        self.expand(data.len());

        unsafe {
            ptr::write(self.write_ptr, 0x02);
            self.write_ptr = self.write_ptr.offset(1);
            let len = data.len() as u32;
            let len_src: *const u32 = mem::transmute(&len);
            ptr::copy(len_src, self.write_ptr as *mut u32, 1);
            self.write_ptr = self.write_ptr.offset(4);
            let dest_slice = slice::from_raw_parts_mut(self.write_ptr, len as usize);
            dest_slice.clone_from_slice(data);
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
            let len = self.cap_bytes - (self.write_ptr as usize - self.start_ptr as usize);
            ptr::write_bytes(self.write_ptr, 0, len);
        }

        self.is_complete = true;
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


#[cfg(test)]
mod tests {

    #![feature(alloc, heap_api)]

    extern crate alloc;
    extern crate core;

    use alloc::heap;
    use std::{mem, ptr, slice};
    use storage::journal::Journal;
    use storage::memory_journal::MemoryJournal;

    #[test]
    fn commits_and_reads_one_item() {
        let mut journal = MemoryJournal::new(1024, 1024);

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

    #[test]
    fn returns_correct_record_size() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data: [u8; 5] = [0, 1, 2, 3, 4];
        journal.open();
        journal.write(&data);
        journal.commit();
        assert_eq!(5, journal.size().unwrap());
    }

    #[test]
    fn is_complete_when_started() {
        let mut journal = MemoryJournal::new(1024, 1024);
        journal.open();
        assert!(journal.is_complete());
    }

    #[test]
    fn is_not_complete_after_write() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data: [u8; 5] = [0, 1, 2, 3, 4];
        journal.open();
        journal.write(&data);
        assert!(!journal.is_complete());
    }

    #[test]
    fn is_complete_after_commit() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data: [u8; 5] = [0, 1, 2, 3, 4];
        journal.open();
        journal.write(&data);
        journal.commit();
        assert!(journal.is_complete());
    }

    #[test]
    fn is_complete_after_discard() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data: [u8; 5] = [0, 1, 2, 3, 4];
        journal.open();
        journal.write(&data);
        journal.discard();
        assert!(journal.is_complete());
    }

    #[test]
    fn read_returns_none_when_reading_discarded_record() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data: [u8; 5] = [0, 1, 2, 3, 4];
        journal.open();
        journal.write(&data);
        journal.discard();
        assert_eq!(None, journal.read());
    }

    #[test]
    fn write_adds_record_in_place_of_discarded_record() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data1: [u8; 2] = [0, 1];
        journal.open();
        journal.write(&data1);
        journal.commit();

        let data2: [u8; 3] = [7, 8, 9];
        journal.write(&data2);
        journal.discard();

        let data3: [u8; 4] = [6, 5, 4, 3];
        journal.write(&data3);
        journal.commit();

        assert_eq!(2, journal.size().unwrap());
        journal.next();
        assert_eq!(4, journal.size().unwrap());
    }

    #[test]
    fn read_returns_none_when_reading_incomplete_record() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data: [u8; 5] = [0, 1, 2, 3, 4];
        journal.open();
        journal.write(&data);
        assert_eq!(None, journal.read());
    }

    #[test]
    fn commits_multiple_items_then_reads_from_beginning() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data1: [u8; 2] = [0, 1];
        journal.open();
        journal.write(&data1);
        journal.commit();

        let data2: [u8; 3] = [7, 8, 9];
        journal.write(&data2);
        journal.commit();

        let data3: [u8; 4] = [6, 5, 4, 3];
        journal.write(&data3);
        journal.commit();

        assert_eq!(2, journal.size().unwrap());
        journal.next();
        assert_eq!(3, journal.size().unwrap());
        journal.next();
        assert_eq!(4, journal.size().unwrap());
    }

    #[test]
    fn read_returns_none_when_no_more_elements() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data: [u8; 5] = [0, 1, 2, 3, 4];
        journal.open();
        journal.write(&data);
        journal.commit();
        assert!(journal.read().is_some());
        journal.next();
        assert_eq!(None, journal.read());
    }

    #[test]
    fn allocates_initial_capacity_of_start_capacity() {
        let mut journal = MemoryJournal::new(1024, 2048);
        assert_eq!(1024, journal.cap_bytes());
    }

    #[test]
    fn allocates_more_capacity_by_given_size_increment() {
        let mut journal = MemoryJournal::new(4096, 2048);
        assert_eq!(4096, journal.cap_bytes());
        let data: [u8; 1500] = unsafe { mem::uninitialized() };
        journal.open();
        journal.write(&data);
        journal.commit();
        journal.write(&data);
        journal.commit();
        journal.write(&data);
        journal.commit();
        assert_eq!(6144, journal.cap_bytes());
        journal.write(&data);
        journal.commit();
        assert_eq!(6144, journal.cap_bytes());
        journal.write(&data);
        journal.commit();
        assert_eq!(8192, journal.cap_bytes());
        journal.write(&data);
        journal.commit();
        assert_eq!(8192, journal.cap_bytes());
    }

    #[test]
    fn allocates_enough_capacity_for_record_when_record_larger_than_size_increment() {
        let mut journal = MemoryJournal::new(1024, 512);
        assert_eq!(1024, journal.cap_bytes());
        let data: [u8; 4000] = unsafe { mem::uninitialized() };
        journal.open();
        journal.write(&data);
        journal.commit();
        assert_eq!(4096, journal.cap_bytes());
    }

    #[test]
    fn reset_starts_reading_from_beginning() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data1: [u8; 2] = [0, 1];
        journal.open();
        journal.write(&data1);
        journal.commit();

        let data2: [u8; 3] = [7, 8, 9];
        journal.write(&data2);
        journal.commit();

        let data3: [u8; 4] = [6, 5, 4, 3];
        journal.write(&data3);
        journal.commit();

        journal.next();
        journal.next();
        journal.reset();

        assert_eq!(2, journal.size().unwrap());
        journal.next();
        assert_eq!(3, journal.size().unwrap());
        journal.next();
        assert_eq!(4, journal.size().unwrap());
    }

    #[test]
    fn read_without_next_reads_same_record() {
        let mut journal = MemoryJournal::new(1024, 1024);
        let data1: [u8; 2] = [0, 1];
        journal.open();
        journal.write(&data1);
        journal.commit();

        let data2: [u8; 3] = [7, 8, 9];
        journal.write(&data2);
        journal.commit();

        assert_eq!(2, journal.size().unwrap());
        assert_eq!(2, journal.size().unwrap());
        journal.next();
        assert_eq!(3, journal.size().unwrap());
    }

}
