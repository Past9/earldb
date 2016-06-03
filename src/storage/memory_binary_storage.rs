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

        if !MemoryBinaryStorage::check_initial_mem_params(
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

        unsafe { Some(ptr::read(self.ptr(offset))) }
    }

    fn is_power_of_two(n: usize) -> bool {
        return (n != 0) && (n & (n - 1)) == 0;
    }

    fn check_initial_mem_params(
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
        if self.use_txn_boundary && (offset + len) >= self.txn_boundary { return None }

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
        if end_offset >= self.capacity { return false }

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

        match self.r_bytes(start_offset, end_offset - start_offset) {
            Some(d) => {
                for b in d {
                    if *b != val { return false }
                }
                true
            },
            None => false
        }
    }


    fn get_use_txn_boundary(&self) -> bool {
        self.use_txn_boundary
    }

    fn set_use_txn_boundary(&mut self, val: bool) {
        self.use_txn_boundary = val;
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

    fn set_expand_size(&mut self, expand_size: usize) {
        // TODO: check memory params
        unimplemented!();
    }

    fn get_align(&self) -> usize {
        self.align
    }

    fn set_align(&mut self, align: usize) {
        // TODO: check memory params
        unimplemented!();
    }


    fn expand(&mut self, min_capacity: usize) -> bool {
        // Determine the new size of the journal in multiples of expand_size
        let expand_increments = (min_capacity as f32 / self.expand_size as f32).ceil() as usize;
        let new_capacity = match expand_increments.checked_mul(self.expand_size) {
            Some(x) => x,
            None => return false
        };

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

    use std::mem;

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

    /*
    #[test]
    fn w_bool_over_capacity_expands_storage() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_f64(256, 12345.6789));
        assert_eq!(512, s.get_capacity());
        assert_eq!(12345.6789, s.r_f64(256).unwrap());
    }
    */

    /*
    #[test]
    fn capacity_returns_0_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert_eq!(0, s.get_capacity());
    }

    #[test]
    fn get_expand_size_returns_expand_size_when_open_or_closed() {
        let mut s = MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap();
        assert_eq!(512, s.get_expand_size());
        s.open();
        assert_eq!(512, s.get_expand_size());
    }

    #[test]
    fn get_use_txn_boundary_returns_value_when_open_or_closed() {
        let mut s1 = MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap();
        assert!(!s1.get_use_txn_boundary());
        s1.open();
        assert!(!s1.get_use_txn_boundary());
        let mut s2 = MemoryBinaryStorage::new(256, 512, true, 1024, 4096).unwrap();
        assert!(s2.get_use_txn_boundary());
        s2.open();
        assert!(s2.get_use_txn_boundary());
    }

    #[test]
    fn get_align_returns_align_when_open_or_closed() {
        let mut s = MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap();
        assert_eq!(1024, s.get_align());
        s.open();
        assert_eq!(1024, s.get_align());
    }

    #[test]
    fn get_txn_boundary_returns_0_when_new_and_closed() {
        let mut s = MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap();
        assert_eq!(0, s.get_txn_boundary());
    }

    #[test]
    fn get_txn_boundary_returns_0_when_new_and_open() {
        let mut s = MemoryBinaryStorage::new(256, 512, false, 1024, 4096).unwrap();
        s.open();
        assert_eq!(0, s.get_txn_boundary());
    }


    #[test]
    fn w_i16_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_i16(0, 8));
    }

    #[test]
    fn w_i32_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_i32(0, 8));
    }

    #[test]
    fn w_i64_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_i64(0, 8));
    }

    #[test]
    fn w_u8_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_u8(0, 8));
    }

    #[test]
    fn w_u16_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_u16(0, 8));
    }

    #[test]
    fn w_u32_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_u32(0, 8));
    }

    #[test]
    fn w_u64_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_u64(0, 8));
    }

    #[test]
    fn w_f32_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_f32(0, 0.8));
    }

    #[test]
    fn w_f64_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_f64(0, 0.8));
    }

    #[test]
    fn w_bool_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_bool(0, true));
    }

    #[test]
    fn w_bytes_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_bytes(0, &[0, 1, 2, 3]));
    }

    #[test]
    fn w_str_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert!(!s.w_str(0, "foo"));
    }

    #[test]
    fn r_i8_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert_eq!(None, s.r_i8(0));
    }

    #[test]
    fn r_i16_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert_eq!(None, s.r_i16(0));
    }

    #[test]
    fn r_i32_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert_eq!(None, s.r_i32(0));
    }

    #[test]
    fn r_i64_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert_eq!(None, s.r_i64(0));
    }

    #[test]
    fn r_u8_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert_eq!(None, s.r_u8(0));
    }

    #[test]
    fn r_u16_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert_eq!(None, s.r_u16(0));
    }

    #[test]
    fn r_u32_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert_eq!(None, s.r_u32(0));
    }

    #[test]
    fn r_u64_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert_eq!(None, s.r_u64(0));
    }

    #[test]
    fn r_f32_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert_eq!(None, s.r_f32(0));
    }

    #[test]
    fn r_f64_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert_eq!(None, s.r_f64(0));
    }

    #[test]
    fn r_bool_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert_eq!(None, s.r_bool(0));
    }

    #[test]
    fn r_bytes_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert_eq!(None, s.r_bytes(0, 8));
    }

    #[test]
    fn r_str_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        assert_eq!(None, s.r_str(0, 8));
    }

    #[test]
    fn w_i16_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_i16(0, 8));
    }

    #[test]
    fn w_i32_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_i32(0, 8));
    }

    #[test]
    fn w_i64_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_i64(0, 8));
    }

    #[test]
    fn w_u8_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_u8(0, 8));
    }

    #[test]
    fn w_u16_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_u16(0, 8));
    }

    #[test]
    fn w_u32_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_u32(0, 8));
    }

    #[test]
    fn w_u64_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_u64(0, 8));
    }

    #[test]
    fn w_f32_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_f32(0, 0.8));
    }

    #[test]
    fn w_f64_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_f64(0, 0.8));
    }

    #[test]
    fn w_bool_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_bool(0, true));
    }

    #[test]
    fn w_bytes_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_bytes(0, &[0, 1, 2, 3]));
    }

    #[test]
    fn w_str_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        assert!(s.w_str(0, "foo"));
    }

    #[test]
    fn r_i8_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_i8(0, i8::max_value());
        assert_eq!(i8::max_value(), s.r_i8(0).unwrap());
        s.w_i8(mem::size_of::<i8>(), i8::min_value());
        assert_eq!(i8::min_value(), s.r_i8(mem::size_of::<i8>()).unwrap());
    }

    #[test]
    fn r_i16_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_i16(0, i16::max_value());
        assert_eq!(i16::max_value(), s.r_i16(0).unwrap());
        s.w_i16(mem::size_of::<i16>(), i16::min_value());
        assert_eq!(i16::min_value(), s.r_i16(mem::size_of::<i16>()).unwrap());
    }

    #[test]
    fn r_i32_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_i32(0, i32::max_value());
        assert_eq!(i32::max_value(), s.r_i32(0).unwrap());
        s.w_i32(mem::size_of::<i32>(), i32::min_value());
        assert_eq!(i32::min_value(), s.r_i32(mem::size_of::<i32>()).unwrap());
    }

    #[test]
    fn r_i64_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_i64(0, i64::max_value());
        assert_eq!(i64::max_value(), s.r_i64(0).unwrap());
        s.w_i64(mem::size_of::<i64>(), i64::min_value());
        assert_eq!(i64::min_value(), s.r_i64(mem::size_of::<i64>()).unwrap());
    }

    #[test]
    fn r_u8_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_u8(0, u8::max_value());
        assert_eq!(u8::max_value(), s.r_u8(0).unwrap());
        s.w_u8(mem::size_of::<u8>(), u8::min_value());
        assert_eq!(u8::min_value(), s.r_u8(mem::size_of::<u8>()).unwrap());
    }

    #[test]
    fn r_u16_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_u16(0, u16::max_value());
        assert_eq!(u16::max_value(), s.r_u16(0).unwrap());
        s.w_u16(mem::size_of::<u16>(), u16::min_value());
        assert_eq!(u16::min_value(), s.r_u16(mem::size_of::<u16>()).unwrap());
    }

    #[test]
    fn r_u32_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_u32(0, u32::max_value());
        assert_eq!(u32::max_value(), s.r_u32(0).unwrap());
        s.w_u32(mem::size_of::<u32>(), u32::min_value());
        assert_eq!(u32::min_value(), s.r_u32(mem::size_of::<u32>()).unwrap());
    }

    #[test]
    fn r_u64_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_u64(0, u64::max_value());
        assert_eq!(u64::max_value(), s.r_u64(0).unwrap());
        s.w_u64(mem::size_of::<u64>(), u64::min_value());
        assert_eq!(u64::min_value(), s.r_u64(mem::size_of::<u64>()).unwrap());
    }

    #[test]
    fn r_f32_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_f32(0, 12345.6789);
        assert_eq!(12345.6789, s.r_f32(0).unwrap());
        s.w_f32(mem::size_of::<f32>(), -0.0004321);
        assert_eq!(-0.0004321, s.r_f32(mem::size_of::<f32>()).unwrap());
    }

    #[test]
    fn r_f64_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_f64(0, 12345.6789);
        assert_eq!(12345.6789, s.r_f64(0).unwrap());
        s.w_f64(mem::size_of::<f64>(), -0.0004321);
        assert_eq!(-0.0004321, s.r_f64(mem::size_of::<f64>()).unwrap());
    }

    #[test]
    fn r_bool_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_bool(0, true);
        assert_eq!(true, s.r_bool(0).unwrap());
        s.w_bool(mem::size_of::<bool>(), false);
        assert_eq!(false, s.r_bool(mem::size_of::<bool>()).unwrap());
    }

    #[test]
    fn r_bytes_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_bytes(0, &[0x0, 0x1, 0x2, 0x3]);
        assert_eq!(&[0x0, 0x1, 0x2, 0x3], s.r_bytes(0, 4).unwrap());
        s.w_bytes(4, &[0x4, 0x5, 0x6, 0x7]);
        assert_eq!(&[0x4, 0x5, 0x6, 0x7], s.r_bytes(4, 4).unwrap());
    }

    #[test]
    fn r_str_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256, 4096).unwrap();
        s.open();
        s.w_str(0, "foobar");
        assert_eq!("foobar", s.r_str(0, 6).unwrap());
        s.w_str(6, "bazquux");
        assert_eq!("bazquux", s.r_str(6, 7).unwrap());
    }

    */


    



}
