use std::fs::{ File, OpenOptions };
use std::path::Path;
use std::mem;
use std::io::{ Cursor, Write, Seek, SeekFrom };
use std::str;

use byteorder::{ LittleEndian, ReadBytesExt, WriteBytesExt };

use storage::util;
use storage::binary_storage;
use storage::binary_storage::BinaryStorage;
use storage::file_synced_buffer::FileSyncedBuffer;
use error::{ Error, AssertionError };


pub static ERR_NO_FILE: &'static str = "File has not been opened";

pub struct FileBinaryStorage {
  path: String,
  create: bool,
  file: Option<File>,
  buffer: Option<FileSyncedBuffer>,
  buffer_page_size: u32,
  buffer_max_pages: u32,
  is_open: bool,
  initial_capacity: u64,
  capacity: u64,
  expand_size: u64,
}
impl FileBinaryStorage {

  pub fn new(
    path: String,
    create: bool,
    initial_capacity: u64,
    buffer_page_size: u32,
    buffer_max_pages: u32,
    expand_size: u64,
  ) -> Result<FileBinaryStorage, Error> {

    try!(FileBinaryStorage::check_params(
      expand_size,
      initial_capacity
    )); 

    Ok(FileBinaryStorage {
      path: path,
      create: create,
      file: None,
      buffer: None,
      buffer_page_size: buffer_page_size,
      buffer_max_pages: buffer_max_pages,
      is_open: false,
      initial_capacity: initial_capacity,
      capacity: 0,
      expand_size: expand_size,
    })
  }

  fn write<T>(&mut self, offset: u64, data: &[u8]) -> Result<(), Error> {
    try!(AssertionError::assert(
      self.is_open, 
      binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED
    ));

    let end_offset = try!(util::u64_add(offset, mem::size_of::<T>() as u64));

    try!(self.expand(end_offset));

    {
      let mut file = try!(self.file());
      try!(file.seek(SeekFrom::Start(offset)));
      try!(file.write(data)); 
    }

    let mut buffer = try!(self.buffer_mut());

    buffer.update(offset, data);

    Ok(())
  }

  fn read<T: Copy>(&self, offset: u64) -> Result<Vec<u8>, Error> {
    try!(AssertionError::assert(
      self.is_open, 
      binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED
    ));

    let end_offset = try!(util::u64_add(offset, mem::size_of::<T>() as u64));

    try!(AssertionError::assert_not(
      end_offset > self.capacity, 
      binary_storage::ERR_READ_PAST_END
    ));

    let buffer = try!(self.buffer());
    Ok(try!(buffer.read(offset, mem::size_of::<T>())))
  }

  fn file(&self) -> Result<&File, AssertionError> {
    match self.file {
      Some(ref f) => Ok(f),
      None => Err(AssertionError::new(ERR_NO_FILE))
    }
  }


  fn buffer(&self) -> Result<&FileSyncedBuffer, AssertionError> {
    match self.buffer {
      Some(ref b) => Ok(b),
      None => Err(AssertionError::new(ERR_NO_FILE))
    }
  }

  fn buffer_mut(&mut self) -> Result<&mut FileSyncedBuffer, AssertionError> {
    match self.buffer {
      Some(ref mut b) => Ok(b),
      None => Err(AssertionError::new(ERR_NO_FILE))
    }
  }

  fn check_params(
    expand_size: u64,
    initial_capacity: u64
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
impl BinaryStorage for FileBinaryStorage {

    fn open(&mut self) -> Result<(), Error> {
      try!(AssertionError::assert_not(
        self.is_open, 
        binary_storage::ERR_OPERATION_INVALID_WHEN_OPEN
      ));

      let preexisting = Path::new(self.path.as_str()).exists();

      let write_file = try!(
        OpenOptions::new()
          .write(true)
          .create(self.create)
          .open(self.path.clone())
      );

      if !preexisting && self.create {
        try!(write_file.set_len(self.initial_capacity as u64));
        try!(write_file.sync_all());
      }

      self.capacity = try!(write_file.metadata()).len();

      let read_file = try!(
        OpenOptions::new()
          .read(true)
          .open(self.path.clone())
      );

      let buffer = FileSyncedBuffer::new(
        read_file, 
        self.buffer_page_size, 
        self.buffer_max_pages 
      );

      self.file = Some(write_file);
      self.buffer = Some(buffer);

      self.is_open = true;
      Ok(())
    }

    fn close(&mut self) -> Result<(), Error> {
      try!(AssertionError::assert(
        self.is_open, 
        binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED
      ));

      self.file = None;
      self.buffer = None;

      self.is_open = false;
      Ok(())
    }

    fn w_i8(&mut self, offset: u64, data: i8) -> Result<(), Error> { 
      self.write::<i8>(offset, vec!(data as u8).as_slice())
    }

    fn w_i16(&mut self, offset: u64, data: i16) -> Result<(), Error> { 
      let mut buf = vec![];
      try!(buf.write_i16::<LittleEndian>(data));
      self.write::<i16>(offset, buf.as_slice())
    }

    fn w_i32(&mut self, offset: u64, data: i32) -> Result<(), Error> { 
      let mut buf = vec![];
      try!(buf.write_i32::<LittleEndian>(data));
      self.write::<i32>(offset, buf.as_slice())
    }

    fn w_i64(&mut self, offset: u64, data: i64) -> Result<(), Error> { 
      let mut buf = vec![];
      try!(buf.write_i64::<LittleEndian>(data));
      self.write::<i64>(offset, buf.as_slice())
    }

    fn w_u8(&mut self, offset: u64, data: u8) -> Result<(), Error> { 
      self.write::<u8>(offset, vec!(data).as_slice())
    }

    fn w_u16(&mut self, offset: u64, data: u16) -> Result<(), Error> { 
      let mut buf = vec![];
      try!(buf.write_u16::<LittleEndian>(data));
      self.write::<u16>(offset, buf.as_slice())
    }

    fn w_u32(&mut self, offset: u64, data: u32) -> Result<(), Error> { 
      let mut buf = vec![];
      try!(buf.write_u32::<LittleEndian>(data));
      self.write::<u32>(offset, buf.as_slice())
    }

    fn w_u64(&mut self, offset: u64, data: u64) -> Result<(), Error> { 
      let mut buf = vec![];
      try!(buf.write_u64::<LittleEndian>(data));
      self.write::<u64>(offset, buf.as_slice())
    }

    fn w_f32(&mut self, offset: u64, data: f32) -> Result<(), Error> { 
      let mut buf = vec![];
      try!(buf.write_f32::<LittleEndian>(data));
      self.write::<f32>(offset, buf.as_slice())
    }

    fn w_f64(&mut self, offset: u64, data: f64) -> Result<(), Error> { 
      let mut buf = vec![];
      try!(buf.write_f64::<LittleEndian>(data));
      self.write::<f64>(offset, buf.as_slice())
    }

    fn w_bool(&mut self, offset: u64, data: bool) -> Result<(), Error> { 
      self.write::<bool>(offset, vec!(data as u8).as_slice())
    }

    fn w_bytes(&mut self, offset: u64, data: &[u8]) -> Result<(), Error> {
      try!(AssertionError::assert(
        self.is_open, 
        binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED
      ));

      let end_offset = try!(util::u64_add(offset, data.len() as u64));

      try!(self.expand(end_offset));

      {
        let mut file = try!(self.file());
        try!(file.seek(SeekFrom::Start(offset as u64)));
        try!(file.write(data)); 
      }

      let mut buffer = try!(self.buffer_mut());
      buffer.update(offset, data);

      Ok(())
    }

    fn w_str(&mut self, offset: u64, data: &str) -> Result<(), Error> { 
      self.w_bytes(offset, data.as_bytes()) 
    }


    fn r_i8(&self, offset: u64) -> Result<i8, Error> { 
      Ok(*(try!(self.read::<i8>(offset)).first().unwrap()) as i8)
    }

    fn r_i16(&self, offset: u64) -> Result<i16, Error> { 
      let data = try!(self.read::<i16>(offset));
      let mut rdr = Cursor::new(data);
      Ok(try!(rdr.read_i16::<LittleEndian>()))
    }

    fn r_i32(&self, offset: u64) -> Result<i32, Error> { 
      let data = try!(self.read::<i32>(offset));
      let mut rdr = Cursor::new(data);
      Ok(try!(rdr.read_i32::<LittleEndian>()))
    }

    fn r_i64(&self, offset: u64) -> Result<i64, Error> { 
      let data = try!(self.read::<i64>(offset));
      let mut rdr = Cursor::new(data);
      Ok(try!(rdr.read_i64::<LittleEndian>()))
    }

    fn r_u8(&self, offset: u64) -> Result<u8, Error> { 
      Ok(*(try!(self.read::<u8>(offset)).first().unwrap()))
    }

    fn r_u16(&self, offset: u64) -> Result<u16, Error> { 
      let data = try!(self.read::<u16>(offset));
      let mut rdr = Cursor::new(data);
      Ok(try!(rdr.read_u16::<LittleEndian>()))
    }

    fn r_u32(&self, offset: u64) -> Result<u32, Error> { 
      let data = try!(self.read::<u32>(offset));
      let mut rdr = Cursor::new(data);
      Ok(try!(rdr.read_u32::<LittleEndian>()))
    }

    fn r_u64(&self, offset: u64) -> Result<u64, Error> { 
      let data = try!(self.read::<u64>(offset));
      let mut rdr = Cursor::new(data);
      Ok(try!(rdr.read_u64::<LittleEndian>()))
    }

    fn r_f32(&self, offset: u64) -> Result<f32, Error> { 
      let data = try!(self.read::<f32>(offset));
      let mut rdr = Cursor::new(data);
      Ok(try!(rdr.read_f32::<LittleEndian>()))
    }

    fn r_f64(&self, offset: u64) -> Result<f64, Error> { 
      let data = try!(self.read::<f64>(offset));
      let mut rdr = Cursor::new(data);
      Ok(try!(rdr.read_f64::<LittleEndian>()))
    }

    fn r_bool(&self, offset: u64) -> Result<bool, Error> { 
      let byte = *try!(self.read::<bool>(offset)).first().unwrap();
      match byte {
        0 => Ok(false),
        _ => Ok(true)
      }
    }

    fn r_bytes(&self, offset: u64, len: usize) -> Result<Vec<u8>, Error> {
      try!(AssertionError::assert(
        self.is_open, 
        binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED
      ));

      let end_offset = try!(util::u64_add(offset, len as u64));

      try!(AssertionError::assert_not(
        end_offset > self.capacity, 
        binary_storage::ERR_READ_PAST_END
      ));

      let buffer = try!(self.buffer());
      let data = try!(buffer.read(offset as u64, len));
      Ok(data)
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

      let start_offset = match start { Some(s) => s, None => 0 };
      let end_offset = match end { Some(e) => e, None => self.capacity };

      try!(AssertionError::assert(
        start_offset < self.capacity, 
        binary_storage::ERR_WRITE_PAST_END
      ));

      try!(AssertionError::assert(
        start_offset < self.capacity, 
        binary_storage::ERR_WRITE_PAST_END
      ));

      try!(AssertionError::assert(
        end_offset <= self.capacity,
        binary_storage::ERR_WRITE_PAST_END
      ));

      try!(AssertionError::assert(
        end_offset > start_offset,
        binary_storage::ERR_WRITE_NOTHING
      ));

      let len = end_offset - start_offset;
      let buf = vec![val; try!(util::u64_as_usize(len))];

      {
        let mut file = try!(self.file());
        try!(file.seek(SeekFrom::Start(start_offset as u64)));
        try!(file.write(buf.as_slice())); 
      }

      let mut buffer = try!(self.buffer_mut());
      buffer.update(start_offset, buf.as_slice());

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

      let start_offset = match start {
        Some(s) => s,
        None => 0
      };
      let end_offset = match end {
        Some(e) => e,
        None => self.capacity
      };

      try!(AssertionError::assert(
        start_offset < self.capacity, 
        binary_storage::ERR_READ_PAST_END
      ));

      try!(AssertionError::assert(
        end_offset <= self.capacity,
        binary_storage::ERR_READ_PAST_END
      ));

      try!(AssertionError::assert(
        end_offset > start_offset,
        binary_storage::ERR_READ_NOTHING
      ));

      let buffer = try!(self.buffer());
      let len = end_offset - start_offset;

      let data = try!(buffer.read(start_offset, try!(util::u64_as_usize(len))));

      for b in data.as_slice() {
        if *b != val { return Ok(false) }
      }

      Ok(true)
    }

    fn get_expand_size(&self) -> u64 {
      self.expand_size
    }

    fn set_expand_size(&mut self, expand_size: u64) -> Result<(), Error> {
      try!(FileBinaryStorage::check_params(
        expand_size,
        self.initial_capacity
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

      // We don't want to reallocate (or even reduce the capacity) if we 
      // already have enough, so just do nothing and return Ok if we 
      // already have enough room.
      if new_capacity <= self.capacity { return Ok(()) }

      // Allocate more disk space
      {
        let file = try!(self.file());
        match file.set_len(new_capacity) {
          Ok(()) => {},
          Err(_) => {
            return Err(Error::Assertion(
              AssertionError::new(binary_storage::ERR_STORAGE_ALLOC)
            ));
          }
        };
      }

      // Set the new capacity 
      self.capacity = new_capacity;
      // Return Ok to indicate that allocation was successful
      Ok(())
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

