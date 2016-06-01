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
    align: usize
}
impl MemoryBinaryStorage {

    pub fn new(
        initial_capacity: usize, 
        expand_size: usize, 
        use_txn_boundary: bool,
        align: usize
    ) -> MemoryBinaryStorage {

        let origin = unsafe { heap::allocate(initial_capacity, align) as *mut u8 };
        unsafe { ptr::write_bytes::<u8>(origin, 0x2, initial_capacity) };

        MemoryBinaryStorage {
            origin: origin as *const u8,
            is_open: false,
            capacity: initial_capacity,
            expand_size: expand_size,
            use_txn_boundary: use_txn_boundary,
            txn_boundary: 0,
            align: align
        }

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
        if self.use_txn_boundary && (offset + mem::size_of::<T>()) >= self.txn_boundary { return None }

        let slice = unsafe {
            slice::from_raw_parts::<u8>(self.ptr(offset), 10)
        };

        unsafe { Some(ptr::read(self.ptr(offset))) }
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

    fn w_i8(&mut self, offset: usize, data: i8) -> bool {
        self.write(offset, data)
    }

    fn w_i16(&mut self, offset: usize, data: i16) -> bool {
        self.write(offset, data)
    }

    fn w_i32(&mut self, offset: usize, data: i32) -> bool {
        self.write(offset, data)
    }

    fn w_i64(&mut self, offset: usize, data: i64) -> bool {
        self.write(offset, data)
    }


    fn w_u8(&mut self, offset: usize, data: u8) -> bool {
        self.write(offset, data)
    }

    fn w_u16(&mut self, offset: usize, data: u16) -> bool {
        self.write(offset, data)
    }

    fn w_u32(&mut self, offset: usize, data: u32) -> bool {
        self.write(offset, data)
    }

    fn w_u64(&mut self, offset: usize, data: u64) -> bool {
        self.write(offset, data)
    }


    fn w_f32(&mut self, offset: usize, data: f32) -> bool {
        self.write(offset, data)
    }

    fn w_f64(&mut self, offset: usize, data: f64) -> bool {
        self.write(offset, data)
    }


    fn w_bool(&mut self, offset: usize, data: bool) -> bool {
        self.write(offset, data)
    }


    fn w_bytes(&mut self, offset: usize, data: &[u8]) -> bool {
        if !self.is_open { return false }
        if self.use_txn_boundary && offset < self.txn_boundary { return false }
        if !self.expand(offset + data.len()) { return false }

        unsafe { ptr::write(self.ptr_mut(offset), data) }
        true
    }

    fn w_str(&mut self, offset: usize, data: &str) -> bool {
        self.w_bytes(offset, data.as_bytes())
    }



    fn r_i8(&self, offset: usize) -> Option<i8> {
        self.read(offset)
    }

    fn r_i16(&self, offset: usize) -> Option<i16> {
        self.read(offset)
    }

    fn r_i32(&self, offset: usize) -> Option<i32> {
        self.read(offset)
    }

    fn r_i64(&self, offset: usize) -> Option<i64> {
        self.read(offset)
    }


    fn r_u8(&self, offset: usize) -> Option<u8> {
        self.read(offset)
    }

    fn r_u16(&self, offset: usize) -> Option<u16> {
        self.read(offset)
    }

    fn r_u32(&self, offset: usize) -> Option<u32> {
        self.read(offset)
    }

    fn r_u64(&self, offset: usize) -> Option<u64> {
        self.read(offset)
    }


    fn r_f32(&self, offset: usize) -> Option<f32> {
        self.read(offset)
    }

    fn r_f64(&self, offset: usize) -> Option<f64> {
        self.read(offset)
    }


    fn r_bool(&self, offset: usize) -> Option<bool> {
        self.read(offset)
    }


    fn r_bytes(&self, offset: usize, len: usize) -> Option<&[u8]> {
        if !self.is_open { return None }
        if self.use_txn_boundary && (offset + len) >= self.txn_boundary { return None }

        unsafe { Some(ptr::read(self.ptr(offset))) }
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
        unimplemented!();
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

    fn set_expand_size(&mut self) -> usize {
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

    fn capacity(&self) -> usize {
        if !self.is_open { return 0 }
        self.capacity
    }


    fn is_open(&self) -> bool {
        self.is_open
    }


}


#[cfg(test)]
mod tests {

    use std::u8;

    use storage::binary_storage::BinaryStorage;
    use storage::memory_binary_storage::MemoryBinaryStorage;


    #[test]
    fn is_closed_when_instantiated() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert!(!s.is_open());
    }

    #[test]
    fn is_open_after_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        assert!(s.is_open());
    }

    #[test]
    fn is_closed_after_open_and_close() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        s.close();
        assert!(!s.is_open());
    }

    #[test]
    fn w_i8_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert!(!s.w_i8(0, 8));
    }

    #[test]
    fn w_i16_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert!(!s.w_i16(0, 8));
    }

    #[test]
    fn w_i32_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert!(!s.w_i32(0, 8));
    }

    #[test]
    fn w_i64_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert!(!s.w_i64(0, 8));
    }

    #[test]
    fn w_u8_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert!(!s.w_u8(0, 8));
    }

    #[test]
    fn w_u16_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert!(!s.w_u16(0, 8));
    }

    #[test]
    fn w_u32_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert!(!s.w_u32(0, 8));
    }

    #[test]
    fn w_u64_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert!(!s.w_u64(0, 8));
    }

    #[test]
    fn w_f32_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert!(!s.w_f32(0, 0.8));
    }

    #[test]
    fn w_f64_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert!(!s.w_f64(0, 0.8));
    }

    #[test]
    fn w_bool_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert!(!s.w_bool(0, true));
    }

    #[test]
    fn w_bytes_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert!(!s.w_bytes(0, &[0, 1, 2, 3]));
    }

    #[test]
    fn w_str_returns_false_when_closed() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert!(!s.w_str(0, "foo"));
    }

    #[test]
    fn r_i8_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert_eq!(None, s.r_i8(0));
    }

    #[test]
    fn r_i16_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert_eq!(None, s.r_i16(0));
    }

    #[test]
    fn r_i32_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert_eq!(None, s.r_i32(0));
    }

    #[test]
    fn r_i64_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert_eq!(None, s.r_i64(0));
    }

    #[test]
    fn r_u8_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert_eq!(None, s.r_u8(0));
    }

    #[test]
    fn r_u16_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert_eq!(None, s.r_u16(0));
    }

    #[test]
    fn r_u32_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert_eq!(None, s.r_u32(0));
    }

    #[test]
    fn r_u64_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert_eq!(None, s.r_u64(0));
    }

    #[test]
    fn r_f32_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert_eq!(None, s.r_f32(0));
    }

    #[test]
    fn r_f64_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert_eq!(None, s.r_f64(0));
    }

    #[test]
    fn r_bool_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert_eq!(None, s.r_bool(0));
    }

    #[test]
    fn r_bytes_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert_eq!(None, s.r_bytes(0, 8));
    }

    #[test]
    fn r_str_returns_none_when_closed() {
        let s = MemoryBinaryStorage::new(256, 256, false, 256);
        assert_eq!(None, s.r_str(0, 8));
    }

    #[test]
    fn w_i8_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        assert!(s.w_i8(0, 8));
    }

    #[test]
    fn w_i16_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        assert!(s.w_i16(0, 8));
    }

    #[test]
    fn w_i32_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        assert!(s.w_i32(0, 8));
    }

    #[test]
    fn w_i64_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        assert!(s.w_i64(0, 8));
    }

    #[test]
    fn w_u8_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        assert!(s.w_u8(0, 8));
    }

    #[test]
    fn w_u16_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        assert!(s.w_u16(0, 8));
    }

    #[test]
    fn w_u32_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        assert!(s.w_u32(0, 8));
    }

    #[test]
    fn w_u64_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        assert!(s.w_u64(0, 8));
    }

    #[test]
    fn w_f32_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        assert!(s.w_f32(0, 0.8));
    }

    #[test]
    fn w_f64_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        assert!(s.w_f64(0, 0.8));
    }

    #[test]
    fn w_bool_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        assert!(s.w_bool(0, true));
    }

    #[test]
    fn w_bytes_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        assert!(s.w_bytes(0, &[0, 1, 2, 3]));
    }

    #[test]
    fn w_str_returns_true_when_open() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        assert!(s.w_str(0, "foo"));
    }

    #[test]
    fn r_i8_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        s.w_i8(0, i8::max_value());
        assert_eq!(i8::max_value(), s.r_i8(0).unwrap());
    }

    #[test]
    fn r_i16_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        s.w_i16(0, i16::max_value());
        assert_eq!(i16::max_value(), s.r_i16(0).unwrap());
    }

    #[test]
    fn r_i32_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        s.w_i32(0, i32::max_value());
        assert_eq!(i32::max_value(), s.r_i32(0).unwrap());
    }

    #[test]
    fn r_i64_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        s.w_i64(0, i64::max_value());
        assert_eq!(i64::max_value(), s.r_i64(0).unwrap());
    }

    #[test]
    fn r_u8_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        s.w_u8(0, u8::max_value());
        assert_eq!(u8::max_value(), s.r_u8(0).unwrap());
    }

    #[test]
    fn r_u16_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        s.w_u16(0, u16::max_value());
        assert_eq!(u16::max_value(), s.r_u16(0).unwrap());
    }

    #[test]
    fn r_u32_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        s.w_u32(0, u32::max_value());
        assert_eq!(u32::max_value(), s.r_u32(0).unwrap());
    }

    #[test]
    fn r_u64_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        s.w_u64(0, u64::max_value());
        assert_eq!(u64::max_value(), s.r_u64(0).unwrap());
    }

    #[test]
    fn r_f32_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        s.w_f32(0, 12345.6789);
        assert_eq!(12345.6789, s.r_f32(0).unwrap());
    }

    #[test]
    fn r_f64_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        s.w_f64(0, 12345.6789);
        assert_eq!(12345.6789, s.r_f64(0).unwrap());
    }

    #[test]
    fn r_bool_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        s.w_bool(0, true);
        assert_eq!(true, s.r_bool(0).unwrap());
        s.w_bool(0, false);
        assert_eq!(false, s.r_bool(0).unwrap());
    }

    #[test]
    fn r_bytes_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        s.w_bytes(0, &[0x0, 0x1, 0x2, 0x3]);
        assert_eq!(&[0x0, 0x1, 0x2, 0x3], s.r_bytes(0, 4).unwrap());
    }

    #[test]
    fn r_str_reads_data() {
        let mut s = MemoryBinaryStorage::new(256, 256, false, 256);
        s.open();
        s.w_str(0, "foobar");
        assert_eq!("foobar", s.r_str(0, 6).unwrap());
    }




}
