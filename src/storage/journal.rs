use std::mem;

use error::{ Error, AssertionError };
use storage::binary_storage::BinaryStorage;
use storage::transactional_storage::TransactionalStorage;
use storage::util::xor_checksum;
use storage::binary_storage;

pub static ERR_WRITE_IN_PROGRESS: & 'static str =
  "Cannot perform this operation while an uncommitted write is in progress";
pub static ERR_WRITE_NOT_IN_PROGRESS: & 'static str =
  "Cannot perform this operation when no write is in progress";
pub static ERR_NOTHING_TO_WRITE: & 'static str =
  "Cannot write 0 bytes";
pub static ERR_NO_COMMITTED_RECORD: & 'static str =
  "Location is not the start of a committed record";
pub static ERR_NO_RECORD_DATA: & 'static str =
  "Record contains no data";
pub static ERR_CHECKSUM_MISMATCH: & 'static str =
  "Checksum mismatch, record data may be corrupted";

pub const PRE_DATA_LEN: u64 = 6;
pub const POST_DATA_LEN: u64 = 3;

pub struct Journal<T: BinaryStorage + Sized> {
  storage: TransactionalStorage<T>,
  read_offset: u64,
  write_offset: u64,
  is_writing: bool,
  uncommitted_size: u64,
  record_count: u64
}
impl<T: BinaryStorage + Sized> Journal<T> {

  pub fn new(mut storage: TransactionalStorage<T>) -> Journal<T> {
    Journal {
      storage: storage,
      read_offset: 0,
      write_offset: 0,
      is_writing: false,
      uncommitted_size: 0,
      record_count: 0
    }
  }


  pub fn open(&mut self) -> Result<(), Error> {
    self.storage.open().and(self.verify())
  }

  pub fn close(&mut self) -> Result<(), Error> {
    match self.storage.close() {
      Ok(_) => {
        self.read_offset = 0;
        self.write_offset = 0;
        self.is_writing = false;
        self.uncommitted_size = 0;
        self.record_count = 0;
        Ok(())
      },
      Err(e) => Err(e)
    }
  }

  pub fn is_open(&self) -> bool {
    self.storage.is_open()
  }

  pub fn verify(&mut self) -> Result<(), Error> {

    // Start at the beginning of storage
    self.reset();

    // Turn off transaction checking temporarily since we don't
    // know where the boundary is yet
    self.storage.set_check_on_read(false);
    
    // Count all the good committed records
    let mut count = 0;
    for _ in self.into_iter() {
      count += 1;
    }
    self.record_count = count;

    // check to see if the start marker exists. If an error occurs during the
    // check, turn transaction checking back on before returning the error 
    let has_start = match self.has_start() {
      Ok(h) => h,
      Err(e) => {
        self.storage.set_check_on_read(true);
        return Err(e);
      }
    };

    if has_start {
      let data = match self.read() {
        Ok(d) => d,
        Err(e) => {
          self.storage.set_check_on_read(true);
          return Err(e);
        }
      };
      self.write_offset = self.read_offset + 
        mem::size_of::<u16>() as u64 + 
        mem::size_of::<u32>() as u64 + 
        data.len() as u64 +
        mem::size_of::<u8>() as u64; 
      self.is_writing = true;
    }

    self.storage.set_check_on_read(true);

    // Reset to the beginning and return Ok
    self.reset();
    Ok(())
      
  }


  pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
    // TODO: constrain data size
    try!(AssertionError::assert_not(self.is_writing, ERR_WRITE_IN_PROGRESS));
    try!(AssertionError::assert(data.len() > 0, ERR_NOTHING_TO_WRITE));

    self.is_writing = true;

    match self.storage.w_u16(self.write_offset, 514) {
      Ok(()) =>  {
        self.write_offset += mem::size_of::<u16>() as u64;
        self.uncommitted_size = mem::size_of::<u16>() as u64;
      },
      Err(e) => match self.discard() {
        Ok(()) => return Err(e),
        Err(d) => return Err(d)
      }
    };

    // Length of data plus checksum byte
    let len = data.len() as u64;

    match self.storage.w_u32(self.write_offset, len as u32) {
      Ok(()) => {
        self.write_offset += mem::size_of::<u32>() as u64;
        self.uncommitted_size += mem::size_of::<u32>() as u64;
      },
      Err(e) => match self.discard() {
        Ok(()) => return Err(e),
        Err(d) => return Err(d)
      }
    };

    match self.storage.w_bytes(self.write_offset, data) {
      Ok(()) => {
        self.write_offset += len;
        self.uncommitted_size += len;
      },
      Err(e) => match self.discard() {
        Ok(()) => return Err(e),
        Err(d) => return Err(d)
      }
    };

    match self.storage.w_u8(self.write_offset, xor_checksum(data)) {
      Ok(()) =>  {
        self.write_offset += mem::size_of::<u8>() as u64;
        self.uncommitted_size += mem::size_of::<u8>() as u64;
      },
      Err(e) => match self.discard() {
        Ok(()) => return Err(e),
        Err(d) => return Err(d)
      }
    }

    Ok(())
  }

  pub fn commit(&mut self) -> Result<(), Error> {
    try!(AssertionError::assert(self.is_writing, ERR_WRITE_NOT_IN_PROGRESS));

    match self.storage.w_u16(self.write_offset, 771) {
      Ok(()) =>  {
        self.write_offset += mem::size_of::<u16>() as u64;
        self.uncommitted_size += mem::size_of::<u16>() as u64;
      },
      Err(e) => match self.discard() {
        Ok(()) => return Err(e),
        Err(d) => return Err(d)
      }
    };

    self.storage.set_txn_boundary(self.write_offset);
    self.uncommitted_size = 0;
    self.is_writing = false;

    self.record_count += 1;

    Ok(())

  }

  pub fn discard(&mut self) -> Result<(), Error> {
    try!(AssertionError::assert(self.is_writing, ERR_WRITE_NOT_IN_PROGRESS));

    self.storage.set_txn_boundary(self.write_offset - self.uncommitted_size);

    self.write_offset -= self.uncommitted_size;
    self.uncommitted_size = 0;
    self.is_writing = false;
    Ok(())
  }

  pub fn is_writing(&self) -> bool {
    self.is_writing
  }

  pub fn reset(&mut self) {
    self.read_offset = 0;
  }

  pub fn has_start(&mut self) -> Result<bool, Error> {
    Ok(
      514 == try!(self.storage.r_u16(self.read_offset)) 
    )
  }

  pub fn has_end(&mut self) -> Result<bool, Error> {
    let len = try!(self.storage.r_u32(self.read_offset + mem::size_of::<u16>() as u64));
    Ok(
      771 == try!(self.storage.r_u16(
        self.read_offset + 
          PRE_DATA_LEN +
          len as u64 +
          mem::size_of::<u8>() as u64
      ))
    )
  }

  pub fn read(&mut self) -> Result<Vec<u8>, Error> {

    let len = try!(
      self.storage.r_u32(self.read_offset + mem::size_of::<u16>() as u64)
    ) as usize;
    try!(AssertionError::assert(len > 1, ERR_NO_RECORD_DATA));
    let mut bytes = try!(self.storage.r_bytes(
      self.read_offset + PRE_DATA_LEN,
      len + mem::size_of::<u8>(),
    ));

    let checksum = match bytes.pop() {
      Some(s) => s,
      None => return Err(Error::Assertion(AssertionError::new(ERR_NO_RECORD_DATA)))
    };

    try!(AssertionError::assert(
      checksum == xor_checksum(bytes.as_slice()), 
      ERR_CHECKSUM_MISMATCH
    ));

    Ok(bytes)
  }

  pub fn jump_to(&mut self, offset: u64) -> Result<(), Error> {
    self.read_offset = offset;

    match self.has_start() {
      Ok(v) => {
        if !v {
          return Err(Error::from(AssertionError::new(ERR_NO_COMMITTED_RECORD)));
        }
      },
      Err(e) => {
        return Err(e);
      }
    };

    match self.has_end() {
      Ok(v) => {
        if !v {
          return Err(Error::from(AssertionError::new(ERR_NO_COMMITTED_RECORD)));
        }
      },
      Err(e) => {
        return Err(e);
      }
    };

    Ok(())
  }


  pub fn read_offset(&self) -> u64 { self.read_offset }

  pub fn write_offset(&self) -> u64 { self.write_offset }

  pub fn capacity(&self) -> Result<u64, Error> { self.storage.get_capacity() }

  pub fn record_count(&self) -> u64 { self.record_count }

  pub fn txn_boundary(&self) -> Result<u64, Error> {
    self.storage.get_txn_boundary()
  }

}
impl<T: BinaryStorage + Sized> Iterator for Journal<T> {

  type Item = Vec<u8>;

  fn next(&mut self) -> Option<Vec<u8>> {

    match self.has_start().and(self.has_end()) {
      Ok(h) => if !h { return None },
      Err(_) => return None
    };

    match self.read() {
      Ok(v) => {

        let new_offset = self.read_offset + 
          PRE_DATA_LEN + 
          v.len() as u64 +
          POST_DATA_LEN;

        match self.jump_to(new_offset) {
          Ok(_) => {},
          Err(_) => {}
        };

        Some(v)

      },
      Err(_) => None
    }

  }
}
