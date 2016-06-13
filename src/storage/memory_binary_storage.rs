extern crate alloc;
extern crate core;

use std::vec::Vec;
use std::str;
use alloc::heap;
use std::{mem, ptr, slice};
use storage::util;
use error::{ Error, MemoryError, AssertionError };
use storage::binary_storage;
use storage::binary_storage::BinaryStorage;



#[derive(Debug)]
pub struct MemoryBinaryStorage {
    origin: *const u8,
    is_open: bool,
    capacity: usize,
    expand_size: usize,
    use_txn_boundary: bool,
    txn_boundary: usize,
    align: usize,
    max_page_size: usize
}
impl MemoryBinaryStorage {

    pub fn new(
        initial_capacity: usize, 
        expand_size: usize, 
        use_txn_boundary: bool,
        align: usize,
        max_page_size: usize
    ) -> Result<MemoryBinaryStorage, Error> {

        try!(MemoryBinaryStorage::check_mem_params(
            align,
            expand_size,
            initial_capacity,
            max_page_size
        )); 

        let origin = unsafe { heap::allocate(initial_capacity, align) as *mut u8 };

        if origin.is_null() { 
            return Err(Error::Memory(MemoryError::new(binary_storage::ERR_MEM_ALLOC)));
        }

        unsafe { ptr::write_bytes::<u8>(origin, 0x0, initial_capacity) };

        Ok(MemoryBinaryStorage {
            origin: origin as *const u8,
            is_open: false,
            capacity: initial_capacity,
            expand_size: expand_size,
            use_txn_boundary: use_txn_boundary,
            txn_boundary: 0,
            align: align,
            max_page_size: max_page_size
        })

    }

    pub fn get_align(&self) -> usize {
        self.align
    }

    pub fn set_align(&mut self, align: usize) -> Result<(), Error> {
        try!(MemoryBinaryStorage::check_mem_params(
            align,
            self.expand_size,
            self.capacity,
            self.max_page_size
        ));

        self.align = align;
        Ok(())
    }

    pub fn get_max_page_size(&self) -> usize {
        self.max_page_size
    }

    fn ptr<T>(&self, offset: usize) -> *const T {
        (self.origin as usize + offset) as *const T
    }

    fn ptr_mut<T>(&mut self, offset: usize) -> *mut T {
        (self.origin as usize + offset) as *mut T
    }

    fn write<T>(&mut self, offset: usize, data: T) -> Result<(), Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));
        try!(AssertionError::assert_not(
            self.use_txn_boundary && offset < self.txn_boundary,
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY
        ));
        try!(self.expand(offset + mem::size_of::<T>()));

        unsafe { ptr::write(self.ptr_mut(offset), data) }
        Ok(())
    }

    fn read<T: Copy>(&self, offset: usize) -> Result<T, Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));
        try!(AssertionError::assert_not(
            self.use_txn_boundary && (offset + mem::size_of::<T>()) > self.txn_boundary,
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY
        ));
        try!(AssertionError::assert(
            offset + mem::size_of::<T>() <= self.capacity, 
            binary_storage::ERR_READ_PAST_END
        ));

        unsafe { Ok(ptr::read(self.ptr(offset))) }
    }

    fn check_mem_params(
        align: usize,
        expand_size: usize,
        initial_capacity: usize,
        max_page_size: usize
    ) -> Result<(), MemoryError> {
        // Expansion size must be greater than zero
        if expand_size < 1 { 
            return Err(MemoryError::new(binary_storage::ERR_EXPAND_SIZE_TOO_SMALL)); 
        }
        // Initial capacity must be greater than zero
        if initial_capacity < 1 { 
            return Err(MemoryError::new(binary_storage::ERR_INITIAL_CAP_TOO_SMALL)); 
        }
        // Alignment must be greater than zero
        if align < 1 { 
            return Err(MemoryError::new(binary_storage::ERR_ALIGN_TOO_SMALL)); 
        }
        // Max page size must be a power of 2 
        if !util::is_power_of_two(max_page_size) { 
            return Err(MemoryError::new(binary_storage::ERR_MAX_PAGE_SIZE_NOT_POWER_OF_2)); 
        }
        // Alignment must be a power of 2
        if !util::is_power_of_two(align) { 
            return Err(MemoryError::new(binary_storage::ERR_ALIGN_NOT_POWER_OF_2)); 
        }
        // Initial capacity must be a power of 2
        if !util::is_power_of_two(initial_capacity) { 
            return Err(MemoryError::new(binary_storage::ERR_INITIAL_CAP_NOT_POWER_OF_2)); 
        }
        // Expansion size must be a power of 2
        if !util::is_power_of_two(expand_size) { 
            return Err(MemoryError::new(binary_storage::ERR_EXPAND_SIZE_NOT_POWER_OF_2)); 
        }
        // Alignment must be no larger than max page size
        if align > max_page_size { 
            return Err(MemoryError::new(binary_storage::ERR_ALIGN_LARGER_THAN_PAGE_SIZE)); 
        }
        // If all checks pass, return true
        Ok(())
    }


}
impl BinaryStorage for MemoryBinaryStorage {

    fn open(&mut self) -> Result<(), Error> {
        try!(AssertionError::assert_not(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_OPEN));
        self.is_open = true;
        Ok(())
    }

    fn close(&mut self) -> Result<(), Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));
        self.is_open = false;
        Ok(())
    }

    fn w_i8(&mut self, offset: usize, data: i8) -> Result<(), Error> { self.write(offset, data) }
    fn w_i16(&mut self, offset: usize, data: i16) -> Result<(), Error> { self.write(offset, data) }
    fn w_i32(&mut self, offset: usize, data: i32) -> Result<(), Error> { self.write(offset, data) }
    fn w_i64(&mut self, offset: usize, data: i64) -> Result<(), Error> { self.write(offset, data) }

    fn w_u8(&mut self, offset: usize, data: u8) -> Result<(), Error> { self.write(offset, data) }
    fn w_u16(&mut self, offset: usize, data: u16) -> Result<(), Error> { self.write(offset, data) }
    fn w_u32(&mut self, offset: usize, data: u32) -> Result<(), Error> { self.write(offset, data) }
    fn w_u64(&mut self, offset: usize, data: u64) -> Result<(), Error> { self.write(offset, data) }

    fn w_f32(&mut self, offset: usize, data: f32) -> Result<(), Error> { self.write(offset, data) }
    fn w_f64(&mut self, offset: usize, data: f64) -> Result<(), Error> { self.write(offset, data) }

    fn w_bool(&mut self, offset: usize, data: bool) -> Result<(), Error> { self.write(offset, data) }

    fn w_bytes(&mut self, offset: usize, data: &[u8]) -> Result<(), Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));
        try!(AssertionError::assert_not(
            self.use_txn_boundary && offset < self.txn_boundary,
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY
        ));
        try!(self.expand(offset + data.len()));

        let dest = unsafe { slice::from_raw_parts_mut(self.ptr_mut(offset), data.len()) };
        dest.clone_from_slice(data);
        Ok(())
    }

    fn w_str(&mut self, offset: usize, data: &str) -> Result<(), Error> { 
        self.w_bytes(offset, data.as_bytes()) 
    }


    fn r_i8(&self, offset: usize) -> Result<i8, Error> { self.read(offset) }
    fn r_i16(&self, offset: usize) -> Result<i16, Error> { self.read(offset) }
    fn r_i32(&self, offset: usize) -> Result<i32, Error> { self.read(offset) }
    fn r_i64(&self, offset: usize) -> Result<i64, Error> { self.read(offset) }

    fn r_u8(&self, offset: usize) -> Result<u8, Error> { self.read(offset) }
    fn r_u16(&self, offset: usize) -> Result<u16, Error> { self.read(offset) }
    fn r_u32(&self, offset: usize) -> Result<u32, Error> { self.read(offset) }
    fn r_u64(&self, offset: usize) -> Result<u64, Error> { self.read(offset) }

    fn r_f32(&self, offset: usize) -> Result<f32, Error> { self.read(offset) }
    fn r_f64(&self, offset: usize) -> Result<f64, Error> { self.read(offset) }

    fn r_bool(&self, offset: usize) -> Result<bool, Error> { self.read(offset) }

    fn r_bytes(&self, offset: usize, len: usize) -> Result<Vec<u8>, Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));
        try!(AssertionError::assert_not(
            self.use_txn_boundary && (offset + len) > self.txn_boundary,
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY
        ));
        try!(AssertionError::assert(
            offset + len <= self.capacity, 
            binary_storage::ERR_READ_PAST_END
        ));

        let src = unsafe { slice::from_raw_parts::<u8>(self.ptr(offset), len) };
        let mut dst: Vec<u8> = Vec::with_capacity(len);
        unsafe { dst.set_len(len) };
        dst.copy_from_slice(src);
        Ok(dst)
    }

    fn r_str(&self, offset: usize, len: usize) -> Result<String, Error> {
        let b = try!(self.r_bytes(offset, len));
        Ok(try!(str::from_utf8(b.as_slice())).to_string())
    }


    fn fill(&mut self, start: Option<usize>, end: Option<usize>, val: u8) -> Result<(), Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));

        let start_offset = match start { Some(s) => s, None => 0 };

        try!(AssertionError::assert(
            start_offset < self.capacity, 
            binary_storage::ERR_WRITE_PAST_END
        ));

        try!(AssertionError::assert_not(
            self.use_txn_boundary && start_offset < self.txn_boundary,
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY
        ));

        let end_offset = match end { Some(e) => e, None => self.capacity };

        try!(AssertionError::assert(
            end_offset > start_offset,
            binary_storage::ERR_WRITE_NOTHING
        ));

        try!(AssertionError::assert(
            end_offset <= self.capacity,
            binary_storage::ERR_WRITE_PAST_END
        ));

        unsafe { ptr::write_bytes::<u8>(self.ptr_mut(start_offset), val, end_offset - start_offset) }
        Ok(())
    }

    fn is_filled(&self, start: Option<usize>, end: Option<usize>, val: u8) -> Result<bool, Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));

        let start_offset = match start {
            Some(s) => s,
            None => 0
        };

        try!(AssertionError::assert(
            start_offset < self.capacity, 
            binary_storage::ERR_READ_PAST_END
        ));

        let end_offset = match end {
            Some(e) => e,
            None => self.capacity
        };

        try!(AssertionError::assert(
            end_offset > start_offset,
            binary_storage::ERR_READ_NOTHING
        ));

        try!(AssertionError::assert(
            end_offset <= self.capacity,
            binary_storage::ERR_READ_PAST_END
        ));

        let data = unsafe {
            slice::from_raw_parts::<u8>(self.ptr(start_offset), end_offset - start_offset)
        };

        for b in data {
            if *b != val { return Ok(false) }
        }

        Ok(true)
    }


    fn get_use_txn_boundary(&self) -> bool {
        self.use_txn_boundary
    }

    fn set_use_txn_boundary(&mut self, val: bool) {
        self.use_txn_boundary = val;
        if !val { self.txn_boundary = 0 }
    }


    fn get_txn_boundary(&self) -> Result<usize, Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));
        try!(AssertionError::assert(
            self.use_txn_boundary, 
            binary_storage::ERR_OPERATION_INVALID_WHEN_NOT_USING_TXN_BOUNDARY
        ));
        Ok(self.txn_boundary)
    }

    fn set_txn_boundary(&mut self, offset: usize) -> Result<(), Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));
        try!(AssertionError::assert(
            self.use_txn_boundary, 
            binary_storage::ERR_OPERATION_INVALID_WHEN_NOT_USING_TXN_BOUNDARY
        ));
        try!(AssertionError::assert(
            offset <= self.capacity, 
            binary_storage::ERR_SET_TXN_BOUNDARY_PAST_END
        ));

        self.txn_boundary = offset;
        Ok(())
    }


    fn get_expand_size(&self) -> usize {
        self.expand_size
    }

    fn set_expand_size(&mut self, expand_size: usize) -> Result<(), Error> {
        try!(MemoryBinaryStorage::check_mem_params(
            self.align,
            expand_size,
            self.capacity,
            self.max_page_size
        ));

        self.expand_size = expand_size;
        Ok(())
    }


    fn expand(&mut self, min_capacity: usize) -> Result<(), Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));

        // Determine the new size of the journal in multiples of expand_size
        let expand_increments = (min_capacity as f64 / self.expand_size as f64).ceil() as usize;
        let new_capacity = match expand_increments.checked_mul(self.expand_size) {
            Some(x) => x,
            None => return Err(Error::Memory(
                MemoryError::new(binary_storage::ERR_ARITHMETIC_OVERFLOW_ON_EXPAND)
            ))
        };

        // We don't want to reallocate (or even reduce the capacity) if we already have enough,
        // so just do nothing and return Ok if we already have enough room
        if new_capacity <= self.capacity { return Ok(()) }

        // Allocate new memory
        let ptr = unsafe {
            heap::reallocate(
                self.origin as *mut u8,
                self.capacity,
                new_capacity,
                self.align
            )
        };

        if ptr.is_null() {
            return Err(Error::Memory(MemoryError::new(binary_storage::ERR_MEM_ALLOC)));
        } else {
            // Set the new capacity and pointer, remembering the old capacity
            let old_capacity = self.capacity;
            self.origin = ptr as *const u8;
            self.capacity = new_capacity;
            // Initialize the new storage (set all bytes to 0x00)
            self.fill(Some(old_capacity), Some(new_capacity), 0x0).unwrap();
            // Return Ok to indicate that allocation was successful
            Ok(())
        }
    }

    fn get_capacity(&self) -> Result<usize, Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));
        Ok(self.capacity)
    }

    fn is_open(&self) -> bool {
        self.is_open
    }


}


#[cfg(test)]
mod memory_binary_storage_tests {

    use std::str;
    use std::error::Error;

    use storage::binary_storage;
    use storage::binary_storage::tests;
    use storage::binary_storage::BinaryStorage;
    use storage::memory_binary_storage::MemoryBinaryStorage;

    // open(), close(), and is_open() tests 
    #[test]
    pub fn open_returns_err_when_already_open() {
        tests::open_returns_err_when_already_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    pub fn close_returns_err_when_already_closed() {
        tests::close_returns_err_when_already_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    pub fn open_returns_ok_when_previously_closed() {
        tests::open_returns_ok_when_previously_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    pub fn close_returns_ok_when_previously_open() {
        tests::close_returns_ok_when_previously_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn is_closed_when_new() {
        tests::is_closed_when_new(MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap());
    }

    #[test]
    fn is_open_after_open() {
        tests::is_open_after_open(MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap());
    }

    #[test]
    fn is_closed_after_open_and_close() {
        tests::is_closed_after_open_and_close(MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap());
    }

    // new() tests
    #[test]
    fn new_sets_initial_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap();
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
    }

    #[test]
    fn new_sets_expand_size() {
        let s = MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap();
        assert_eq!(512, s.get_expand_size());
    }

    #[test]
    fn new_sets_use_txn_boundary() {
        let s1 = MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap();
        assert!(!s1.get_use_txn_boundary());
        let s2 = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        assert!(s2.get_use_txn_boundary());
    }

    #[test]
    fn new_sets_align() {
        let s = MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap();
        assert_eq!(1024, s.get_align());
    }

    #[test]
    fn new_sets_max_page_size() {
        let s = MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap();
        assert_eq!(4096, s.get_max_page_size());
    }

    #[test]
    fn new_requires_initial_capacity_greater_than_0() {
        let s = MemoryBinaryStorage::new(0, 512, false, 1024, 4096);
        assert!(s.is_err());
        assert_eq!(binary_storage::ERR_INITIAL_CAP_TOO_SMALL, s.unwrap_err().description());
    }

    #[test]
    fn new_requires_expand_size_greater_than_0() {
        let s = MemoryBinaryStorage::new(256, 0, false, 1024, 4096);
        assert!(s.is_err());
        assert_eq!(binary_storage::ERR_EXPAND_SIZE_TOO_SMALL, s.unwrap_err().description());
    }

    #[test]
    fn new_requires_alignment_greater_than_0() {
        let s = MemoryBinaryStorage::new(256, 512, false, 0, 4096);
        assert!(s.is_err());
        assert_eq!(binary_storage::ERR_ALIGN_TOO_SMALL, s.unwrap_err().description());
    }

    #[test]
    fn new_requires_max_page_size_is_power_of_2() {
        let s1 = MemoryBinaryStorage::new(256, 512, false, 1024, 2048);
        assert!(s1.is_ok());

        let s2 = MemoryBinaryStorage::new(256, 512, false, 1024, 2049);
        assert!(s2.is_err());
        assert_eq!(binary_storage::ERR_MAX_PAGE_SIZE_NOT_POWER_OF_2, s2.unwrap_err().description());

        let s3 = MemoryBinaryStorage::new(256, 512, false, 1024, 3072);
        assert!(s3.is_err());
        assert_eq!(binary_storage::ERR_MAX_PAGE_SIZE_NOT_POWER_OF_2, s3.unwrap_err().description());

        let s4 = MemoryBinaryStorage::new(256, 512, false, 1024, 4096);
        assert!(s4.is_ok());
    }

    #[test]
    fn new_requires_alignment_is_power_of_2() {
        let s1 = MemoryBinaryStorage::new(256, 512, false, 1024, 4096);
        assert!(s1.is_ok());

        let s2 = MemoryBinaryStorage::new(256, 512, false, 1025, 4096);
        assert!(s2.is_err());
        assert_eq!(binary_storage::ERR_ALIGN_NOT_POWER_OF_2, s2.unwrap_err().description());

        let s3 = MemoryBinaryStorage::new(256, 512, false, 1536, 4096);
        assert!(s3.is_err());
        assert_eq!(binary_storage::ERR_ALIGN_NOT_POWER_OF_2, s3.unwrap_err().description());

        let s4 = MemoryBinaryStorage::new(256, 512, false, 2048, 4096);
        assert!(s4.is_ok());
    }

    #[test]
    fn new_requires_initial_capacity_is_power_of_2() {
        let s1 = MemoryBinaryStorage::new(256, 512, false, 1024, 4096);
        assert!(s1.is_ok());

        let s2 = MemoryBinaryStorage::new(257, 512, false, 1024, 4096);
        assert!(s2.is_err());
        assert_eq!(binary_storage::ERR_INITIAL_CAP_NOT_POWER_OF_2, s2.unwrap_err().description());

        let s3 = MemoryBinaryStorage::new(384, 512, false, 1024, 4096);
        assert!(s3.is_err());
        assert_eq!(binary_storage::ERR_INITIAL_CAP_NOT_POWER_OF_2, s3.unwrap_err().description());

        let s4 = MemoryBinaryStorage::new(512, 512, false, 1024, 4096);
        assert!(s4.is_ok());
    }

    #[test]
    fn new_requires_expand_size_is_power_of_2() {
        let s1 = MemoryBinaryStorage::new(256, 512, false, 1024, 4096);
        assert!(s1.is_ok());

        let s2 = MemoryBinaryStorage::new(256, 513, false, 1024, 4096);
        assert!(s2.is_err());
        assert_eq!(binary_storage::ERR_EXPAND_SIZE_NOT_POWER_OF_2, s2.unwrap_err().description());

        let s3 = MemoryBinaryStorage::new(256, 768, false, 1024, 4096);
        assert!(s3.is_err());
        assert_eq!(binary_storage::ERR_EXPAND_SIZE_NOT_POWER_OF_2, s3.unwrap_err().description());

        let s4 = MemoryBinaryStorage::new(256, 1024, false, 1024, 4096);
        assert!(s4.is_ok());
    }

    #[test]
    fn new_requires_alignment_no_larger_than_max_page_size() {
        let s1 = MemoryBinaryStorage::new(256, 512, false, 512, 1024);
        assert!(s1.is_ok());

        let s2 = MemoryBinaryStorage::new(256, 512, false, 1024, 1024);
        assert!(s2.is_ok());

        let s3 = MemoryBinaryStorage::new(256, 512, false, 2048, 1024);
        assert!(s3.is_err());
        assert_eq!(binary_storage::ERR_ALIGN_LARGER_THAN_PAGE_SIZE, s3.unwrap_err().description());
    }

    #[test]
    fn new_initializes_memory_to_zeros() {
        let mut s = MemoryBinaryStorage::new(256, 512, false, 512, 1024).unwrap();
        s.open().unwrap();
        assert!(s.is_filled(None, None, 0x0).unwrap());
    }

    // w_i8() tests
    #[test]
    fn w_i8_returns_err_when_closed() {
        tests::w_i8_returns_err_when_closed(MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap());
    }

    #[test]
    fn w_i8_returns_ok_when_open() {
        tests::w_i8_returns_ok_when_open(MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap());
    }

    #[test]
    fn w_i8_does_not_write_when_closed() {
        tests::w_i8_does_not_write_when_closed(MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap());
    }

    #[test]
    fn w_i8_does_not_write_before_txn_boundary() {
        tests::w_i8_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_i8_over_capacity_expands_storage() {
        tests::w_i8_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // w_i16() tests
    #[test]
    fn w_i16_returns_err_when_closed() {
        tests::w_i16_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_i16_returns_ok_when_open() {
        tests::w_i16_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_i16_does_not_write_when_closed() {
        tests::w_i16_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_i16_does_not_write_before_txn_boundary() {
        tests::w_i16_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_i16_over_capacity_expands_storage() {
        tests::w_i16_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // w_i32() tests
    #[test]
    fn w_i32_returns_err_when_closed() {
        tests::w_i32_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_i32_returns_ok_when_open() {
        tests::w_i32_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_i32_does_not_write_when_closed() {
        tests::w_i32_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_i32_does_not_write_before_txn_boundary() {
        tests::w_i32_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_i32_over_capacity_expands_storage() {
        tests::w_i32_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // w_i64() tests
    #[test]
    fn w_i64_returns_err_when_closed() {
        tests::w_i64_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_i64_returns_ok_when_open() {
        tests::w_i64_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_i64_does_not_write_when_closed() {
        tests::w_i64_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_i64_does_not_write_before_txn_boundary() {
        tests::w_i64_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_i64_over_capacity_expands_storage() {
        tests::w_i64_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // w_u8() tests
    #[test]
    fn w_u8_returns_err_when_closed() {
        tests::w_u8_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_u8_returns_ok_when_open() {
        tests::w_u8_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_u8_does_not_write_when_closed() {
        tests::w_u8_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_u8_does_not_write_before_txn_boundary() {
        tests::w_u8_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_u8_over_capacity_expands_storage() {
        tests::w_u8_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // w_u16() tests
    #[test]
    fn w_u16_returns_err_when_closed() {
        tests::w_u16_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_u16_returns_ok_when_open() {
        tests::w_u16_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_u16_does_not_write_when_closed() {
        tests::w_u16_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_u16_does_not_write_before_txn_boundary() {
        tests::w_u16_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_u16_over_capacity_expands_storage() {
        tests::w_u16_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // w_u32() tests
    #[test]
    fn w_u32_returns_err_when_closed() {
        tests::w_u32_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_u32_returns_ok_when_open() {
        tests::w_u32_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_u32_does_not_write_when_closed() {
        tests::w_u32_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_u32_does_not_write_before_txn_boundary() {
        tests::w_u32_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_u32_over_capacity_expands_storage() {
        tests::w_u32_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // w_u64() tests
    #[test]
    fn w_u64_returns_err_when_closed() {
        tests::w_u64_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_u64_returns_ok_when_open() {
        tests::w_u64_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_u64_does_not_write_when_closed() {
        tests::w_u64_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_u64_does_not_write_before_txn_boundary() {
        tests::w_u64_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_u64_over_capacity_expands_storage() {
        tests::w_u64_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // w_f32() tests
    #[test]
    fn w_f32_returns_err_when_closed() {
        tests::w_f32_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_f32_returns_ok_when_open() {
        tests::w_f32_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_f32_does_not_write_when_closed() {
        tests::w_f32_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_f32_does_not_write_before_txn_boundary() {
        tests::w_f32_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_f32_over_capacity_expands_storage() {
        tests::w_f32_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // w_f64() tests
    #[test]
    fn w_f64_returns_err_when_closed() {
        tests::w_f64_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_f64_returns_ok_when_open() {
        tests::w_f64_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_f64_does_not_write_when_closed() {
        tests::w_f64_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_f64_does_not_write_before_txn_boundary() {
        tests::w_f64_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_f64_over_capacity_expands_storage() {
        tests::w_f64_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // w_bool() tests
    #[test]
    fn w_bool_returns_err_when_closed() {
        tests::w_bool_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_bool_returns_ok_when_open() {
        tests::w_bool_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_bool_does_not_write_when_closed() {
        tests::w_bool_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_bool_does_not_write_before_txn_boundary() {
        tests::w_bool_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_bool_over_capacity_expands_storage() {
        tests::w_bool_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // w_bytes() tests
    #[test]
    fn w_bytes_returns_err_when_closed() {
        tests::w_bytes_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_bytes_returns_ok_when_open() {
        tests::w_bytes_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_bytes_does_not_write_when_closed() {
        tests::w_bytes_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_bytes_does_not_write_before_txn_boundary() {
        tests::w_bytes_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_bytes_over_capacity_expands_storage() {
        tests::w_bytes_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_bytes_over_capacity_expands_storage_multiple_times() {
        tests::w_bytes_over_capacity_expands_storage_multiple_times(
            MemoryBinaryStorage::new(256, 4, false, 256, 4096).unwrap()
        );
    }

    // w_str() tests
    #[test]
    fn w_str_returns_err_when_closed() {
        tests::w_str_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_str_returns_ok_when_open() {
        tests::w_str_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_str_does_not_write_when_closed() {
        tests::w_str_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_str_does_not_write_before_txn_boundary() {
        tests::w_str_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_str_over_capacity_expands_storage() {
        tests::w_str_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn w_str_over_capacity_expands_storage_multiple_times() {
        tests::w_str_over_capacity_expands_storage_multiple_times(
            MemoryBinaryStorage::new(256, 4, false, 256, 4096).unwrap()
        );
    }

    // r_i8() tests
    #[test]
    fn r_i8_returns_err_when_closed() {
        tests::r_i8_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i8_returns_ok_when_open() {
        tests::r_i8_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i8_reads_zero_from_unwritten_storage() {
        tests::r_i8_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i8_reads_written_data() {
        tests::r_i8_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i8_does_not_read_past_txn_boundary() {
        tests::r_i8_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i8_does_not_read_past_capacity() {
        tests::r_i8_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i8_result_is_not_mutated_on_subsequent_write() {
        tests::r_i8_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // r_i16() tests
    #[test]
    fn r_i16_returns_err_when_closed() {
        tests::r_i16_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i16_returns_ok_when_open() {
        tests::r_i16_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i16_reads_zero_from_unwritten_storage() {
        tests::r_i16_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i16_reads_written_data() {
        tests::r_i16_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i16_does_not_read_past_txn_boundary() {
        tests::r_i16_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i16_does_not_read_past_capacity() {
        tests::r_i16_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i16_result_is_not_mutated_on_subsequent_write() {
        tests::r_i16_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // r_i32() tests
    #[test]
    fn r_i32_returns_err_when_closed() {
        tests::r_i32_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i32_returns_ok_when_open() {
        tests::r_i32_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i32_reads_zero_from_unwritten_storage() {
        tests::r_i32_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i32_reads_written_data() {
        tests::r_i32_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i32_does_not_read_past_txn_boundary() {
        tests::r_i32_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i32_does_not_read_past_capacity() {
        tests::r_i32_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i32_result_is_not_mutated_on_subsequent_write() {
        tests::r_i32_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // r_i64() tests
    #[test]
    fn r_i64_returns_err_when_closed() {
        tests::r_i64_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i64_returns_ok_when_open() {
        tests::r_i64_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i64_reads_zero_from_unwritten_storage() {
        tests::r_i64_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i64_reads_written_data() {
        tests::r_i64_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i64_does_not_read_past_txn_boundary() {
        tests::r_i64_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i64_does_not_read_past_capacity() {
        tests::r_i64_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_i64_result_is_not_mutated_on_subsequent_write() {
        tests::r_i64_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // r_u8() tests
    #[test]
    fn r_u8_returns_err_when_closed() {
        tests::r_u8_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u8_returns_ok_when_open() {
        tests::r_u8_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u8_reads_zero_from_unwritten_storage() {
        tests::r_u8_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u8_reads_written_data() {
        tests::r_u8_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u8_does_not_read_past_txn_boundary() {
        tests::r_u8_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u8_does_not_read_past_capacity() {
        tests::r_u8_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u8_result_is_not_mutated_on_subsequent_write() {
        tests::r_u8_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // r_u16() tests
    #[test]
    fn r_u16_returns_err_when_closed() {
        tests::r_u16_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u16_returns_ok_when_open() {
        tests::r_u16_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u16_reads_zero_from_unwritten_storage() {
        tests::r_u16_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u16_reads_written_data() {
        tests::r_u16_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u16_does_not_read_past_txn_boundary() {
        tests::r_u16_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u16_does_not_read_past_capacity() {
        tests::r_u16_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u16_result_is_not_mutated_on_subsequent_write() {
        tests::r_u16_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // r_u32() tests
    #[test]
    fn r_u32_returns_err_when_closed() {
        tests::r_u32_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u32_returns_ok_when_open() {
        tests::r_u32_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u32_reads_zero_from_unwritten_storage() {
        tests::r_u32_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u32_reads_written_data() {
        tests::r_u32_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u32_does_not_read_past_txn_boundary() {
        tests::r_u32_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u32_does_not_read_past_capacity() {
        tests::r_u32_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u32_result_is_not_mutated_on_subsequent_write() {
        tests::r_u32_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // r_u64() tests
    #[test]
    fn r_u64_returns_err_when_closed() {
        tests::r_u64_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u64_returns_ok_when_open() {
        tests::r_u64_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u64_reads_zero_from_unwritten_storage() {
        tests::r_u64_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u64_reads_written_data() {
        tests::r_u64_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u64_does_not_read_past_txn_boundary() {
        tests::r_u64_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u64_does_not_read_past_capacity() {
        tests::r_u64_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_u64_result_is_not_mutated_on_subsequent_write() {
        tests::r_u64_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // r_f32() tests
    #[test]
    fn r_f32_returns_err_when_closed() {
        tests::r_f32_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_f32_returns_ok_when_open() {
        tests::r_f32_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_f32_reads_zero_from_unwritten_storage() {
        tests::r_f32_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_f32_reads_written_data() {
        tests::r_f32_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_f32_does_not_read_past_txn_boundary() {
        tests::r_f32_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_f32_does_not_read_past_capacity() {
        tests::r_f32_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_f32_result_is_not_mutated_on_subsequent_write() {
        tests::r_f32_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // r_f64() tests
    #[test]
    fn r_f64_returns_err_when_closed() {
        tests::r_f64_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_f64_returns_ok_when_open() {
        tests::r_f64_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_f64_reads_zero_from_unwritten_storage() {
        tests::r_f64_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_f64_reads_written_data() {
        tests::r_f64_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_f64_does_not_read_past_txn_boundary() {
        tests::r_f64_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_f64_does_not_read_past_capacity() {
        tests::r_f64_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_f64_result_is_not_mutated_on_subsequent_write() {
        tests::r_f64_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // r_bool() tests
    #[test]
    fn r_bool_returns_err_when_closed() {
        tests::r_bool_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_bool_returns_ok_when_open() {
        tests::r_bool_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_bool_reads_false_from_unwritten_storage() {
        tests::r_bool_reads_false_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_bool_reads_written_data() {
        tests::r_bool_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_bool_does_not_read_past_txn_boundary() {
        tests::r_bool_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_bool_does_not_read_past_capacity() {
        tests::r_bool_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_bool_result_is_not_mutated_on_subsequent_write() {
        tests::r_bool_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // r_bytes() tests
    #[test]
    fn r_bytes_returns_err_when_closed() {
        tests::r_bytes_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_bytes_returns_ok_when_open() {
        tests::r_bytes_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_bytes_reads_zeros_from_unwritten_storage() {
        tests::r_bytes_reads_zeros_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_bytes_reads_written_data() {
        tests::r_bytes_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_bytes_does_not_read_past_txn_boundary() {
        tests::r_bytes_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_bytes_does_not_read_past_capacity() {
        tests::r_bytes_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_bytes_result_is_not_mutated_on_subsequent_write() {
        tests::r_bytes_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // r_str() tests
    #[test]
    fn r_str_returns_err_when_closed() {
        tests::r_str_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_str_returns_ok_when_open() {
        tests::r_str_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_str_reads_nulls_from_unwritten_storage() {
        tests::r_str_reads_nulls_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_str_reads_written_data() {
        tests::r_str_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_str_does_not_read_past_txn_boundary() {
        tests::r_str_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_str_does_not_read_past_capacity() {
        tests::r_str_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn r_str_result_is_not_mutated_on_subsequent_write() {
        tests::r_str_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // fill() tests
    #[test]
    fn fill_returns_err_when_closed() {
        tests::fill_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn fill_does_not_write_when_closed() {
        tests::fill_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn fill_returns_ok_when_open() {
        tests::fill_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn fill_repeats_byte_in_storage_range() {
        tests::fill_repeats_byte_in_storage_range(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn fill_starts_from_beginning_when_start_offset_is_none() {
        tests::fill_starts_from_beginning_when_start_offset_is_none(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn fill_goes_to_end_when_end_offset_is_none() {
        tests::fill_goes_to_end_when_end_offset_is_none(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn fill_returns_err_when_end_offset_is_before_start_offset() {
        tests::fill_returns_err_when_end_offset_is_before_start_offset(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn fill_does_not_write_when_end_offset_is_before_start_offset() {
        tests::fill_does_not_write_when_end_offset_is_before_start_offset(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn fill_returns_err_when_before_txn_boundary() {
        tests::fill_returns_err_when_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn fill_does_not_write_when_before_txn_boundary() {
        tests::fill_does_not_write_when_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn fill_returns_ok_when_after_txn_boundary() {
        tests::fill_returns_ok_when_after_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn fill_writes_when_after_txn_boundary() {
        tests::fill_writes_when_after_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn fill_returns_err_when_past_capacity() {
        tests::fill_returns_err_when_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn fill_does_not_write_when_past_capacity() {
        tests::fill_does_not_write_when_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn fill_does_not_expand_capacity() {
        tests::fill_does_not_expand_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // assert_filled() tests
    #[test]
    fn is_filled_retuns_err_when_closed() {
        tests::is_filled_retuns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn is_filled_returns_err_when_start_offset_past_capacity() {
        tests::is_filled_returns_err_when_start_offset_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn is_filled_returns_err_when_end_offset_at_or_before_start_offset() {
        tests::is_filled_returns_err_when_end_offset_at_or_before_start_offset(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn is_filled_returns_err_when_end_offset_past_capacity() {
        tests::is_filled_returns_err_when_end_offset_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn is_filled_checks_whether_all_bytes_in_range_match_value() {
        tests::is_filled_checks_whether_all_bytes_in_range_match_value(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn is_filled_starts_from_start_offset() {
        tests::is_filled_starts_from_start_offset(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn is_filled_starts_from_beginning_when_start_offset_is_none() {
        tests::is_filled_starts_from_beginning_when_start_offset_is_none(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn is_filled_goes_to_end_offset() {
        tests::is_filled_goes_to_end_offset(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn is_filled_goes_to_end_when_end_offset_is_none() {
        tests::is_filled_goes_to_end_when_end_offset_is_none(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    // get_use_txn_boundary(), set_use_txn_boundary(), get_txn_boundary(), and set_txn_boundary() tests
    #[test]
    fn set_use_txn_boundary_changes_value() {
        tests::set_use_txn_boundary_changes_value(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn set_use_txn_boundary_resets_boundary_to_zero_when_txn_boundary_turned_off() {
        tests::set_use_txn_boundary_resets_boundary_to_zero_when_txn_boundary_turned_off(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn get_txn_boundary_returns_err_when_closed() {
        tests::get_txn_boundary_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn get_txn_boundary_returns_err_when_not_using_txn_boundary() {
        tests::get_txn_boundary_returns_err_when_not_using_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn get_txn_boundary_starts_at_0() {
        tests::get_txn_boundary_starts_at_0(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn set_txn_boundary_returns_err_when_not_using_txn_boundary() {
        tests::set_txn_boundary_returns_err_when_not_using_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn set_txn_boundary_does_not_change_boundary_when_not_using_txn_boundary() {
        tests::set_txn_boundary_does_not_change_boundary_when_not_using_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn set_txn_boundary_returns_err_when_closed() {
        tests::set_txn_boundary_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn set_txn_boundary_does_not_change_boundary_when_closed() {
        tests::set_txn_boundary_does_not_change_boundary_when_closed(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn set_txn_boundary_returns_err_when_past_capacity() {
        tests::set_txn_boundary_returns_err_when_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn set_txn_boundary_does_not_change_boundary_when_past_capacity() {
        tests::set_txn_boundary_does_not_change_boundary_when_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn set_txn_boundary_does_not_expand_capacity_when_past_capacity() {
        tests::set_txn_boundary_does_not_expand_capacity_when_past_capacity(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    #[test]
    fn set_txn_boundary_changes_boundary() {
        tests::set_txn_boundary_changes_boundary(
            MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap()
        );
    }

    /*
    // get_expand_size() and set_expand_size() tests
    #[test]
    fn get_expand_size_returns_initial_expand_size() {
        tests::get_expand_size_returns_initial_expand_size(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn set_expand_size_returns_err_when_expand_size_is_zero() {
        tests::set_expand_size_returns_err_when_expand_size_is_zero(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn set_expand_size_does_not_change_expand_size_when_expand_size_is_zero() {
        tests::set_expand_size_does_not_change_expand_size_when_expand_size_is_zero(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn set_expand_size_returns_err_when_expand_size_is_not_power_of_2() {
        tests::set_expand_size_returns_err_when_expand_size_is_not_power_of_2(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn set_expand_size_does_not_change_expand_size_when_expand_size_is_not_power_of_2() {
        tests::set_expand_size_does_not_change_expand_size_when_expand_size_is_not_power_of_2(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn set_expand_size_returns_true_when_checks_pass() {
        tests::set_expand_size_returns_true_when_checks_pass(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn set_expand_size_changes_expand_size_when_checks_pass() {
        tests::set_expand_size_changes_expand_size_when_checks_pass(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn capacity_increases_to_increments_of_last_set_expand_size() {
        tests::capacity_increases_to_increments_of_last_set_expand_size(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    // get_align() and set_align() tests
    #[test]
    fn get_align_returns_initial_align() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        assert_eq!(1024, s.get_align());
    }

    #[test]
    fn set_align_returns_err_when_align_is_zero() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        let res = s.set_align(0);
        assert!(res.is_err());
        assert_eq!(binary_storage::ERR_ALIGN_TOO_SMALL, res.unwrap_err().description());
    }

    #[test]
    fn set_align_does_not_change_align_when_align_is_zero() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.set_align(0);
        assert_eq!(1024, s.get_align());
    }

    #[test]
    fn set_align_returns_false_when_align_is_not_power_of_2() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        let res = s.set_align(1025);
        assert!(res.is_err());
        assert_eq!(binary_storage::ERR_ALIGN_NOT_POWER_OF_2, res.unwrap_err().description());
    }

    #[test]
    fn set_align_does_not_change_align_when_align_is_not_power_of_2() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.set_align(1025);
        assert_eq!(1024, s.get_align());
    }

    #[test]
    fn set_align_returns_true_when_checks_pass() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        assert!(s.set_align(2048).is_ok());
    }

    #[test]
    fn set_align_changes_align_when_checks_pass() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.set_align(2048);
        assert_eq!(2048, s.get_align());
    }

    // get_capacity() tests
    #[test]
    fn get_capacity_returns_0_when_closed() {
        tests::get_capacity_returns_0_when_closed(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn get_capacity_returns_initial_capacity_when_open() {
        tests::get_capacity_returns_initial_capacity_when_open(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn get_capacity_returns_new_capacity_after_expansion() {
        tests::get_capacity_returns_new_capacity_after_expansion(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    // get_max_page_size() tests
    #[test]
    fn get_max_page_size_returns_max_page_size() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        assert_eq!(4096, s.get_max_page_size());
    }

    // expand() tests
    #[test]
    fn expand_returns_false_when_closed() {
        tests::expand_returns_false_when_closed(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn expand_does_not_change_capacity_when_closed() {
        tests::expand_does_not_change_capacity_when_closed(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn expand_returns_true_when_already_has_capacity() {
        tests::expand_returns_true_when_already_has_capacity(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn expand_does_not_change_capacity_when_already_has_capacity() {
        tests::expand_does_not_change_capacity_when_already_has_capacity(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn expand_returns_false_when_allocation_arithmetic_overflows() {
        tests::expand_returns_false_when_allocation_arithmetic_overflows(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn expand_does_not_change_capacity_when_allocation_arithmetic_overflows() {
        tests::expand_does_not_change_capacity_when_allocation_arithmetic_overflows(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn expand_returns_false_when_allocation_fails() {
        tests::expand_returns_false_when_allocation_fails(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn expand_does_not_change_capacity_when_allocation_fails() {
        tests::expand_does_not_change_capacity_when_allocation_fails(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn expand_returns_true_when_successful() {
        tests::expand_returns_true_when_successful(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn expand_changes_capacity_by_expand_size_when_successful() {
        tests::expand_changes_capacity_by_expand_size_when_successful(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }

    #[test]
    fn expand_changes_capacity_by_multiples_of_expand_size_when_successful() {
        tests::expand_changes_capacity_by_multiples_of_expand_size_when_successful(
            MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap()
        );
    }
    */


}
