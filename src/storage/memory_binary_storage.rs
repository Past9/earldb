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
  capacity: u64,
  expand_size: u64,
  align: usize
}
impl MemoryBinaryStorage {

  pub fn new(
    initial_capacity: u64, 
    expand_size: u64 
  ) -> Result<MemoryBinaryStorage, Error> {

    try!(MemoryBinaryStorage::check_params(
      expand_size,
      initial_capacity
    )); 

    let align = mem::size_of::<usize>();

    let c_capacity = try!(util::u64_as_usize(initial_capacity));

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
      align: align
    })

  }

  fn ptr<T>(&self, offset: usize) -> *const T {
    (self.origin as usize + offset) as *const T
  }

  fn ptr_mut<T>(&mut self, offset: usize) -> *mut T {
    (self.origin as usize + offset) as *mut T
  }

  fn write<T>(&mut self, offset: u64, data: T) -> Result<(), Error> {
    try!(AssertionError::assert(
      self.is_open, 
      binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED
    ));

    let c_offset = try!(util::u64_as_usize(offset));
    let end_offset = try!(util::usize_add(c_offset, mem::size_of::<T>()));
    try!(util::usize_add(self.origin as usize, end_offset));

    try!(self.expand(end_offset as u64));
    unsafe { ptr::write(self.ptr_mut(c_offset), data) }
    Ok(())
  }

  fn read<T: Copy>(&self, offset: u64) -> Result<T, Error> {
    try!(AssertionError::assert(
      self.is_open, 
      binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED
    ));

    let c_capacity = try!(util::u64_as_usize(self.capacity));
    let c_offset = try!(util::u64_as_usize(offset));
    let end_offset = try!(util::usize_add(c_offset, mem::size_of::<T>()));
    try!(util::usize_add(self.origin as usize, end_offset));

    try!(AssertionError::assert_not(
      end_offset > c_capacity, 
      binary_storage::ERR_READ_PAST_END
    ));

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
    try!(AssertionError::assert_not(
      self.is_open, 
      binary_storage::ERR_OPERATION_INVALID_WHEN_OPEN
    ));
    self.is_open = true;
    Ok(())
  }

  fn close(&mut self) -> Result<(), Error> {
    try!(AssertionError::assert(
      self.is_open, 
      binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED
    ));
    self.is_open = false;
    Ok(())
  }

  fn w_i8(
    &mut self, 
    offset: u64, 
    data: i8
  ) -> Result<(), Error> { self.write(offset, data) }

  fn w_i16(
    &mut self, 
    offset: u64, 
    data: i16
  ) -> Result<(), Error> { self.write(offset, data) }

  fn w_i32(
    &mut self, 
    offset: u64, 
    data: i32
  ) -> Result<(), Error> { self.write(offset, data) }

  fn w_i64(
    &mut self, 
    offset: u64, 
    data: i64
  ) -> Result<(), Error> { self.write(offset, data) }

  fn w_u8(
    &mut self, 
    offset: u64, 
    data: u8
  ) -> Result<(), Error> { self.write(offset, data) }

  fn w_u16(
    &mut self, 
    offset: u64, 
    data: u16
  ) -> Result<(), Error> { self.write(offset, data) }

  fn w_u32(
    &mut self, 
    offset: u64, 
    data: u32
  ) -> Result<(), Error> { self.write(offset, data) }

  fn w_u64(
    &mut self, 
    offset: u64, 
    data: u64
  ) -> Result<(), Error> { self.write(offset, data) }

  fn w_f32(
    &mut self, 
    offset: u64, 
    data: f32
  ) -> Result<(), Error> { self.write(offset, data) }

  fn w_f64(
    &mut self, 
    offset: u64, 
    data: f64
  ) -> Result<(), Error> { self.write(offset, data) }

  fn w_bool(
    &mut self, 
    offset: u64, 
    data: bool
  ) -> Result<(), Error> { self.write(offset, data) }

  fn w_bytes(&mut self, offset: u64, data: &[u8]) -> Result<(), Error> {
    try!(AssertionError::assert(
      self.is_open, 
      binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED
    ));

    let c_offset = try!(util::u64_as_usize(offset));
    let end_offset = try!(util::usize_add(c_offset, data.len()));
    try!(util::usize_add(self.origin as usize, end_offset));

    try!(self.expand(end_offset as u64));

    let dest = unsafe { 
      slice::from_raw_parts_mut(self.ptr_mut(c_offset), data.len()) 
    };
    dest.clone_from_slice(data);
    Ok(())
  }

  fn w_str(&mut self, offset: u64, data: &str) -> Result<(), Error> { 
    self.w_bytes(offset, data.as_bytes()) 
  }


  fn r_i8(&self, offset: u64) -> Result<i8, Error> { self.read(offset) }
  fn r_i16(&self, offset: u64) -> Result<i16, Error> { self.read(offset) }
  fn r_i32(&self, offset: u64) -> Result<i32, Error> { self.read(offset) }
  fn r_i64(&self, offset: u64) -> Result<i64, Error> { self.read(offset) }

  fn r_u8(&self, offset: u64) -> Result<u8, Error> { self.read(offset) }
  fn r_u16(&self, offset: u64) -> Result<u16, Error> { self.read(offset) }
  fn r_u32(&self, offset: u64) -> Result<u32, Error> { self.read(offset) }
  fn r_u64(&self, offset: u64) -> Result<u64, Error> { self.read(offset) }

  fn r_f32(&self, offset: u64) -> Result<f32, Error> { self.read(offset) }
  fn r_f64(&self, offset: u64) -> Result<f64, Error> { self.read(offset) }

  fn r_bool(&self, offset: u64) -> Result<bool, Error> { self.read(offset) }

  fn r_bytes(&self, offset: u64, len: usize) -> Result<Vec<u8>, Error> {
    try!(AssertionError::assert(
      self.is_open, 
      binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED
    ));

    let c_capacity = try!(util::u64_as_usize(self.capacity));
    let c_offset = try!(util::u64_as_usize(offset));
    let end_offset = try!(util::usize_add(c_offset, len));
    try!(util::usize_add(self.origin as usize, end_offset));

    try!(AssertionError::assert_not(
      end_offset > c_capacity, 
      binary_storage::ERR_READ_PAST_END
    ));

    let src = unsafe { slice::from_raw_parts::<u8>(self.ptr(c_offset), len) };
    let mut dst = vec![0; len];
    dst.copy_from_slice(src);
    Ok(dst)
  }

  fn r_str(&self, offset: u64, len: usize) -> Result<String, Error> {
    let b = try!(self.r_bytes(offset, len));
    Ok(try!(str::from_utf8(b.as_slice())).to_string())
  }


  fn fill(
    &mut self, 
    start: Option<u64>, 
    end: Option<u64>, 
    val: u8
  ) -> Result<(), Error> {
    try!(AssertionError::assert(
      self.is_open, 
      binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED
    ));

    let start_offset = try!(util::u64_as_usize(
      match start { Some(s) => s, None => 0 }
    ));
    let end_offset = try!(util::u64_as_usize(
      match end { Some(e) => e, None => self.capacity }
    ));

    let c_capacity = try!(util::u64_as_usize(self.capacity));

    try!(AssertionError::assert(
      start_offset < c_capacity, 
      binary_storage::ERR_WRITE_PAST_END
    ));

    try!(AssertionError::assert(
      end_offset <= c_capacity, 
      binary_storage::ERR_WRITE_PAST_END
    ));

    try!(AssertionError::assert(
      end_offset > start_offset,
      binary_storage::ERR_WRITE_NOTHING
    ));
    
    unsafe { 
      ptr::write_bytes::<u8>(
        self.ptr_mut(start_offset), 
        val, 
        end_offset - start_offset
      ) 
    }
    Ok(())
  }

  fn is_filled(
    &self, 
    start: Option<u64>, 
    end: Option<u64>, 
    val: u8
  ) -> Result<bool, Error> {
    try!(AssertionError::assert(
      self.is_open, 
      binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED
    ));

    let start_offset = try!(util::u64_as_usize(
      match start { Some(s) => s, None => 0 }
    ));
    let end_offset = try!(util::u64_as_usize(
      match end { Some(e) => e, None => self.capacity }
    ));

    let c_capacity = try!(util::u64_as_usize(self.capacity));

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
    try!(AssertionError::assert(
      self.is_open, 
      binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED
    ));

    // Determine the new size of the journal in multiples of expand_size
    let expand_increments = (
      min_capacity as f64 / self.expand_size as f64
    ).ceil() as u64;
    let new_capacity = match expand_increments.checked_mul(self.expand_size) {
      Some(x) => x,
      None => return Err(Error::Assertion(
        AssertionError::new(binary_storage::ERR_ARITHMETIC_OVERFLOW)
      ))
    };

    let c_capacity = try!(util::u64_as_usize(self.capacity));
    let c_new_capacity = try!(util::u64_as_usize(new_capacity));

    // We don't want to reallocate (or even reduce the capacity) if we 
    // already have enough, so just do nothing and return Ok if we 
    // already have enough room.
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
      return Err(
        Error::Assertion(AssertionError::new(binary_storage::ERR_STORAGE_ALLOC))
      );
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
    try!(AssertionError::assert(
      self.is_open, 
      binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED
    ));
    Ok(self.capacity)
  }

  fn is_open(&self) -> bool {
    self.is_open
  }


}

