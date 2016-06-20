use std::vec::Vec;
use std::str;
use alloc::heap;
use std::{mem, ptr, slice};
use storage::util;
use error::{ Error, MemoryError, AssertionError };
use storage::binary_storage;
use storage::binary_storage::BinaryStorage;


mod safe_calc {

    use error::{ Error, AssertionError };
    use storage::binary_storage;

    pub fn u64_as_usize(n: u64) -> Result<usize, AssertionError> {
        try!(AssertionError::assert_not(
            n > usize::max_value() as u64, 
            binary_storage::ERR_ARITHMETIC_OVERFLOW
        ));
        Ok(n as usize)
    }

    pub fn usize_add(a: usize, b: usize) -> Result<usize, AssertionError> {
        match a.checked_add(b) {
            Some(n) => Ok(n),
            None => Err(AssertionError::new(binary_storage::ERR_ARITHMETIC_OVERFLOW))
        }
    }

}


#[derive(Debug)]
pub struct MemoryBinaryStorage {
    origin: *const u8,
    is_open: bool,
    capacity: u64,
    expand_size: u64,
    use_txn_boundary: bool,
    txn_boundary: u64,
    align: usize
}
impl MemoryBinaryStorage {

    pub fn new(
        initial_capacity: u64, 
        expand_size: u64, 
        use_txn_boundary: bool
    ) -> Result<MemoryBinaryStorage, Error> {

        try!(MemoryBinaryStorage::check_params(
            expand_size,
            initial_capacity
        )); 

        let align = mem::size_of::<usize>();

        let c_capacity = try!(safe_calc::u64_as_usize(initial_capacity));

        let origin = unsafe { heap::allocate(c_capacity, align) };

        if origin.is_null() { 
            return Err(Error::Memory(MemoryError::new(binary_storage::ERR_STORAGE_ALLOC)));
        }

        unsafe { ptr::write_bytes::<u8>(origin, 0x0, c_capacity) };

        Ok(MemoryBinaryStorage {
            origin: origin as *const u8,
            is_open: false,
            capacity: initial_capacity,
            expand_size: expand_size,
            use_txn_boundary: use_txn_boundary,
            txn_boundary: 0,
            align: align
        })

    }

    fn ptr<T>(&self, offset: usize) -> *const T {
        (self.origin as usize * offset) as *const T
    }

    fn ptr_mut<T>(&mut self, offset: usize) -> *mut T {
        (self.origin as usize * offset) as *mut T
    }

    fn write<T>(&mut self, offset: u64, data: T) -> Result<(), Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));

        let c_capacity = try!(safe_calc::u64_as_usize(self.capacity));
        let c_offset = try!(safe_calc::u64_as_usize(offset));
        let end_offset = try!(safe_calc::usize_add(c_offset, mem::size_of::<T>()));
        try!(safe_calc::usize_add(self.origin as usize, end_offset));

        try!(AssertionError::assert_not(end_offset > c_capacity, binary_storage::ERR_WRITE_PAST_END));

        if self.use_txn_boundary {
            let c_boundary = try!(safe_calc::u64_as_usize(self.txn_boundary));
            try!(AssertionError::assert_not(
                c_offset < c_boundary, 
                binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY
            ));
        }
        
        try!(self.expand(end_offset as u64));
        unsafe { ptr::write(self.ptr_mut(c_offset), data) }
        Ok(())
    }

    fn read<T: Copy>(&self, offset: u64) -> Result<T, Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));

        let c_capacity = try!(safe_calc::u64_as_usize(self.capacity));
        let c_offset = try!(safe_calc::u64_as_usize(offset));
        let end_offset = try!(safe_calc::usize_add(c_offset, mem::size_of::<T>()));
        try!(safe_calc::usize_add(self.origin as usize, end_offset));

        try!(AssertionError::assert_not(end_offset > c_capacity, binary_storage::ERR_READ_PAST_END));

        if self.use_txn_boundary {
            let c_boundary = try!(safe_calc::u64_as_usize(self.txn_boundary));
            try!(AssertionError::assert_not(
                end_offset > c_boundary, 
                binary_storage::ERR_READ_AFTER_TXN_BOUNDARY
            ));
        }

        unsafe { Ok(ptr::read(self.ptr(c_offset))) }
    }

    fn check_params(
        expand_size: u64,
        initial_capacity: u64,
    ) -> Result<(), AssertionError> {
        // Expansion size must be greater than zero
        try!(AssertionError::assert(
            expand_size > 0, 
            binary_storage::ERR_EXPAND_SIZE_TOO_SMALL
        ));
        // Initial capacity must be greater than zero
        try!(AssertionError::assert(
            initial_capacity > 0, 
            binary_storage::ERR_INITIAL_CAP_TOO_SMALL
        ));
        // Initial capacity must be a power of 2
        try!(AssertionError::assert(
            initial_capacity.is_power_of_two(), 
            binary_storage::ERR_INITIAL_CAP_NOT_POWER_OF_2
        ));
        // Expansion size must be a power of 2
        try!(AssertionError::assert(
            expand_size.is_power_of_two(), 
            binary_storage::ERR_EXPAND_SIZE_NOT_POWER_OF_2
        ));
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

    fn w_i8(&mut self, offset: u64, data: i8) -> Result<(), Error> { self.write(offset, data) }
    fn w_i16(&mut self, offset: u64, data: i16) -> Result<(), Error> { self.write(offset, data) }
    fn w_i32(&mut self, offset: u64, data: i32) -> Result<(), Error> { self.write(offset, data) }
    fn w_i64(&mut self, offset: u64, data: i64) -> Result<(), Error> { self.write(offset, data) }

    fn w_u8(&mut self, offset: u64, data: u8) -> Result<(), Error> { self.write(offset, data) }
    fn w_u16(&mut self, offset: u64, data: u16) -> Result<(), Error> { self.write(offset, data) }
    fn w_u32(&mut self, offset: u64, data: u32) -> Result<(), Error> { self.write(offset, data) }
    fn w_u64(&mut self, offset: u64, data: u64) -> Result<(), Error> { self.write(offset, data) }

    fn w_f32(&mut self, offset: u64, data: f32) -> Result<(), Error> { self.write(offset, data) }
    fn w_f64(&mut self, offset: u64, data: f64) -> Result<(), Error> { self.write(offset, data) }

    fn w_bool(&mut self, offset: u64, data: bool) -> Result<(), Error> { self.write(offset, data) }

    fn w_bytes(&mut self, offset: u64, data: &[u8]) -> Result<(), Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));

        let c_capacity = try!(safe_calc::u64_as_usize(self.capacity));
        let c_offset = try!(safe_calc::u64_as_usize(offset));
        let end_offset = try!(safe_calc::usize_add(c_offset, data.len()));
        try!(safe_calc::usize_add(self.origin as usize, end_offset));

        try!(AssertionError::assert_not(end_offset > c_capacity, binary_storage::ERR_WRITE_PAST_END));

        if self.use_txn_boundary {
            let c_boundary = try!(safe_calc::u64_as_usize(self.txn_boundary));
            try!(AssertionError::assert_not(
                c_offset < c_boundary, 
                binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY
            ));
        }

        try!(self.expand(end_offset as u64));

        let dest = unsafe { slice::from_raw_parts_mut(self.ptr_mut(c_offset), data.len()) };
        dest.clone_from_slice(data);
        Ok(())
    }

    fn w_str(&mut self, offset: u64, data: &str) -> Result<(), Error> { 
        self.w_bytes(offset, data.as_bytes()) 
    }


    fn r_i8(&mut self, offset: u64) -> Result<i8, Error> { self.read(offset) }
    fn r_i16(&mut self, offset: u64) -> Result<i16, Error> { self.read(offset) }
    fn r_i32(&mut self, offset: u64) -> Result<i32, Error> { self.read(offset) }
    fn r_i64(&mut self, offset: u64) -> Result<i64, Error> { self.read(offset) }

    fn r_u8(&mut self, offset: u64) -> Result<u8, Error> { self.read(offset) }
    fn r_u16(&mut self, offset: u64) -> Result<u16, Error> { self.read(offset) }
    fn r_u32(&mut self, offset: u64) -> Result<u32, Error> { self.read(offset) }
    fn r_u64(&mut self, offset: u64) -> Result<u64, Error> { self.read(offset) }

    fn r_f32(&mut self, offset: u64) -> Result<f32, Error> { self.read(offset) }
    fn r_f64(&mut self, offset: u64) -> Result<f64, Error> { self.read(offset) }

    fn r_bool(&mut self, offset: u64) -> Result<bool, Error> { self.read(offset) }

    fn r_bytes(&mut self, offset: u64, len: usize) -> Result<Vec<u8>, Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));

        let c_capacity = try!(safe_calc::u64_as_usize(self.capacity));
        let c_offset = try!(safe_calc::u64_as_usize(offset));
        let end_offset = try!(safe_calc::usize_add(c_offset, len));
        try!(safe_calc::usize_add(self.origin as usize, end_offset));

        try!(AssertionError::assert_not(end_offset > c_capacity, binary_storage::ERR_READ_PAST_END));

        if self.use_txn_boundary {
            let c_boundary = try!(safe_calc::u64_as_usize(self.txn_boundary));
            try!(AssertionError::assert_not(
                end_offset > c_boundary, 
                binary_storage::ERR_READ_AFTER_TXN_BOUNDARY
            ));
        }

        let src = unsafe { slice::from_raw_parts::<u8>(self.ptr(c_offset), len) };
        let mut dst = vec![0; len];
        dst.copy_from_slice(src);
        Ok(dst)
    }

    fn r_str(&mut self, offset: u64, len: usize) -> Result<String, Error> {
        let b = try!(self.r_bytes(offset, len));
        Ok(try!(str::from_utf8(b.as_slice())).to_string())
    }


    fn fill(&mut self, start: Option<u64>, end: Option<u64>, val: u8) -> Result<(), Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));

        let start_offset = try!(safe_calc::u64_as_usize(
            match start { Some(s) => s, None => 0 }
        ));
        let end_offset = try!(safe_calc::u64_as_usize(
            match end { Some(e) => e, None => self.capacity }
        ));

        let c_capacity = try!(safe_calc::u64_as_usize(self.capacity));

        try!(AssertionError::assert(
            start_offset < c_capacity, 
            binary_storage::ERR_WRITE_PAST_END
        ));

        try!(AssertionError::assert(
            end_offset < c_capacity, 
            binary_storage::ERR_WRITE_PAST_END
        ));

        try!(AssertionError::assert(
            end_offset > start_offset,
            binary_storage::ERR_WRITE_NOTHING
        ));

        if self.use_txn_boundary {
            let c_boundary = try!(safe_calc::u64_as_usize(self.txn_boundary));
            try!(AssertionError::assert_not(
                start_offset < c_boundary, 
                binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY
            ));
        }

        unsafe { ptr::write_bytes::<u8>(self.ptr_mut(start_offset), val, end_offset - start_offset) }
        Ok(())
    }

    fn is_filled(&mut self, start: Option<u64>, end: Option<u64>, val: u8) -> Result<bool, Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));

        let start_offset = try!(safe_calc::u64_as_usize(
            match start { Some(s) => s, None => 0 }
        ));
        let end_offset = try!(safe_calc::u64_as_usize(
            match end { Some(e) => e, None => self.capacity }
        ));

        let c_capacity = try!(safe_calc::u64_as_usize(self.capacity));

        try!(AssertionError::assert(
            start_offset < c_capacity, 
            binary_storage::ERR_READ_PAST_END
        ));

        try!(AssertionError::assert(
            end_offset <= c_capacity,
            binary_storage::ERR_READ_PAST_END
        ));

        try!(AssertionError::assert(
            end_offset > start_offset,
            binary_storage::ERR_READ_NOTHING
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


    fn get_txn_boundary(&self) -> Result<u64, Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));
        try!(AssertionError::assert(
            self.use_txn_boundary, 
            binary_storage::ERR_OPERATION_INVALID_WHEN_NOT_USING_TXN_BOUNDARY
        ));
        Ok(self.txn_boundary)
    }

    fn set_txn_boundary(&mut self, offset: u64) -> Result<(), Error> {
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


    fn get_expand_size(&self) -> u64 {
        self.expand_size
    }

    fn set_expand_size(&mut self, expand_size: u64) -> Result<(), Error> {
        try!(MemoryBinaryStorage::check_params(
            expand_size,
            self.capacity
        ));

        self.expand_size = expand_size;
        Ok(())
    }


    fn expand(&mut self, min_capacity: u64) -> Result<(), Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));

        // Determine the new size of the journal in multiples of expand_size
        let expand_increments = (min_capacity as f64 / self.expand_size as f64).ceil() as u64;
        let new_capacity = match expand_increments.checked_mul(self.expand_size) {
            Some(x) => x,
            None => return Err(Error::Assertion(
                AssertionError::new(binary_storage::ERR_ARITHMETIC_OVERFLOW)
            ))
        };

        let c_capacity = try!(safe_calc::u64_as_usize(self.capacity));
        let c_new_capacity = try!(safe_calc::u64_as_usize(new_capacity));

        // We don't want to reallocate (or even reduce the capacity) if we already have enough,
        // so just do nothing and return Ok if we already have enough room
        if c_new_capacity <= c_capacity { return Ok(()) }

        // Allocate new memory
        let ptr = unsafe { 
            heap::reallocate(
                self.origin as *mut u8,
                c_capacity,
                c_new_capacity,
                self.align
            )
        };

        if ptr.is_null() {
            return Err(Error::Assertion(AssertionError::new(binary_storage::ERR_STORAGE_ALLOC)));
        } else {
            // Set the new capacity and pointer, remembering the old capacity
            let old_capacity = self.capacity;
            self.origin = ptr as *const u8;
            self.capacity = new_capacity;
            // Initialize the new storage (set all bytes to 0x00)
            try!(self.fill(Some(old_capacity), Some(new_capacity), 0x0));
            // Return Ok to indicate that allocation was successful
            Ok(())
        }
    }

    fn get_capacity(&self) -> Result<u64, Error> {
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
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    pub fn close_returns_err_when_already_closed() {
        tests::close_returns_err_when_already_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    pub fn open_returns_ok_when_previously_closed() {
        tests::open_returns_ok_when_previously_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    pub fn close_returns_ok_when_previously_open() {
        tests::close_returns_ok_when_previously_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn is_closed_when_new() {
        tests::is_closed_when_new(MemoryBinaryStorage::new(256, 256, false).unwrap());
    }

    #[test]
    fn is_open_after_open() {
        tests::is_open_after_open(MemoryBinaryStorage::new(256, 256, false).unwrap());
    }

    #[test]
    fn is_closed_after_open_and_close() {
        tests::is_closed_after_open_and_close(MemoryBinaryStorage::new(256, 256, false).unwrap());
    }

    // new() tests
    #[test]
    fn new_sets_initial_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 512, false).unwrap();
        s.open().unwrap();
        assert_eq!(256, s.get_capacity().unwrap());
    }

    #[test]
    fn new_sets_expand_size() {
        let s = MemoryBinaryStorage::new(256, 512, false).unwrap();
        assert_eq!(512, s.get_expand_size());
    }

    #[test]
    fn new_sets_use_txn_boundary() {
        let s1 = MemoryBinaryStorage::new(256, 512, false).unwrap();
        assert!(!s1.get_use_txn_boundary());
        let s2 = MemoryBinaryStorage::new(256, 512, true).unwrap();
        assert!(s2.get_use_txn_boundary());
    }

    #[test]
    fn new_requires_initial_capacity_greater_than_0() {
        let s = MemoryBinaryStorage::new(0, 512, false);
        assert!(s.is_err());
        assert_eq!(binary_storage::ERR_INITIAL_CAP_TOO_SMALL, s.unwrap_err().description());
    }

    #[test]
    fn new_requires_expand_size_greater_than_0() {
        let s = MemoryBinaryStorage::new(256, 0, false);
        assert!(s.is_err());
        assert_eq!(binary_storage::ERR_EXPAND_SIZE_TOO_SMALL, s.unwrap_err().description());
    }

    #[test]
    fn new_requires_initial_capacity_is_power_of_2() {
        let s1 = MemoryBinaryStorage::new(256, 512, false);
        assert!(s1.is_ok());

        let s2 = MemoryBinaryStorage::new(257, 512, false);
        assert!(s2.is_err());
        assert_eq!(binary_storage::ERR_INITIAL_CAP_NOT_POWER_OF_2, s2.unwrap_err().description());

        let s3 = MemoryBinaryStorage::new(384, 512, false);
        assert!(s3.is_err());
        assert_eq!(binary_storage::ERR_INITIAL_CAP_NOT_POWER_OF_2, s3.unwrap_err().description());

        let s4 = MemoryBinaryStorage::new(512, 512, false);
        assert!(s4.is_ok());
    }

    #[test]
    fn new_requires_expand_size_is_power_of_2() {
        let s1 = MemoryBinaryStorage::new(256, 512, false);
        assert!(s1.is_ok());

        let s2 = MemoryBinaryStorage::new(256, 513, false);
        assert!(s2.is_err());
        assert_eq!(binary_storage::ERR_EXPAND_SIZE_NOT_POWER_OF_2, s2.unwrap_err().description());

        let s3 = MemoryBinaryStorage::new(256, 768, false);
        assert!(s3.is_err());
        assert_eq!(binary_storage::ERR_EXPAND_SIZE_NOT_POWER_OF_2, s3.unwrap_err().description());

        let s4 = MemoryBinaryStorage::new(256, 1024, false);
        assert!(s4.is_ok());
    }

    #[test]
    fn new_initializes_memory_to_zeros() {
        let mut s = MemoryBinaryStorage::new(256, 512, false).unwrap();
        s.open().unwrap();
        assert!(s.is_filled(None, None, 0x0).unwrap());
    }

    // w_i8() tests
    #[test]
    fn w_i8_returns_err_when_closed() {
        tests::w_i8_returns_err_when_closed(MemoryBinaryStorage::new(256, 256, false).unwrap());
    }

    #[test]
    fn w_i8_returns_ok_when_open() {
        tests::w_i8_returns_ok_when_open(MemoryBinaryStorage::new(256, 256, false).unwrap());
    }

    #[test]
    fn w_i8_does_not_write_when_closed() {
        tests::w_i8_does_not_write_when_closed(MemoryBinaryStorage::new(256, 256, false).unwrap());
    }

    #[test]
    fn w_i8_does_not_write_before_txn_boundary() {
        tests::w_i8_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_i8_over_capacity_expands_storage() {
        tests::w_i8_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // w_i16() tests
    #[test]
    fn w_i16_returns_err_when_closed() {
        tests::w_i16_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_i16_returns_ok_when_open() {
        tests::w_i16_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_i16_does_not_write_when_closed() {
        tests::w_i16_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_i16_does_not_write_before_txn_boundary() {
        tests::w_i16_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_i16_over_capacity_expands_storage() {
        tests::w_i16_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // w_i32() tests
    #[test]
    fn w_i32_returns_err_when_closed() {
        tests::w_i32_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_i32_returns_ok_when_open() {
        tests::w_i32_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_i32_does_not_write_when_closed() {
        tests::w_i32_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_i32_does_not_write_before_txn_boundary() {
        tests::w_i32_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_i32_over_capacity_expands_storage() {
        tests::w_i32_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // w_i64() tests
    #[test]
    fn w_i64_returns_err_when_closed() {
        tests::w_i64_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_i64_returns_ok_when_open() {
        tests::w_i64_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_i64_does_not_write_when_closed() {
        tests::w_i64_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_i64_does_not_write_before_txn_boundary() {
        tests::w_i64_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_i64_over_capacity_expands_storage() {
        tests::w_i64_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // w_u8() tests
    #[test]
    fn w_u8_returns_err_when_closed() {
        tests::w_u8_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_u8_returns_ok_when_open() {
        tests::w_u8_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_u8_does_not_write_when_closed() {
        tests::w_u8_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_u8_does_not_write_before_txn_boundary() {
        tests::w_u8_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_u8_over_capacity_expands_storage() {
        tests::w_u8_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // w_u16() tests
    #[test]
    fn w_u16_returns_err_when_closed() {
        tests::w_u16_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_u16_returns_ok_when_open() {
        tests::w_u16_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_u16_does_not_write_when_closed() {
        tests::w_u16_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_u16_does_not_write_before_txn_boundary() {
        tests::w_u16_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_u16_over_capacity_expands_storage() {
        tests::w_u16_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // w_u32() tests
    #[test]
    fn w_u32_returns_err_when_closed() {
        tests::w_u32_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_u32_returns_ok_when_open() {
        tests::w_u32_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_u32_does_not_write_when_closed() {
        tests::w_u32_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_u32_does_not_write_before_txn_boundary() {
        tests::w_u32_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_u32_over_capacity_expands_storage() {
        tests::w_u32_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // w_u64() tests
    #[test]
    fn w_u64_returns_err_when_closed() {
        tests::w_u64_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_u64_returns_ok_when_open() {
        tests::w_u64_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_u64_does_not_write_when_closed() {
        tests::w_u64_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_u64_does_not_write_before_txn_boundary() {
        tests::w_u64_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_u64_over_capacity_expands_storage() {
        tests::w_u64_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // w_f32() tests
    #[test]
    fn w_f32_returns_err_when_closed() {
        tests::w_f32_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_f32_returns_ok_when_open() {
        tests::w_f32_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_f32_does_not_write_when_closed() {
        tests::w_f32_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_f32_does_not_write_before_txn_boundary() {
        tests::w_f32_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_f32_over_capacity_expands_storage() {
        tests::w_f32_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // w_f64() tests
    #[test]
    fn w_f64_returns_err_when_closed() {
        tests::w_f64_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_f64_returns_ok_when_open() {
        tests::w_f64_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_f64_does_not_write_when_closed() {
        tests::w_f64_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_f64_does_not_write_before_txn_boundary() {
        tests::w_f64_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_f64_over_capacity_expands_storage() {
        tests::w_f64_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // w_bool() tests
    #[test]
    fn w_bool_returns_err_when_closed() {
        tests::w_bool_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_bool_returns_ok_when_open() {
        tests::w_bool_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_bool_does_not_write_when_closed() {
        tests::w_bool_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_bool_does_not_write_before_txn_boundary() {
        tests::w_bool_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_bool_over_capacity_expands_storage() {
        tests::w_bool_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // w_bytes() tests
    #[test]
    fn w_bytes_returns_err_when_closed() {
        tests::w_bytes_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_bytes_returns_ok_when_open() {
        tests::w_bytes_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_bytes_does_not_write_when_closed() {
        tests::w_bytes_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_bytes_does_not_write_before_txn_boundary() {
        tests::w_bytes_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_bytes_over_capacity_expands_storage() {
        tests::w_bytes_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_bytes_over_capacity_expands_storage_multiple_times() {
        tests::w_bytes_over_capacity_expands_storage_multiple_times(
            MemoryBinaryStorage::new(256, 4, false).unwrap()
        );
    }

    // w_str() tests
    #[test]
    fn w_str_returns_err_when_closed() {
        tests::w_str_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_str_returns_ok_when_open() {
        tests::w_str_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_str_does_not_write_when_closed() {
        tests::w_str_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_str_does_not_write_before_txn_boundary() {
        tests::w_str_does_not_write_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_str_over_capacity_expands_storage() {
        tests::w_str_over_capacity_expands_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn w_str_over_capacity_expands_storage_multiple_times() {
        tests::w_str_over_capacity_expands_storage_multiple_times(
            MemoryBinaryStorage::new(256, 4, false).unwrap()
        );
    }

    // r_i8() tests
    #[test]
    fn r_i8_returns_err_when_closed() {
        tests::r_i8_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i8_returns_ok_when_open() {
        tests::r_i8_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i8_reads_zero_from_unwritten_storage() {
        tests::r_i8_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i8_reads_written_data() {
        tests::r_i8_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i8_does_not_read_past_txn_boundary() {
        tests::r_i8_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i8_does_not_read_past_capacity() {
        tests::r_i8_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i8_result_is_not_mutated_on_subsequent_write() {
        tests::r_i8_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // r_i16() tests
    #[test]
    fn r_i16_returns_err_when_closed() {
        tests::r_i16_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i16_returns_ok_when_open() {
        tests::r_i16_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i16_reads_zero_from_unwritten_storage() {
        tests::r_i16_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i16_reads_written_data() {
        tests::r_i16_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i16_does_not_read_past_txn_boundary() {
        tests::r_i16_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i16_does_not_read_past_capacity() {
        tests::r_i16_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i16_result_is_not_mutated_on_subsequent_write() {
        tests::r_i16_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // r_i32() tests
    #[test]
    fn r_i32_returns_err_when_closed() {
        tests::r_i32_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i32_returns_ok_when_open() {
        tests::r_i32_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i32_reads_zero_from_unwritten_storage() {
        tests::r_i32_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i32_reads_written_data() {
        tests::r_i32_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i32_does_not_read_past_txn_boundary() {
        tests::r_i32_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i32_does_not_read_past_capacity() {
        tests::r_i32_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i32_result_is_not_mutated_on_subsequent_write() {
        tests::r_i32_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // r_i64() tests
    #[test]
    fn r_i64_returns_err_when_closed() {
        tests::r_i64_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i64_returns_ok_when_open() {
        tests::r_i64_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i64_reads_zero_from_unwritten_storage() {
        tests::r_i64_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i64_reads_written_data() {
        tests::r_i64_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i64_does_not_read_past_txn_boundary() {
        tests::r_i64_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i64_does_not_read_past_capacity() {
        tests::r_i64_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_i64_result_is_not_mutated_on_subsequent_write() {
        tests::r_i64_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // r_u8() tests
    #[test]
    fn r_u8_returns_err_when_closed() {
        tests::r_u8_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u8_returns_ok_when_open() {
        tests::r_u8_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u8_reads_zero_from_unwritten_storage() {
        tests::r_u8_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u8_reads_written_data() {
        tests::r_u8_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u8_does_not_read_past_txn_boundary() {
        tests::r_u8_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u8_does_not_read_past_capacity() {
        tests::r_u8_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u8_result_is_not_mutated_on_subsequent_write() {
        tests::r_u8_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // r_u16() tests
    #[test]
    fn r_u16_returns_err_when_closed() {
        tests::r_u16_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u16_returns_ok_when_open() {
        tests::r_u16_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u16_reads_zero_from_unwritten_storage() {
        tests::r_u16_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u16_reads_written_data() {
        tests::r_u16_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u16_does_not_read_past_txn_boundary() {
        tests::r_u16_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u16_does_not_read_past_capacity() {
        tests::r_u16_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u16_result_is_not_mutated_on_subsequent_write() {
        tests::r_u16_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // r_u32() tests
    #[test]
    fn r_u32_returns_err_when_closed() {
        tests::r_u32_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u32_returns_ok_when_open() {
        tests::r_u32_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u32_reads_zero_from_unwritten_storage() {
        tests::r_u32_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u32_reads_written_data() {
        tests::r_u32_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u32_does_not_read_past_txn_boundary() {
        tests::r_u32_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u32_does_not_read_past_capacity() {
        tests::r_u32_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u32_result_is_not_mutated_on_subsequent_write() {
        tests::r_u32_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // r_u64() tests
    #[test]
    fn r_u64_returns_err_when_closed() {
        tests::r_u64_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u64_returns_ok_when_open() {
        tests::r_u64_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u64_reads_zero_from_unwritten_storage() {
        tests::r_u64_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u64_reads_written_data() {
        tests::r_u64_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u64_does_not_read_past_txn_boundary() {
        tests::r_u64_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u64_does_not_read_past_capacity() {
        tests::r_u64_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_u64_result_is_not_mutated_on_subsequent_write() {
        tests::r_u64_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // r_f32() tests
    #[test]
    fn r_f32_returns_err_when_closed() {
        tests::r_f32_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_f32_returns_ok_when_open() {
        tests::r_f32_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_f32_reads_zero_from_unwritten_storage() {
        tests::r_f32_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_f32_reads_written_data() {
        tests::r_f32_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_f32_does_not_read_past_txn_boundary() {
        tests::r_f32_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_f32_does_not_read_past_capacity() {
        tests::r_f32_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_f32_result_is_not_mutated_on_subsequent_write() {
        tests::r_f32_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // r_f64() tests
    #[test]
    fn r_f64_returns_err_when_closed() {
        tests::r_f64_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_f64_returns_ok_when_open() {
        tests::r_f64_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_f64_reads_zero_from_unwritten_storage() {
        tests::r_f64_reads_zero_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_f64_reads_written_data() {
        tests::r_f64_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_f64_does_not_read_past_txn_boundary() {
        tests::r_f64_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_f64_does_not_read_past_capacity() {
        tests::r_f64_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_f64_result_is_not_mutated_on_subsequent_write() {
        tests::r_f64_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // r_bool() tests
    #[test]
    fn r_bool_returns_err_when_closed() {
        tests::r_bool_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_bool_returns_ok_when_open() {
        tests::r_bool_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_bool_reads_false_from_unwritten_storage() {
        tests::r_bool_reads_false_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_bool_reads_written_data() {
        tests::r_bool_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_bool_does_not_read_past_txn_boundary() {
        tests::r_bool_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_bool_does_not_read_past_capacity() {
        tests::r_bool_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_bool_result_is_not_mutated_on_subsequent_write() {
        tests::r_bool_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // r_bytes() tests
    #[test]
    fn r_bytes_returns_err_when_closed() {
        tests::r_bytes_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_bytes_returns_ok_when_open() {
        tests::r_bytes_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_bytes_reads_zeros_from_unwritten_storage() {
        tests::r_bytes_reads_zeros_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_bytes_reads_written_data() {
        tests::r_bytes_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_bytes_does_not_read_past_txn_boundary() {
        tests::r_bytes_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_bytes_does_not_read_past_capacity() {
        tests::r_bytes_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_bytes_result_is_not_mutated_on_subsequent_write() {
        tests::r_bytes_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // r_str() tests
    #[test]
    fn r_str_returns_err_when_closed() {
        tests::r_str_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_str_returns_ok_when_open() {
        tests::r_str_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_str_reads_nulls_from_unwritten_storage() {
        tests::r_str_reads_nulls_from_unwritten_storage(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_str_reads_written_data() {
        tests::r_str_reads_written_data(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_str_does_not_read_past_txn_boundary() {
        tests::r_str_does_not_read_past_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_str_does_not_read_past_capacity() {
        tests::r_str_does_not_read_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn r_str_result_is_not_mutated_on_subsequent_write() {
        tests::r_str_result_is_not_mutated_on_subsequent_write(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // fill() tests
    #[test]
    fn fill_returns_err_when_closed() {
        tests::fill_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn fill_does_not_write_when_closed() {
        tests::fill_does_not_write_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn fill_returns_ok_when_open() {
        tests::fill_returns_ok_when_open(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn fill_repeats_byte_in_storage_range() {
        tests::fill_repeats_byte_in_storage_range(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn fill_starts_from_beginning_when_start_offset_is_none() {
        tests::fill_starts_from_beginning_when_start_offset_is_none(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn fill_goes_to_end_when_end_offset_is_none() {
        tests::fill_goes_to_end_when_end_offset_is_none(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn fill_returns_err_when_end_offset_is_before_start_offset() {
        tests::fill_returns_err_when_end_offset_is_before_start_offset(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn fill_does_not_write_when_end_offset_is_before_start_offset() {
        tests::fill_does_not_write_when_end_offset_is_before_start_offset(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn fill_returns_err_when_before_txn_boundary() {
        tests::fill_returns_err_when_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn fill_does_not_write_when_before_txn_boundary() {
        tests::fill_does_not_write_when_before_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn fill_returns_ok_when_after_txn_boundary() {
        tests::fill_returns_ok_when_after_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn fill_writes_when_after_txn_boundary() {
        tests::fill_writes_when_after_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn fill_returns_err_when_past_capacity() {
        tests::fill_returns_err_when_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn fill_does_not_write_when_past_capacity() {
        tests::fill_does_not_write_when_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn fill_does_not_expand_capacity() {
        tests::fill_does_not_expand_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // assert_filled() tests
    #[test]
    fn is_filled_retuns_err_when_closed() {
        tests::is_filled_retuns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn is_filled_returns_err_when_start_offset_past_capacity() {
        tests::is_filled_returns_err_when_start_offset_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn is_filled_returns_err_when_end_offset_at_or_before_start_offset() {
        tests::is_filled_returns_err_when_end_offset_at_or_before_start_offset(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn is_filled_returns_err_when_end_offset_past_capacity() {
        tests::is_filled_returns_err_when_end_offset_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn is_filled_checks_whether_all_bytes_in_range_match_value() {
        tests::is_filled_checks_whether_all_bytes_in_range_match_value(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn is_filled_starts_from_start_offset() {
        tests::is_filled_starts_from_start_offset(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn is_filled_starts_from_beginning_when_start_offset_is_none() {
        tests::is_filled_starts_from_beginning_when_start_offset_is_none(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn is_filled_goes_to_end_offset() {
        tests::is_filled_goes_to_end_offset(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn is_filled_goes_to_end_when_end_offset_is_none() {
        tests::is_filled_goes_to_end_when_end_offset_is_none(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // get_use_txn_boundary(), set_use_txn_boundary(), get_txn_boundary(), and set_txn_boundary() tests
    #[test]
    fn set_use_txn_boundary_changes_value() {
        tests::set_use_txn_boundary_changes_value(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn set_use_txn_boundary_resets_boundary_to_zero_when_txn_boundary_turned_off() {
        tests::set_use_txn_boundary_resets_boundary_to_zero_when_txn_boundary_turned_off(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn get_txn_boundary_returns_err_when_closed() {
        tests::get_txn_boundary_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn get_txn_boundary_returns_err_when_not_using_txn_boundary() {
        tests::get_txn_boundary_returns_err_when_not_using_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn get_txn_boundary_starts_at_0() {
        tests::get_txn_boundary_starts_at_0(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn set_txn_boundary_returns_err_when_not_using_txn_boundary() {
        tests::set_txn_boundary_returns_err_when_not_using_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn set_txn_boundary_does_not_change_boundary_when_not_using_txn_boundary() {
        tests::set_txn_boundary_does_not_change_boundary_when_not_using_txn_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn set_txn_boundary_returns_err_when_closed() {
        tests::set_txn_boundary_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn set_txn_boundary_does_not_change_boundary_when_closed() {
        tests::set_txn_boundary_does_not_change_boundary_when_closed(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn set_txn_boundary_returns_err_when_past_capacity() {
        tests::set_txn_boundary_returns_err_when_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn set_txn_boundary_does_not_change_boundary_when_past_capacity() {
        tests::set_txn_boundary_does_not_change_boundary_when_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn set_txn_boundary_does_not_expand_capacity_when_past_capacity() {
        tests::set_txn_boundary_does_not_expand_capacity_when_past_capacity(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    #[test]
    fn set_txn_boundary_changes_boundary() {
        tests::set_txn_boundary_changes_boundary(
            MemoryBinaryStorage::new(256, 256, false).unwrap()
        );
    }

    // get_expand_size() and set_expand_size() tests
    #[test]
    fn get_expand_size_returns_initial_expand_size() {
        tests::get_expand_size_returns_initial_expand_size(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn set_expand_size_returns_err_when_expand_size_is_zero() {
        tests::set_expand_size_returns_err_when_expand_size_is_zero(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn set_expand_size_does_not_change_expand_size_when_expand_size_is_zero() {
        tests::set_expand_size_does_not_change_expand_size_when_expand_size_is_zero(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn set_expand_size_returns_err_when_expand_size_is_not_power_of_2() {
        tests::set_expand_size_returns_err_when_expand_size_is_not_power_of_2(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn set_expand_size_does_not_change_expand_size_when_expand_size_is_not_power_of_2() {
        tests::set_expand_size_does_not_change_expand_size_when_expand_size_is_not_power_of_2(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn set_expand_size_returns_true_when_checks_pass() {
        tests::set_expand_size_returns_true_when_checks_pass(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn set_expand_size_changes_expand_size_when_checks_pass() {
        tests::set_expand_size_changes_expand_size_when_checks_pass(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn capacity_increases_to_increments_of_last_set_expand_size() {
        tests::capacity_increases_to_increments_of_last_set_expand_size(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    // get_capacity() tests
    #[test]
    fn get_capacity_returns_err_when_closed() {
        tests::get_capacity_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn get_capacity_returns_initial_capacity_when_open() {
        tests::get_capacity_returns_initial_capacity_when_open(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn get_capacity_returns_new_capacity_after_expansion() {
        tests::get_capacity_returns_new_capacity_after_expansion(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    // expand() tests
    #[test]
    fn expand_returns_err_when_closed() {
        tests::expand_returns_err_when_closed(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn expand_does_not_change_capacity_when_closed() {
        tests::expand_does_not_change_capacity_when_closed(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn expand_returns_ok_when_already_has_capacity() {
        tests::expand_returns_ok_when_already_has_capacity(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn expand_does_not_change_capacity_when_already_has_capacity() {
        tests::expand_does_not_change_capacity_when_already_has_capacity(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn expand_returns_err_when_allocation_arithmetic_overflows() {
        tests::expand_returns_err_when_allocation_arithmetic_overflows(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn expand_does_not_change_capacity_when_allocation_arithmetic_overflows() {
        tests::expand_does_not_change_capacity_when_allocation_arithmetic_overflows(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn expand_returns_err_when_allocation_fails() {
        tests::expand_returns_err_when_allocation_fails(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn expand_does_not_change_capacity_when_allocation_fails() {
        tests::expand_does_not_change_capacity_when_allocation_fails(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn expand_returns_ok_when_successful() {
        tests::expand_returns_ok_when_successful(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn expand_changes_capacity_by_expand_size_when_successful() {
        tests::expand_changes_capacity_by_expand_size_when_successful(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }

    #[test]
    fn expand_changes_capacity_by_multiples_of_expand_size_when_successful() {
        tests::expand_changes_capacity_by_multiples_of_expand_size_when_successful(
            MemoryBinaryStorage::new(256, 512, false).unwrap()
        );
    }


}
