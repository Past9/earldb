#![feature(alloc, heap_api)]

extern crate alloc;
extern crate core;

use std::str;
use alloc::heap;
use std::{mem, ptr, slice};
use storage::binary_storage::BinaryStorage;


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
    ) -> Option<MemoryBinaryStorage> {

        if !MemoryBinaryStorage::check_mem_params(
            align,
            expand_size,
            initial_capacity,
            max_page_size
        ) { return None };

        let origin = unsafe { heap::allocate(initial_capacity, align) as *mut u8 };
        unsafe { ptr::write_bytes::<u8>(origin, 0x0, initial_capacity) };

        Some(MemoryBinaryStorage {
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

    fn ptr<T>(&self, offset: usize) -> *const T {
        (self.origin as usize + offset) as *const T
    }

    fn ptr_mut<T>(&mut self, offset: usize) -> *mut T {
        (self.origin as usize + offset) as *mut T
    }

    fn write<T>(&mut self, offset: usize, data: T) -> bool {
        if !self.is_open { return false }
        if self.use_txn_boundary && offset < self.txn_boundary { return false }
        if !self.expand(offset + mem::size_of::<T>()) { return false }

        unsafe { ptr::write(self.ptr_mut(offset), data) }
        true
    }

    fn read<T: Copy>(&self, offset: usize) -> Option<T> {
        if !self.is_open { return None }
        if self.use_txn_boundary && (offset + mem::size_of::<T>()) > self.txn_boundary { return None }
        if offset + mem::size_of::<T>() > self.capacity { return None }

        unsafe { Some(ptr::read(self.ptr(offset))) }
    }

    fn is_power_of_two(n: usize) -> bool {
        return (n != 0) && (n & (n - 1)) == 0;
    }

    fn check_mem_params(
        align: usize,
        expand_size: usize,
        initial_capacity: usize,
        max_page_size: usize
    ) -> bool {
        // Initial capacity and expansion size must be greater than zero
        if initial_capacity < 1 || expand_size < 1 { return false }
        // Max page size must be a power of 2 
        if !MemoryBinaryStorage::is_power_of_two(max_page_size) { return false }
        // Alignment must be a power of 2
        if !MemoryBinaryStorage::is_power_of_two(align) { return false }
        // Initial capacity must be a power of 2
        if !MemoryBinaryStorage::is_power_of_two(initial_capacity) { return false }
        // Expansion size must be a power of 2
        if !MemoryBinaryStorage::is_power_of_two(expand_size) { return false }
        // Alignment must be no larger than max page size
        if align > max_page_size { return false }
        // If all checks pass, return true
        true
    }


}
impl BinaryStorage for MemoryBinaryStorage {

    fn open(&mut self) -> bool {
        if self.is_open { return false }
        self.is_open = true;
        true
    }

    fn close(&mut self) -> bool {
        if !self.is_open { return false }
        self.is_open = false;
        true
    }

    fn w_i8(&mut self, offset: usize, data: i8) -> bool { self.write(offset, data) }
    fn w_i16(&mut self, offset: usize, data: i16) -> bool { self.write(offset, data) }
    fn w_i32(&mut self, offset: usize, data: i32) -> bool { self.write(offset, data) }
    fn w_i64(&mut self, offset: usize, data: i64) -> bool { self.write(offset, data) }

    fn w_u8(&mut self, offset: usize, data: u8) -> bool { self.write(offset, data) }
    fn w_u16(&mut self, offset: usize, data: u16) -> bool { self.write(offset, data) }
    fn w_u32(&mut self, offset: usize, data: u32) -> bool { self.write(offset, data) }
    fn w_u64(&mut self, offset: usize, data: u64) -> bool { self.write(offset, data) }

    fn w_f32(&mut self, offset: usize, data: f32) -> bool { self.write(offset, data) }
    fn w_f64(&mut self, offset: usize, data: f64) -> bool { self.write(offset, data) }

    fn w_bool(&mut self, offset: usize, data: bool) -> bool { self.write(offset, data) }

    fn w_bytes(&mut self, offset: usize, data: &[u8]) -> bool {
        if !self.is_open { return false }
        if self.use_txn_boundary && offset < self.txn_boundary { return false }
        if !self.expand(offset + data.len()) { return false }

        let dest = unsafe { slice::from_raw_parts_mut(self.ptr_mut(offset), data.len()) };
        dest.clone_from_slice(data);

        true
    }

    fn w_str(&mut self, offset: usize, data: &str) -> bool { self.w_bytes(offset, data.as_bytes()) }


    fn r_i8(&self, offset: usize) -> Option<i8> { self.read(offset) }
    fn r_i16(&self, offset: usize) -> Option<i16> { self.read(offset) }
    fn r_i32(&self, offset: usize) -> Option<i32> { self.read(offset) }
    fn r_i64(&self, offset: usize) -> Option<i64> { self.read(offset) }

    fn r_u8(&self, offset: usize) -> Option<u8> { self.read(offset) }
    fn r_u16(&self, offset: usize) -> Option<u16> { self.read(offset) }
    fn r_u32(&self, offset: usize) -> Option<u32> { self.read(offset) }
    fn r_u64(&self, offset: usize) -> Option<u64> { self.read(offset) }

    fn r_f32(&self, offset: usize) -> Option<f32> { self.read(offset) }
    fn r_f64(&self, offset: usize) -> Option<f64> { self.read(offset) }

    fn r_bool(&self, offset: usize) -> Option<bool> { self.read(offset) }

    fn r_bytes(&self, offset: usize, len: usize) -> Option<&[u8]> {
        if !self.is_open { return None }
        if self.use_txn_boundary && (offset + len) > self.txn_boundary { return None }
        if offset + len > self.capacity { return None }

        unsafe { Some(slice::from_raw_parts(self.ptr(offset), len)) }
    }

    fn r_str(&self, offset: usize, len: usize) -> Option<&str> {
        match self.r_bytes(offset, len) {
            Some(v) => match str::from_utf8(v) {
                Ok(s) => Some(s),
                Err(e) => None
            },
            None => None
        }
    }


    fn fill(&mut self, start: Option<usize>, end: Option<usize>, val: u8) -> bool {
        if !self.is_open { return false }

        let start_offset = match start { Some(s) => s, None => 0 };

        if start_offset >= self.capacity { return false }
        if self.use_txn_boundary && start_offset < self.txn_boundary { return false }

        let end_offset = match end { Some(e) => e, None => self.capacity };

        if end_offset <= start_offset { return false }
        if end_offset > self.capacity { return false }

        unsafe { ptr::write_bytes::<u8>(self.ptr_mut(start_offset), val, end_offset - start_offset) }

        true
    }

    fn assert_filled(&self, start: Option<usize>, end: Option<usize>, val: u8) -> bool {
        if !self.is_open { return false }

        let start_offset = match start {
            Some(s) => s,
            None => 0
        };

        if start_offset >= self.capacity { return false }

        let end_offset = match end {
            Some(e) => e,
            None => self.capacity
        };

        if end_offset <= start_offset { return false }
        if end_offset > self.capacity { return false }

        let data = unsafe {
            slice::from_raw_parts::<u8>(self.ptr(start_offset), end_offset - start_offset)
        };

        for b in data {
            if *b != val { return false }
        }
        true
    }


    fn get_use_txn_boundary(&self) -> bool {
        self.use_txn_boundary
    }

    fn set_use_txn_boundary(&mut self, val: bool) {
        self.use_txn_boundary = val;
        if !val { self.txn_boundary = 0 }
    }


    fn get_txn_boundary(&self) -> usize {
        self.txn_boundary
    }

    fn set_txn_boundary(&mut self, offset: usize) -> bool {
        if !self.is_open { return false }
        if !self.use_txn_boundary { return false }
        if offset > self.capacity { return false }

        self.txn_boundary = offset;
        true
    }


    fn get_expand_size(&self) -> usize {
        self.expand_size
    }

    fn set_expand_size(&mut self, expand_size: usize) -> bool {
        if !MemoryBinaryStorage::check_mem_params(
            self.align,
            expand_size,
            self.capacity,
            self.max_page_size
        ) { return false }

        self.expand_size = expand_size;
        true
    }

    fn get_align(&self) -> usize {
        self.align
    }

    fn set_align(&mut self, align: usize) -> bool {
        if !MemoryBinaryStorage::check_mem_params(
            align,
            self.expand_size,
            self.capacity,
            self.max_page_size
        ) { return false }

        self.align = align;
        true
    }


    fn expand(&mut self, min_capacity: usize) -> bool {
        if !self.is_open { return false }

        // Determine the new size of the journal in multiples of expand_size
        let expand_increments = (min_capacity as f64 / self.expand_size as f64).ceil() as usize;
        let new_capacity = match expand_increments.checked_mul(self.expand_size) {
            Some(x) => x,
            None => return false
        };

        // We don't want to reallocate (or even reduce the capacity) if we already have enough,
        // so just do nothing and return true if we already have enough room
        if new_capacity < self.capacity { return true }


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
            return false;
        } else {
            // Set the new capacity and pointer, remembering the old capacity
            let old_capacity = self.capacity;
            self.origin = ptr as *const u8;
            self.capacity = new_capacity;
            // Initialize the new storage (set all bytes to 0x00)
            self.fill(Some(old_capacity), Some(new_capacity), 0x0);
            // Return true to indicate that allocation was successful
            return true;
        }
    }

    fn get_capacity(&self) -> usize {
        if !self.is_open { return 0 }
        self.capacity
    }

    fn get_max_page_size(&self) -> usize {
        self.max_page_size
    }

    fn is_open(&self) -> bool {
        self.is_open
    }


}


#[cfg(test)]
mod tests {

    use std::{mem, str};

    use storage::binary_storage::BinaryStorage;
    use storage::memory_binary_storage::MemoryBinaryStorage;


    // open(), close(), and is_open() tests 
    #[test]
    fn is_closed_when_new() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.is_open());
    }

    #[test]
    fn is_open_after_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.is_open());
    }

    #[test]
    fn is_closed_after_open_and_close() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.close();
        assert!(!s.is_open());
    }

    // new() tests
    #[test]
    fn new_sets_initial_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap();
        s.open();
        assert_eq!(256, s.get_capacity());
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
        assert!(s.is_none());
    }

    #[test]
    fn new_requires_expand_size_greater_than_0() {
        let s = MemoryBinaryStorage::new(256, 0, false, 1024, 4096);
        assert!(s.is_none());
    }

    #[test]
    fn new_requires_max_page_size_is_power_of_2() {
        let s1 = MemoryBinaryStorage::new(256, 512, false, 1024, 2048);
        assert!(s1.is_some());
        let s2 = MemoryBinaryStorage::new(256, 512, false, 1024, 2049);
        assert!(s2.is_none());
        let s3 = MemoryBinaryStorage::new(256, 512, false, 1024, 3072);
        assert!(s3.is_none());
        let s4 = MemoryBinaryStorage::new(256, 512, false, 1024, 4096);
        assert!(s4.is_some());
    }

    #[test]
    fn new_requires_alignment_is_power_of_2() {
        let s1 = MemoryBinaryStorage::new(256, 512, false, 1024, 4096);
        assert!(s1.is_some());
        let s2 = MemoryBinaryStorage::new(256, 512, false, 1025, 4096);
        assert!(s2.is_none());
        let s3 = MemoryBinaryStorage::new(256, 512, false, 1536, 4096);
        assert!(s3.is_none());
        let s4 = MemoryBinaryStorage::new(256, 512, false, 2048, 4096);
        assert!(s4.is_some());
    }

    #[test]
    fn new_requires_initial_capacity_is_power_of_2() {
        let s1 = MemoryBinaryStorage::new(256, 512, false, 1024, 4096);
        assert!(s1.is_some());
        let s2 = MemoryBinaryStorage::new(257, 512, false, 1024, 4096);
        assert!(s2.is_none());
        let s3 = MemoryBinaryStorage::new(384, 512, false, 1024, 4096);
        assert!(s3.is_none());
        let s4 = MemoryBinaryStorage::new(512, 512, false, 1024, 4096);
        assert!(s4.is_some());
    }

    #[test]
    fn new_requires_expand_size_is_power_of_2() {
        let s1 = MemoryBinaryStorage::new(256, 512, false, 1024, 4096);
        assert!(s1.is_some());
        let s2 = MemoryBinaryStorage::new(256, 513, false, 1024, 4096);
        assert!(s2.is_none());
        let s3 = MemoryBinaryStorage::new(256, 768, false, 1024, 4096);
        assert!(s3.is_none());
        let s4 = MemoryBinaryStorage::new(256, 1024, false, 1024, 4096);
        assert!(s4.is_some());
    }

    #[test]
    fn new_requires_alignment_no_larger_than_max_page_size() {
        let s1 = MemoryBinaryStorage::new(256, 512, false, 512, 1024);
        assert!(s1.is_some());
        let s2 = MemoryBinaryStorage::new(256, 512, false, 1024, 1024);
        assert!(s2.is_some());
        let s3 = MemoryBinaryStorage::new(256, 512, false, 2048, 1024);
        assert!(s3.is_none());
    }

    #[test]
    fn new_initializes_memory_to_zeros() {
        let mut s = MemoryBinaryStorage::new(256, 512, false, 512, 1024).unwrap();
        s.open();
        assert!(s.assert_filled(None, None, 0x0));
    }

    // w_i8() tests
    #[test]
    fn w_i8_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_i8(0, i8::max_value()));
    }

    #[test]
    fn w_i8_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_i8(0, i8::max_value()));
    }

    #[test]
    fn w_i8_does_not_write_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.w_i8(0, i8::max_value());
        s.open();
        assert_eq!(0, s.r_i8(0).unwrap());
    }

    #[test]
    fn w_i8_does_not_write_before_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(4);
        assert!(!s.w_i8(3, i8::max_value()));
        assert!(s.w_i8(4, i8::max_value()));
        s.set_txn_boundary(16);
        assert_eq!(0, s.r_i8(3).unwrap());
        assert_eq!(i8::max_value(), s.r_i8(4).unwrap());
    }

    #[test]
    fn w_i8_over_capacity_expands_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_i8(256, i8::max_value()));
        assert_eq!(512, s.get_capacity());
        assert_eq!(i8::max_value(), s.r_i8(256).unwrap());
    }

    // w_i16() tests
    #[test]
    fn w_i16_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_i16(0, i16::max_value()));
    }

    #[test]
    fn w_i16_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_i16(0, i16::max_value()));
    }

    #[test]
    fn w_i16_does_not_write_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.w_i16(0, i16::max_value());
        s.open();
        assert_eq!(0, s.r_i16(0).unwrap());
    }

    #[test]
    fn w_i16_does_not_write_before_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(4);
        assert!(!s.w_i16(3, i16::max_value()));
        assert!(s.w_i16(4, i16::max_value()));
        s.set_txn_boundary(16);
        assert_eq!(0, s.r_i16(2).unwrap());
        assert_eq!(i16::max_value(), s.r_i16(4).unwrap());
    }

    #[test]
    fn w_i16_over_capacity_expands_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_i16(256, i16::max_value()));
        assert_eq!(512, s.get_capacity());
        assert_eq!(i16::max_value(), s.r_i16(256).unwrap());
    }

    // w_i32() tests
    #[test]
    fn w_i32_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_i32(0, i32::max_value()));
    }

    #[test]
    fn w_i32_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_i32(0, i32::max_value()));
    }

    #[test]
    fn w_i32_does_not_write_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.w_i32(0, i32::max_value());
        s.open();
        assert_eq!(0, s.r_i32(0).unwrap());
    }

    #[test]
    fn w_i32_does_not_write_before_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(8);
        assert!(!s.w_i32(7, i32::max_value()));
        assert!(s.w_i32(8, i32::max_value()));
        s.set_txn_boundary(16);
        assert_eq!(0, s.r_i32(4).unwrap());
        assert_eq!(i32::max_value(), s.r_i32(8).unwrap());
    }

    #[test]
    fn w_i32_over_capacity_expands_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_i32(256, i32::max_value()));
        assert_eq!(512, s.get_capacity());
        assert_eq!(i32::max_value(), s.r_i32(256).unwrap());
    }

    // w_i64() tests
    #[test]
    fn w_i64_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_i64(0, i64::max_value()));
    }

    #[test]
    fn w_i64_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_i64(0, i64::max_value()));
    }

    #[test]
    fn w_i64_does_not_write_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.w_i64(0, i64::max_value());
        s.open();
        assert_eq!(0, s.r_i64(0).unwrap());
    }

    #[test]
    fn w_i64_does_not_write_before_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(8);
        assert!(!s.w_i64(7, i64::max_value()));
        assert!(s.w_i64(8, i64::max_value()));
        s.set_txn_boundary(16);
        assert_eq!(0, s.r_i64(0).unwrap());
        assert_eq!(i64::max_value(), s.r_i64(8).unwrap());
    }

    #[test]
    fn w_i64_over_capacity_expands_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_i64(256, i64::max_value()));
        assert_eq!(512, s.get_capacity());
        assert_eq!(i64::max_value(), s.r_i64(256).unwrap());
    }

    // w_u8() tests
    #[test]
    fn w_u8_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_u8(0, u8::max_value()));
    }

    #[test]
    fn w_u8_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_u8(0, u8::max_value()));
    }

    #[test]
    fn w_u8_does_not_write_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.w_u8(0, u8::max_value());
        s.open();
        assert_eq!(0, s.r_u8(0).unwrap());
    }

    #[test]
    fn w_u8_does_not_write_before_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(4);
        assert!(!s.w_u8(3, u8::max_value()));
        assert!(s.w_u8(4, u8::max_value()));
        s.set_txn_boundary(16);
        assert_eq!(0, s.r_u8(3).unwrap());
        assert_eq!(u8::max_value(), s.r_u8(4).unwrap());
    }

    #[test]
    fn w_u8_over_capacity_expands_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_u8(256, u8::max_value()));
        assert_eq!(512, s.get_capacity());
        assert_eq!(u8::max_value(), s.r_u8(256).unwrap());
    }

    // w_u16() tests
    #[test]
    fn w_u16_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_u16(0, u16::max_value()));
    }

    #[test]
    fn w_u16_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_u16(0, u16::max_value()));
    }

    #[test]
    fn w_u16_does_not_write_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.w_u16(0, u16::max_value());
        s.open();
        assert_eq!(0, s.r_u16(0).unwrap());
    }

    #[test]
    fn w_u16_does_not_write_before_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(4);
        assert!(!s.w_u16(3, u16::max_value()));
        assert!(s.w_u16(4, u16::max_value()));
        s.set_txn_boundary(16);
        assert_eq!(0, s.r_u16(2).unwrap());
        assert_eq!(u16::max_value(), s.r_u16(4).unwrap());
    }

    #[test]
    fn w_u16_over_capacity_expands_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_u16(256, u16::max_value()));
        assert_eq!(512, s.get_capacity());
        assert_eq!(u16::max_value(), s.r_u16(256).unwrap());
    }

    // w_u32() tests
    #[test]
    fn w_u32_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_u32(0, u32::max_value()));
    }

    #[test]
    fn w_u32_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_u32(0, u32::max_value()));
    }

    #[test]
    fn w_u32_does_not_write_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.w_u32(0, u32::max_value());
        s.open();
        assert_eq!(0, s.r_u32(0).unwrap());
    }

    #[test]
    fn w_u32_does_not_write_before_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(8);
        assert!(!s.w_u32(7, u32::max_value()));
        assert!(s.w_u32(8, u32::max_value()));
        s.set_txn_boundary(16);
        assert_eq!(0, s.r_u32(4).unwrap());
        assert_eq!(u32::max_value(), s.r_u32(8).unwrap());
    }

    #[test]
    fn w_u32_over_capacity_expands_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_u32(256, u32::max_value()));
        assert_eq!(512, s.get_capacity());
        assert_eq!(u32::max_value(), s.r_u32(256).unwrap());
    }

    // w_u64() tests
    #[test]
    fn w_u64_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_u64(0, u64::max_value()));
    }

    #[test]
    fn w_u64_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_u64(0, u64::max_value()));
    }

    #[test]
    fn w_u64_does_not_write_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.w_u64(0, u64::max_value());
        s.open();
        assert_eq!(0, s.r_u64(0).unwrap());
    }

    #[test]
    fn w_u64_does_not_write_before_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(8);
        assert!(!s.w_u64(7, u64::max_value()));
        assert!(s.w_u64(8, u64::max_value()));
        s.set_txn_boundary(16);
        assert_eq!(0, s.r_u64(0).unwrap());
        assert_eq!(u64::max_value(), s.r_u64(8).unwrap());
    }

    #[test]
    fn w_u64_over_capacity_expands_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_u64(256, u64::max_value()));
        assert_eq!(512, s.get_capacity());
        assert_eq!(u64::max_value(), s.r_u64(256).unwrap());
    }

    // w_f32() tests
    #[test]
    fn w_f32_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_f32(0, 12345.6789));
    }

    #[test]
    fn w_f32_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_f32(0, 12345.6789));
    }

    #[test]
    fn w_f32_does_not_write_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.w_f32(0, 12345.6789);
        s.open();
        assert_eq!(0.0, s.r_f32(0).unwrap());
    }

    #[test]
    fn w_f32_does_not_write_before_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(8);
        assert!(!s.w_f32(7, 12345.6789));
        assert!(s.w_f32(8, 12345.6789));
        s.set_txn_boundary(16);
        assert_eq!(0.0, s.r_f32(4).unwrap());
        assert_eq!(12345.6789, s.r_f32(8).unwrap());
    }

    #[test]
    fn w_f32_over_capacity_expands_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_f32(256, 12345.6789));
        assert_eq!(512, s.get_capacity());
        assert_eq!(12345.6789, s.r_f32(256).unwrap());
    }

    // w_f64() tests
    #[test]
    fn w_f64_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_f64(0, 12345.6789));
    }

    #[test]
    fn w_f64_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_f64(0, 12345.6789));
    }

    #[test]
    fn w_f64_does_not_write_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.w_f64(0, 12345.6789);
        s.open();
        assert_eq!(0.0, s.r_f64(0).unwrap());
    }

    #[test]
    fn w_f64_does_not_write_before_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(8);
        assert!(!s.w_f64(7, 12345.6789));
        assert!(s.w_f64(8, 12345.6789));
        s.set_txn_boundary(16);
        assert_eq!(0.0, s.r_f64(0).unwrap());
        assert_eq!(12345.6789, s.r_f64(8).unwrap());
    }

    #[test]
    fn w_f64_over_capacity_expands_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_f64(256, 12345.6789));
        assert_eq!(512, s.get_capacity());
        assert_eq!(12345.6789, s.r_f64(256).unwrap());
    }

    // w_bool() tests
    #[test]
    fn w_bool_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_bool(0, false));
        assert!(!s.w_bool(0, true));
    }

    #[test]
    fn w_bool_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_bool(0, false));
        assert!(s.w_bool(0, true));
    }

    #[test]
    fn w_bool_does_not_write_when_closed() {
        let mut s1 = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s1.w_bool(0, false);
        s1.open();
        assert_eq!(false, s1.r_bool(0).unwrap());

        let mut s2 = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s2.w_bool(0, true);
        s2.open();
        assert_eq!(false, s2.r_bool(0).unwrap());
    }

    #[test]
    fn w_bool_does_not_write_before_txn_boundary() {
        let mut s1 = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s1.open();
        s1.set_txn_boundary(4);
        assert!(!s1.w_bool(3, false));
        assert!(s1.w_bool(4, false));
        s1.set_txn_boundary(8);
        assert_eq!(false, s1.r_bool(3).unwrap());
        assert_eq!(false, s1.r_bool(4).unwrap());

        let mut s2 = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s2.open();
        s2.set_txn_boundary(4);
        assert!(!s2.w_bool(3, true));
        assert!(s2.w_bool(4, true));
        s2.set_txn_boundary(8);
        assert_eq!(false, s2.r_bool(3).unwrap());
        assert_eq!(true, s2.r_bool(4).unwrap());
    }

    #[test]
    fn w_bool_over_capacity_expands_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_bool(256, true));
        assert_eq!(512, s.get_capacity());
        assert_eq!(true, s.r_bool(256).unwrap());
    }

    // w_bytes() tests
    #[test]
    fn w_bytes_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_bytes(0, &[0x0, 0x1, 0x2, 0x3, 0x4]));
    }

    #[test]
    fn w_bytes_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_bytes(0, &[0x0, 0x1, 0x2, 0x3, 0x4]));
    }

    #[test]
    fn w_bytes_does_not_write_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.w_bytes(0, &[0x0, 0x1, 0x2, 0x3, 0x4]);
        s.open();
        assert_eq!(&[0x0, 0x0, 0x0, 0x0, 0x0], s.r_bytes(0, 5).unwrap());
    }

    #[test]
    fn w_bytes_does_not_write_before_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(8);
        assert!(!s.w_bytes(7, &[0x0, 0x1, 0x2, 0x3, 0x4]));
        assert!(s.w_bytes(8, &[0x0, 0x1, 0x2, 0x3, 0x4]));
        s.set_txn_boundary(16);
        assert_eq!(&[0x0, 0x0, 0x0, 0x0, 0x0], s.r_bytes(3, 5).unwrap());
        assert_eq!(&[0x0, 0x1, 0x2, 0x3, 0x4], s.r_bytes(8, 5).unwrap());
    }

    #[test]
    fn w_bytes_over_capacity_expands_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_bytes(255, &[0x0, 0x1]));
        assert_eq!(512, s.get_capacity());
        assert_eq!(&[0x0, 0x1], s.r_bytes(255, 2).unwrap());
    }

    #[test]
    fn w_bytes_over_capacity_expands_storage_multiple_times() {
        let mut s = MemoryBinaryStorage::new(256, 4, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_bytes(255, &[0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6]));
        assert_eq!(264, s.get_capacity());
        assert_eq!(&[0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6], s.r_bytes(255, 7).unwrap());
    }

    // w_str() tests
    #[test]
    fn w_str_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_str(0, "foobar"));
        assert!(!s.w_str(0, "I \u{2661} Rust"));
    }

    #[test]
    fn w_str_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_str(0, "foobar"));
        assert!(s.w_str(0, "I \u{2661} Rust"));
    }

    #[test]
    fn w_str_does_not_write_when_closed() {
        let mut s1 = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s1.w_str(0, "foobar");
        s1.open();
        assert_eq!(str::from_utf8(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x0]).unwrap(), s1.r_str(0, 6).unwrap());

        let mut s2 = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s2.w_str(0, "I \u{2661} Rust");
        s2.open();
        assert_eq!(
            str::from_utf8(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0]).unwrap(), 
            s2.r_str(0, 10).unwrap()
        );
    }

    #[test]
    fn w_str_does_not_write_before_txn_boundary() {
        let mut s1 = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s1.open();
        s1.set_txn_boundary(8);
        assert!(!s1.w_str(7, "foobar"));
        assert!(s1.w_str(8, "foobar"));
        s1.set_txn_boundary(16);
        assert_eq!(str::from_utf8(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x0]).unwrap(), s1.r_str(2, 6).unwrap());
        assert_eq!("foobar", s1.r_str(8, 6).unwrap());

        let mut s2 = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s2.open();
        s2.set_txn_boundary(16);
        assert!(!s2.w_str(15, "I \u{2661} Rust"));
        assert!(s2.w_str(16, "I \u{2661} Rust"));
        s2.set_txn_boundary(32);
        assert_eq!(
            str::from_utf8(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0]).unwrap(), 
            s2.r_str(6, 10).unwrap()
        );
        assert_eq!("I \u{2661} Rust", s2.r_str(16, 10).unwrap());
    }

    #[test]
    fn w_str_over_capacity_expands_storage() {
        let mut s1 = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s1.open();
        assert_eq!(256, s1.get_capacity());
        assert!(s1.w_str(255, "foobar"));
        assert_eq!(512, s1.get_capacity());
        assert_eq!("foobar", s1.r_str(255, 6).unwrap());

        let mut s2 = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s2.open();
        assert_eq!(256, s2.get_capacity());
        assert!(s2.w_str(255, "I \u{2661} Rust"));
        assert_eq!(512, s2.get_capacity());
        assert_eq!("I \u{2661} Rust", s2.r_str(255, 10).unwrap());
    }

    #[test]
    fn w_str_over_capacity_expands_storage_multiple_times() {
        let mut s1 = MemoryBinaryStorage::new(256, 4, false, 256, 4096).unwrap();
        s1.open();
        assert_eq!(256, s1.get_capacity());
        assert!(s1.w_str(255, "foobar"));
        assert_eq!(264, s1.get_capacity());
        assert_eq!("foobar", s1.r_str(255, 6).unwrap());

        let mut s2 = MemoryBinaryStorage::new(256, 4, false, 256, 4096).unwrap();
        s2.open();
        assert_eq!(256, s2.get_capacity());
        assert!(s2.w_str(255, "I \u{2661} Rust"));
        assert_eq!(268, s2.get_capacity());
        assert_eq!("I \u{2661} Rust", s2.r_str(255, 10).unwrap());
    }

    // r_i8() tests
    #[test]
    fn r_i8_returns_none_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(s.r_i8(0).is_none());
    }

    #[test]
    fn r_i8_returns_some_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_i8(0).is_some());
    }

    #[test]
    fn r_i8_reads_zero_from_unwritten_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(0, s.r_i8(0).unwrap());
    }

    #[test]
    fn r_i8_reads_written_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_i8(0, i8::max_value());
        assert_eq!(i8::max_value(), s.r_i8(0).unwrap());
        s.w_i8(32, i8::max_value());
        assert_eq!(i8::max_value(), s.r_i8(32).unwrap());
    }

    #[test]
    fn r_i8_does_not_read_past_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(4);
        assert!(s.r_i8(3).is_some());
        assert!(s.r_i8(4).is_none());
    }

    #[test]
    fn r_i8_does_not_read_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_i8(255).is_some());
        assert!(s.r_i8(256).is_none());
    }

    // r_i16() tests
    #[test]
    fn r_i16_returns_none_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(s.r_i16(0).is_none());
    }

    #[test]
    fn r_i16_returns_some_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_i16(0).is_some());
    }

    #[test]
    fn r_i16_reads_zero_from_unwritten_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(0, s.r_i16(0).unwrap());
    }

    #[test]
    fn r_i16_reads_written_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_i16(0, i16::max_value());
        assert_eq!(i16::max_value(), s.r_i16(0).unwrap());
        s.w_i16(32, i16::max_value());
        assert_eq!(i16::max_value(), s.r_i16(32).unwrap());
    }

    #[test]
    fn r_i16_does_not_read_past_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(4);
        assert!(s.r_i16(2).is_some());
        assert!(s.r_i16(3).is_none());
        assert!(s.r_i16(4).is_none());
    }

    #[test]
    fn r_i16_does_not_read_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_i16(254).is_some());
        assert!(s.r_i16(255).is_none());
        assert!(s.r_i16(256).is_none());
    }

    // r_i32() tests
    #[test]
    fn r_i32_returns_none_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(s.r_i32(0).is_none());
    }

    #[test]
    fn r_i32_returns_some_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_i32(0).is_some());
    }

    #[test]
    fn r_i32_reads_zero_from_unwritten_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(0, s.r_i32(0).unwrap());
    }

    #[test]
    fn r_i32_reads_written_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_i32(0, i32::max_value());
        assert_eq!(i32::max_value(), s.r_i32(0).unwrap());
        s.w_i32(32, i32::max_value());
        assert_eq!(i32::max_value(), s.r_i32(32).unwrap());
    }

    #[test]
    fn r_i32_does_not_read_past_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(8);
        assert!(s.r_i32(4).is_some());
        assert!(s.r_i32(6).is_none());
        assert!(s.r_i32(8).is_none());
    }

    #[test]
    fn r_i32_does_not_read_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_i32(252).is_some());
        assert!(s.r_i32(254).is_none());
        assert!(s.r_i32(256).is_none());
    }

    // r_i64() tests
    #[test]
    fn r_i64_returns_none_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(s.r_i64(0).is_none());
    }

    #[test]
    fn r_i64_returns_some_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_i64(0).is_some());
    }

    #[test]
    fn r_i64_reads_zero_from_unwritten_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(0, s.r_i64(0).unwrap());
    }

    #[test]
    fn r_i64_reads_written_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_i64(0, i64::max_value());
        assert_eq!(i64::max_value(), s.r_i64(0).unwrap());
        s.w_i64(32, i64::max_value());
        assert_eq!(i64::max_value(), s.r_i64(32).unwrap());
    }

    #[test]
    fn r_i64_does_not_read_past_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(16);
        assert!(s.r_i64(8).is_some());
        assert!(s.r_i64(12).is_none());
        assert!(s.r_i64(16).is_none());
    }

    #[test]
    fn r_i64_does_not_read_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_i64(248).is_some());
        assert!(s.r_i64(252).is_none());
        assert!(s.r_i64(256).is_none());
    }

    // r_u8() tests
    #[test]
    fn r_u8_returns_none_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(s.r_u8(0).is_none());
    }

    #[test]
    fn r_u8_returns_some_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_u8(0).is_some());
    }

    #[test]
    fn r_u8_reads_zero_from_unwritten_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(0, s.r_u8(0).unwrap());
    }

    #[test]
    fn r_u8_reads_written_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_u8(0, u8::max_value());
        assert_eq!(u8::max_value(), s.r_u8(0).unwrap());
        s.w_u8(32, u8::max_value());
        assert_eq!(u8::max_value(), s.r_u8(32).unwrap());
    }

    #[test]
    fn r_u8_does_not_read_past_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(4);
        assert!(s.r_u8(3).is_some());
        assert!(s.r_u8(4).is_none());
    }

    #[test]
    fn r_u8_does_not_read_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_u8(255).is_some());
        assert!(s.r_u8(256).is_none());
    }

    // r_u16() tests
    #[test]
    fn r_u16_returns_none_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(s.r_u16(0).is_none());
    }

    #[test]
    fn r_u16_returns_some_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_u16(0).is_some());
    }

    #[test]
    fn r_u16_reads_zero_from_unwritten_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(0, s.r_u16(0).unwrap());
    }

    #[test]
    fn r_u16_reads_written_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_u16(0, u16::max_value());
        assert_eq!(u16::max_value(), s.r_u16(0).unwrap());
        s.w_u16(32, u16::max_value());
        assert_eq!(u16::max_value(), s.r_u16(32).unwrap());
    }

    #[test]
    fn r_u16_does_not_read_past_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(4);
        assert!(s.r_u16(2).is_some());
        assert!(s.r_u16(3).is_none());
        assert!(s.r_u16(4).is_none());
    }

    #[test]
    fn r_u16_does_not_read_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_u16(254).is_some());
        assert!(s.r_u16(255).is_none());
        assert!(s.r_u16(256).is_none());
    }

    // r_u32() tests
    #[test]
    fn r_u32_returns_none_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(s.r_u32(0).is_none());
    }

    #[test]
    fn r_u32_returns_some_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_u32(0).is_some());
    }

    #[test]
    fn r_u32_reads_zero_from_unwritten_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(0, s.r_u32(0).unwrap());
    }

    #[test]
    fn r_u32_reads_written_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_u32(0, u32::max_value());
        assert_eq!(u32::max_value(), s.r_u32(0).unwrap());
        s.w_u32(32, u32::max_value());
        assert_eq!(u32::max_value(), s.r_u32(32).unwrap());
    }

    #[test]
    fn r_u32_does_not_read_past_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(8);
        assert!(s.r_u32(4).is_some());
        assert!(s.r_u32(6).is_none());
        assert!(s.r_u32(8).is_none());
    }

    #[test]
    fn r_u32_does_not_read_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_u32(252).is_some());
        assert!(s.r_u32(254).is_none());
        assert!(s.r_u32(256).is_none());
    }

    // r_i64() tests
    #[test]
    fn r_u64_returns_none_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(s.r_u64(0).is_none());
    }

    #[test]
    fn r_u64_returns_some_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_u64(0).is_some());
    }

    #[test]
    fn r_u64_reads_zero_from_unwritten_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(0, s.r_u64(0).unwrap());
    }

    #[test]
    fn r_u64_reads_written_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_u64(0, u64::max_value());
        assert_eq!(u64::max_value(), s.r_u64(0).unwrap());
        s.w_u64(32, u64::max_value());
        assert_eq!(u64::max_value(), s.r_u64(32).unwrap());
    }

    #[test]
    fn r_u64_does_not_read_past_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(16);
        assert!(s.r_u64(8).is_some());
        assert!(s.r_u64(12).is_none());
        assert!(s.r_u64(16).is_none());
    }

    #[test]
    fn r_u64_does_not_read_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_u64(248).is_some());
        assert!(s.r_u64(252).is_none());
        assert!(s.r_u64(256).is_none());
    }

    // r_f32() tests
    #[test]
    fn r_f32_returns_none_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(s.r_f32(0).is_none());
    }

    #[test]
    fn r_f32_returns_some_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_f32(0).is_some());
    }

    #[test]
    fn r_f32_reads_zero_from_unwritten_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(0.0, s.r_f32(0).unwrap());
    }

    #[test]
    fn r_f32_reads_written_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_f32(0, 12345.6789);
        assert_eq!(12345.6789, s.r_f32(0).unwrap());
        s.w_f32(32, 12345.6789);
        assert_eq!(12345.6789, s.r_f32(32).unwrap());
    }

    #[test]
    fn r_f32_does_not_read_past_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(8);
        assert!(s.r_f32(4).is_some());
        assert!(s.r_f32(6).is_none());
        assert!(s.r_f32(8).is_none());
    }

    #[test]
    fn r_f32_does_not_read_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_f32(252).is_some());
        assert!(s.r_f32(254).is_none());
        assert!(s.r_f32(256).is_none());
    }

    // r_f64() tests
    #[test]
    fn r_f64_returns_none_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(s.r_f64(0).is_none());
    }

    #[test]
    fn r_f64_returns_some_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_f64(0).is_some());
    }

    #[test]
    fn r_f64_reads_zero_from_unwritten_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(0.0, s.r_f64(0).unwrap());
    }

    #[test]
    fn r_f64_reads_written_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_f64(0, 12345.6789);
        assert_eq!(12345.6789, s.r_f64(0).unwrap());
        s.w_f64(32, 12345.6789);
        assert_eq!(12345.6789, s.r_f64(32).unwrap());
    }

    #[test]
    fn r_f64_does_not_read_past_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(16);
        assert!(s.r_f64(8).is_some());
        assert!(s.r_f64(12).is_none());
        assert!(s.r_f64(16).is_none());
    }

    #[test]
    fn r_f64_does_not_read_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_f64(248).is_some());
        assert!(s.r_f64(252).is_none());
        assert!(s.r_f64(256).is_none());
    }

    // r_bool() tests
    #[test]
    fn r_bool_returns_none_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(s.r_bool(0).is_none());
    }

    #[test]
    fn r_bool_returns_some_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_bool(0).is_some());
    }

    #[test]
    fn r_bool_reads_false_from_unwritten_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(false, s.r_bool(0).unwrap());
    }

    #[test]
    fn r_bool_reads_written_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_bool(0, false);
        assert_eq!(false, s.r_bool(0).unwrap());
        s.w_bool(32, true);
        assert_eq!(true, s.r_bool(32).unwrap());
    }

    #[test]
    fn r_bool_does_not_read_past_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(8);
        assert!(s.r_bool(7).is_some());
        assert!(s.r_bool(8).is_none());
    }

    #[test]
    fn r_bool_does_not_read_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_bool(255).is_some());
        assert!(s.r_bool(256).is_none());
    }

    // r_bytes() tests
    #[test]
    fn r_bytes_returns_none_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(s.r_bytes(0, 5).is_none());
    }

    #[test]
    fn r_bytes_returns_some_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_bytes(0, 5).is_some());
    }

    #[test]
    fn r_bytes_reads_zeros_from_unwritten_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(&[0x0, 0x0, 0x0, 0x0, 0x0], s.r_bytes(0, 5).unwrap());
    }

    #[test]
    fn r_bytes_reads_written_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_bytes(0, &[0x0, 0x1, 0x2, 0x3, 0x4]);
        assert_eq!(&[0x0, 0x1, 0x2, 0x3, 0x4], s.r_bytes(0, 5).unwrap());
        s.w_bytes(32, &[0x5, 0x6, 0x7, 0x8, 0x9]);
        assert_eq!(&[0x5, 0x6, 0x7, 0x8, 0x9], s.r_bytes(32, 5).unwrap());
    }

    #[test]
    fn r_bytes_does_not_read_past_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(8);
        assert!(s.r_bytes(6, 2).is_some());
        assert!(s.r_bytes(7, 2).is_none());
        assert!(s.r_bytes(8, 2).is_none());
    }

    #[test]
    fn r_bytes_does_not_read_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_bytes(254, 2).is_some());
        assert!(s.r_bytes(255, 2).is_none());
        assert!(s.r_bytes(256, 2).is_none());
    }

    // r_str() tests
    #[test]
    fn r_str_returns_none_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(s.r_str(0, 5).is_none());
    }

    #[test]
    fn r_str_returns_some_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_str(0, 5).is_some());
    }

    #[test]
    fn r_str_reads_nulls_from_unwritten_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(str::from_utf8(&[0x0, 0x0, 0x0, 0x0, 0x0]).unwrap(), s.r_str(0, 5).unwrap());
    }

    #[test]
    fn r_str_reads_written_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_str(0, "foobar");
        assert_eq!("foobar", s.r_str(0, 6).unwrap());
        s.w_str(32, "I \u{2661} Rust");
        assert_eq!("I \u{2661} Rust", s.r_str(32, 10).unwrap());
    }

    #[test]
    fn r_str_does_not_read_past_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(8);
        assert!(s.r_str(6, 2).is_some());
        assert!(s.r_str(7, 2).is_none());
        assert!(s.r_str(8, 2).is_none());
    }

    #[test]
    fn r_str_does_not_read_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.r_str(254, 2).is_some());
        assert!(s.r_str(255, 2).is_none());
        assert!(s.r_str(256, 2).is_none());
    }

    // fill() tests
    #[test]
    fn fill_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.fill(None, None, 0x1));
    }

    #[test]
    fn fill_does_not_write_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.fill(None, None, 0x1);
        s.open();
        assert!(s.assert_filled(None, None, 0x0));
    }

    #[test]
    fn fill_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.fill(None, None, 0x1));
    }

    #[test]
    fn fill_repeats_byte_in_storage_range() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.fill(Some(10), Some(20), 0x1));
        assert!(s.assert_filled(Some(0), Some(10), 0x0));
        assert!(s.assert_filled(Some(10), Some(20), 0x1));
        assert!(s.assert_filled(Some(20), None, 0x0));
    }

    #[test]
    fn fill_starts_from_beginning_when_start_offset_is_none() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.fill(None, Some(20), 0x1));
        assert!(s.assert_filled(Some(0), Some(20), 0x1));
        assert!(s.assert_filled(Some(20), None, 0x0));
    }

    #[test]
    fn fill_goes_to_end_when_end_offset_is_none() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.fill(Some(10), None, 0x1));
        assert!(s.assert_filled(None, Some(10), 0x0));
        assert!(s.assert_filled(Some(10), None, 0x1));
    }

    #[test]
    fn fill_returns_false_when_end_offset_is_before_start_offset() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(!s.fill(Some(20), Some(10), 0x1));
    }

    #[test]
    fn fill_does_not_write_when_end_offset_is_before_start_offset() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.fill(Some(20), Some(10), 0x1);
        assert!(s.assert_filled(None, None, 0x0));
    }

    #[test]
    fn fill_returns_false_when_before_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(10);
        assert!(!s.fill(Some(9), None, 0x1));
    }

    #[test]
    fn fill_does_not_write_when_before_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(10);
        s.fill(Some(9), None, 0x1);
        assert!(s.assert_filled(None, None, 0x0));
    }

    #[test]
    fn fill_returns_true_when_after_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(10);
        assert!(s.fill(Some(10), None, 0x1));
    }

    #[test]
    fn fill_writes_when_after_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(10);
        s.fill(Some(10), None, 0x1);
        assert!(s.assert_filled(None, Some(10), 0x0));
        assert!(s.assert_filled(Some(10), None, 0x1));
    }

    #[test]
    fn fill_returns_false_when_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(!s.fill(Some(9), Some(257), 0x1));
    }

    #[test]
    fn fill_does_not_write_when_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.fill(Some(9), Some(257), 0x1);
        assert!(s.assert_filled(None, None, 0x0));
    }

    #[test]
    fn fill_does_not_expand_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.fill(Some(9), Some(257), 0x1);
        assert_eq!(256, s.get_capacity());
    }

    // assert_filled() tests
    #[test]
    fn assert_filled_retuns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.assert_filled(None, None, 0x0));
    }

    #[test]
    fn assert_filled_returns_false_when_start_offset_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.assert_filled(Some(255), None, 0x0));
        assert!(!s.assert_filled(Some(256), None, 0x0));
    }

    #[test]
    fn assert_filled_returns_false_when_end_offset_at_or_before_start_offset() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.assert_filled(Some(10), Some(11), 0x0));
        assert!(!s.assert_filled(Some(10), Some(10), 0x0));
        assert!(!s.assert_filled(Some(10), Some(9), 0x0));
    }

    #[test]
    fn assert_filled_returns_false_when_end_offset_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.assert_filled(Some(10), Some(256), 0x0));
        assert!(!s.assert_filled(Some(10), Some(257), 0x0));
    }

    #[test]
    fn assert_filled_checks_whether_all_bytes_in_range_match_value() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.fill(Some(10), Some(20), 0x1);
        assert!(s.assert_filled(None, Some(10), 0x0));
        assert!(!s.assert_filled(None, Some(11), 0x0));
        assert!(s.assert_filled(Some(10), Some(20), 0x1));
        assert!(!s.assert_filled(Some(9), Some(20), 0x1));
        assert!(!s.assert_filled(Some(10), Some(21), 0x1));
        assert!(s.assert_filled(Some(20), None, 0x0));
        assert!(!s.assert_filled(Some(19), None, 0x0));
    }

    #[test]
    fn assert_filled_starts_from_start_offset() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.fill(Some(0), Some(10), 0x1);
        assert!(s.assert_filled(Some(10), None, 0x0));
        assert!(!s.assert_filled(Some(9), None, 0x0));
    }

    #[test]
    fn assert_filled_starts_from_beginning_when_start_offset_is_none() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.fill(Some(1), None, 0x1);
        assert!(s.assert_filled(None, Some(1), 0x0));
        assert!(!s.assert_filled(Some(1), Some(2), 0x0));
    }

    #[test]
    fn assert_filled_goes_to_end_offset() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.fill(Some(250), None, 0x1);
        assert!(s.assert_filled(None, Some(250), 0x0));
        assert!(!s.assert_filled(None, Some(251), 0x0));
    }

    #[test]
    fn assert_filled_goes_to_end_when_end_offset_is_none() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.fill(Some(255), None, 0x1);
        assert!(s.assert_filled(None, Some(255), 0x0));
        assert!(!s.assert_filled(None, None, 0x0));
    }

    // get_use_txn_boundary(), set_use_txn_boundary(), get_txn_boundary(), and set_txn_boundary() tests
    #[test]
    fn get_use_txn_boundary_returns_initialized_value() {
        let s1 = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s1.get_use_txn_boundary());
        let s2 = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        assert!(s2.get_use_txn_boundary());
    }

    #[test]
    fn set_use_txn_boundary_changes_value() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.set_use_txn_boundary(true);
        assert!(s.get_use_txn_boundary());
        s.set_use_txn_boundary(false);
        assert!(!s.get_use_txn_boundary());
    }

    #[test]
    fn set_use_txn_boundary_resets_boundary_to_zero_when_false() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(10);
        assert_eq!(10, s.get_txn_boundary());
        s.set_use_txn_boundary(false);
        assert_eq!(0, s.get_txn_boundary());
        s.set_use_txn_boundary(true);
        assert_eq!(0, s.get_txn_boundary());
    }

    #[test]
    fn get_txn_boundary_starts_at_0_whether_used_or_not() {
        let s1 = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert_eq!(0, s1.get_txn_boundary());
        let s2 = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        assert_eq!(0, s2.get_txn_boundary());
    }

    #[test]
    fn set_txn_boundary_returns_false_when_not_using_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(!s.set_txn_boundary(10));
    }

    #[test]
    fn set_txn_boundary_does_not_change_boundary_when_not_using_txn_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(10);
        assert_eq!(0, s.get_txn_boundary());
    }

    #[test]
    fn set_txn_boundary_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        assert!(!s.set_txn_boundary(10));
    }

    #[test]
    fn set_txn_boundary_does_not_change_boundary_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.set_txn_boundary(10);
        s.open();
        assert_eq!(0, s.get_txn_boundary());
    }

    #[test]
    fn set_txn_boundary_returns_false_when_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        assert!(!s.set_txn_boundary(257));
        assert!(s.set_txn_boundary(256));
    }

    #[test]
    fn set_txn_boundary_does_not_change_boundary_when_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(257);
        assert_eq!(0, s.get_txn_boundary());
    }

    #[test]
    fn set_txn_boundary_does_not_expand_capacity_when_past_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        assert_eq!(256, s.get_capacity());
        s.set_txn_boundary(257);
        assert_eq!(256, s.get_capacity());
    }

    #[test]
    fn set_txn_boundary_changes_boundary() {
        let mut s = MemoryBinaryStorage::new(256, 256, true, 256, 4096).unwrap();
        s.open();
        s.set_txn_boundary(50);
        assert_eq!(50, s.get_txn_boundary());
        s.set_txn_boundary(25);
        assert_eq!(25, s.get_txn_boundary());
        s.set_txn_boundary(200);
        assert_eq!(200, s.get_txn_boundary());
    }

    // get_expand_size() and set_expand_size() tests
    #[test]
    fn get_expand_size_returns_initial_expand_size() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        assert_eq!(512, s.get_expand_size());
    }

    #[test]
    fn set_expand_size_returns_false_when_expand_size_is_zero() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        assert!(!s.set_expand_size(0));
    }

    #[test]
    fn set_expand_size_does_not_change_expand_size_when_expand_size_is_zero() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.set_expand_size(0);
        assert_eq!(512, s.get_expand_size());
    }

    #[test]
    fn set_expand_size_returns_false_when_expand_size_is_not_power_of_2() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        assert!(!s.set_expand_size(513));
    }

    #[test]
    fn set_expand_size_does_not_change_expand_size_when_expand_size_is_not_power_of_2() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.set_expand_size(513);
        assert_eq!(512, s.get_expand_size());
    }

    #[test]
    fn set_expand_size_returns_true_when_checks_pass() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        assert!(s.set_expand_size(1024));
    }

    #[test]
    fn set_expand_size_changes_expand_size_when_checks_pass() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.set_expand_size(1024);
        assert_eq!(1024, s.get_expand_size());
    }

    #[test]
    fn capacity_increases_to_increments_of_last_set_expand_size() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.open();
        s.w_u8(256, 0x1);
        assert_eq!(512, s.get_capacity());
        s.set_expand_size(8);
        s.w_u8(512, 0x1);
        assert_eq!(520, s.get_capacity());
    }

    // get_align() and set_align() tests
    #[test]
    fn get_align_returns_initial_align() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        assert_eq!(1024, s.get_align());
    }

    #[test]
    fn set_align_returns_false_when_align_is_zero() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        assert!(!s.set_align(0));
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
        assert!(!s.set_align(1025));
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
        assert!(s.set_align(2048));
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
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        assert_eq!(0, s.get_capacity());
        s.open();
        s.close();
        assert_eq!(0, s.get_capacity());
    }

    #[test]
    fn get_capacity_returns_initial_capacity_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.open();
        assert_eq!(256, s.get_capacity());
    }

    #[test]
    fn get_capacity_returns_new_capacity_after_expansion() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.open();
        s.w_u8(256, 0x1);
        assert_eq!(512, s.get_capacity());
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
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        assert!(!s.expand(10000));
    }

    #[test]
    fn expand_does_not_change_capacity_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.expand(10000);
        s.open();
        assert_eq!(256, s.get_capacity());
    }

    #[test]
    fn expand_returns_true_when_already_has_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.open();
        s.set_expand_size(16);
        assert!(s.expand(50));
    }

    #[test]
    fn expand_does_not_change_capacity_when_already_has_capacity() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.open();
        s.set_expand_size(16);
        s.expand(50);
        assert_eq!(256, s.get_capacity());
    }

    #[test]
    fn expand_returns_false_when_allocation_arithmetic_overflows() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.open();
        assert!(!s.expand(usize::max_value()));
    }

    #[test]
    fn expand_does_not_change_capacity_when_allocation_arithmetic_overflows() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.open();
        s.expand(usize::max_value());
        assert_eq!(256, s.get_capacity());
    }

    #[test]
    fn expand_returns_false_when_allocation_fails() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.open();
        assert!(!s.expand((usize::max_value() - 1024) as usize));
    }

    #[test]
    fn expand_does_not_change_capacity_when_allocation_fails() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.open();
        s.expand((usize::max_value() - 1024) as usize);
        assert_eq!(256, s.get_capacity());
    }

    #[test]
    fn expand_returns_true_when_successful() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.open();
        assert!(s.expand(300));
    }

    #[test]
    fn expand_changes_capacity_by_expand_size_when_successful() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.open();
        s.expand(300);
        assert_eq!(512, s.get_capacity());
    }

    #[test]
    fn expand_changes_capacity_by_multiples_of_expand_size_when_successful() {
        let mut s = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        s.open();
        s.expand(3000);
        assert_eq!(3072, s.get_capacity());
    }


}
