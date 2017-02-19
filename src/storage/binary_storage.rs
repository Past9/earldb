use error::{ Error };

pub static ERR_STORAGE_ALLOC: &'static str = 
  "Storage allocation failed";
pub static ERR_ARITHMETIC_OVERFLOW: &'static str = 
  "Operation failed due to arithmetic overflow";
pub static ERR_EXPAND_SIZE_TOO_SMALL: &'static str = 
  "Expansion size must be greater that zero";
pub static ERR_INITIAL_CAP_TOO_SMALL: &'static str = 
  "Initial capacity must be greater than zero";
pub static ERR_INITIAL_CAP_NOT_POWER_OF_2: &'static str = 
  "Initial capacity must be a power of 2";
pub static ERR_EXPAND_SIZE_NOT_POWER_OF_2: &'static str = 
  "Expansion size must be a power of 2";
pub static ERR_WRITE_PAST_END: & 'static str = 
  "Cannot write past end of allocated storage";
pub static ERR_READ_PAST_END: & 'static str = 
  "Cannot read past end of allocated storage";
pub static ERR_OPERATION_INVALID_WHEN_OPEN: & 'static str = 
  "Cannot perform this operation when storage is open";
pub static ERR_OPERATION_INVALID_WHEN_CLOSED: & 'static str = 
  "Cannot perform this operation when storage is closed";
pub static ERR_WRITE_NOTHING: & 'static str = 
  "End of write must be after start of write";
pub static ERR_READ_NOTHING: & 'static str = 
  "End of read must be after start of read";


pub trait BinaryStorage {

  fn open(&mut self) -> Result<(), Error>;
  fn close(&mut self) -> Result<(), Error>;

  fn is_open(&self) -> bool;

  fn w_i8(&mut self, offset: usize, data: i8) -> Result<(), Error>;
  fn w_i16(&mut self, offset: usize, data: i16) -> Result<(), Error>;
  fn w_i32(&mut self, offset: usize, data: i32) -> Result<(), Error>;
  fn w_i64(&mut self, offset: usize, data: i64) -> Result<(), Error>;

  fn w_u8(&mut self, offset: usize, data: u8) -> Result<(), Error>;
  fn w_u16(&mut self, offset: usize, data: u16) -> Result<(), Error>;
  fn w_u32(&mut self, offset: usize, data: u32) -> Result<(), Error>;
  fn w_u64(&mut self, offset: usize, data: u64) -> Result<(), Error>;

  fn w_f32(&mut self, offset: usize, data: f32) -> Result<(), Error>;
  fn w_f64(&mut self, offset: usize, data: f64) -> Result<(), Error>;

  fn w_bool(&mut self, offset: usize, data: bool) -> Result<(), Error>;

  fn w_bytes(&mut self, offset: usize, data: &[u8]) -> Result<(), Error>;
  fn w_str(&mut self, offset: usize, data: &str) -> Result<(), Error>;


  fn r_i8(&self, offset: usize) -> Result<i8, Error>;
  fn r_i16(&self, offset: usize) -> Result<i16, Error>;
  fn r_i32(&self, offset: usize) -> Result<i32, Error>;
  fn r_i64(&self, offset: usize) -> Result<i64, Error>;

  fn r_u8(&self, offset: usize) -> Result<u8, Error>;
  fn r_u16(&self, offset: usize) -> Result<u16, Error>;
  fn r_u32(&self, offset: usize) -> Result<u32, Error>;
  fn r_u64(&self, offset: usize) -> Result<u64, Error>;

  fn r_f32(&self, offset: usize) -> Result<f32, Error>;
  fn r_f64(&self, offset: usize) -> Result<f64, Error>;

  fn r_bool(&self, offset: usize) -> Result<bool, Error>;

  fn r_bytes(&self, offset: usize, len: usize) -> Result<Vec<u8>, Error>;
  fn r_str(&self, offset: usize, len: usize) -> Result<String, Error>;

  fn fill(
    &mut self, 
    start: Option<usize>, 
    end: Option<usize>, 
    val: u8
  ) -> Result<(), Error>;

  fn is_filled(
    &self, 
    start: Option<usize>, 
    end: Option<usize>, 
    val: u8
  ) -> Result<bool, Error>;

  fn get_expand_size(&self) -> usize;
  fn set_expand_size(&mut self, expand_size: usize) -> Result<(), Error>;

  fn get_capacity(&self) -> Result<usize, Error>;

  fn expand(&mut self, min_capacity: usize) -> Result<(), Error>;

}

