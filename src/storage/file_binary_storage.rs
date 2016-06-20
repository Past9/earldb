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


pub static ERR_NO_FILE: &'static str = 
    "File has not been opened";
pub static ERR_READ_LEN_TOO_LARGE: & 'static str = 
    "Read length is too large";

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
    use_txn_boundary: bool,
    txn_boundary: u64
}
impl FileBinaryStorage {

    pub fn new(
        path: String,
        create: bool,
        initial_capacity: u64,
        buffer_page_size: u32,
        buffer_max_pages: u32,
        expand_size: u64,
        use_txn_boundary: bool
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
            use_txn_boundary: use_txn_boundary,
            txn_boundary: 0
        })
    }

    fn write<T>(&mut self, offset: u64, data: &[u8]) -> Result<(), Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));
        try!(AssertionError::assert_not(
            self.use_txn_boundary && offset < self.txn_boundary,
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY
        ));
        try!(self.expand(offset + mem::size_of::<T>() as u64));

        {
            let mut file = try!(self.file());
            try!(file.seek(SeekFrom::Start(offset as u64)));
            try!(file.write(data)); 
        }

        let mut buffer = try!(self.buffer());

        buffer.update(offset as u64, data);

        Ok(())
    }

    fn read<T: Copy>(&mut self, offset: u64) -> Result<Vec<u8>, Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));
        try!(AssertionError::assert_not(
            self.use_txn_boundary && (offset + mem::size_of::<T>() as u64) > self.txn_boundary,
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY
        ));
        try!(AssertionError::assert(
            offset + mem::size_of::<T>() as u64 <= self.capacity, 
            binary_storage::ERR_READ_PAST_END
        ));

        let buffer = try!(self.buffer());
        Ok(try!(buffer.read(offset as u64, mem::size_of::<T>())))
    }

    fn file(&self) -> Result<&File, AssertionError> {
        match self.file {
            Some(ref f) => Ok(f),
            None => Err(AssertionError::new(ERR_NO_FILE))
        }
    }

    fn buffer(&mut self) -> Result<&mut FileSyncedBuffer, AssertionError> {
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
        try!(AssertionError::assert_not(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_OPEN));

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
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));

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
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));
        try!(AssertionError::assert_not(
            self.use_txn_boundary && offset < self.txn_boundary,
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY
        ));
        try!(self.expand(offset + data.len() as u64));

        {
            let mut file = try!(self.file());
            try!(file.seek(SeekFrom::Start(offset as u64)));
            try!(file.write(data)); 
        }

        let mut buffer = try!(self.buffer());
        buffer.update(offset as u64, data);

        Ok(())
    }

    fn w_str(&mut self, offset: u64, data: &str) -> Result<(), Error> { 
        self.w_bytes(offset, data.as_bytes()) 
    }


    fn r_i8(&mut self, offset: u64) -> Result<i8, Error> { 
        Ok(*(try!(self.read::<i8>(offset)).first().unwrap()) as i8)
    }

    fn r_i16(&mut self, offset: u64) -> Result<i16, Error> { 
        let data = try!(self.read::<i16>(offset));
        let mut rdr = Cursor::new(data);
        Ok(try!(rdr.read_i16::<LittleEndian>()))
    }

    fn r_i32(&mut self, offset: u64) -> Result<i32, Error> { 
        let data = try!(self.read::<i32>(offset));
        let mut rdr = Cursor::new(data);
        Ok(try!(rdr.read_i32::<LittleEndian>()))
    }

    fn r_i64(&mut self, offset: u64) -> Result<i64, Error> { 
        let data = try!(self.read::<i64>(offset));
        let mut rdr = Cursor::new(data);
        Ok(try!(rdr.read_i64::<LittleEndian>()))
    }

    fn r_u8(&mut self, offset: u64) -> Result<u8, Error> { 
        Ok(*(try!(self.read::<u8>(offset)).first().unwrap()))
    }

    fn r_u16(&mut self, offset: u64) -> Result<u16, Error> { 
        let data = try!(self.read::<u16>(offset));
        let mut rdr = Cursor::new(data);
        Ok(try!(rdr.read_u16::<LittleEndian>()))
    }

    fn r_u32(&mut self, offset: u64) -> Result<u32, Error> { 
        let data = try!(self.read::<u32>(offset));
        let mut rdr = Cursor::new(data);
        Ok(try!(rdr.read_u32::<LittleEndian>()))
    }

    fn r_u64(&mut self, offset: u64) -> Result<u64, Error> { 
        let data = try!(self.read::<u64>(offset));
        let mut rdr = Cursor::new(data);
        Ok(try!(rdr.read_u64::<LittleEndian>()))
    }

    fn r_f32(&mut self, offset: u64) -> Result<f32, Error> { 
        let data = try!(self.read::<f32>(offset));
        let mut rdr = Cursor::new(data);
        Ok(try!(rdr.read_f32::<LittleEndian>()))
    }

    fn r_f64(&mut self, offset: u64) -> Result<f64, Error> { 
        let data = try!(self.read::<f64>(offset));
        let mut rdr = Cursor::new(data);
        Ok(try!(rdr.read_f64::<LittleEndian>()))
    }

    fn r_bool(&mut self, offset: u64) -> Result<bool, Error> { 
        let byte = *try!(self.read::<bool>(offset)).first().unwrap();
        match byte {
            0 => Ok(false),
            _ => Ok(true)
        }
    }

    fn r_bytes(&mut self, offset: u64, len: usize) -> Result<Vec<u8>, Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));
        try!(AssertionError::assert_not(
            self.use_txn_boundary && (offset + len as u64) > self.txn_boundary,
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY
        ));
        try!(AssertionError::assert(
            offset + (len as u64) <= self.capacity, 
            binary_storage::ERR_READ_PAST_END
        ));

        let buffer = try!(self.buffer());
        let data = try!(buffer.read(offset as u64, len));
        Ok(data)
    }

    fn r_str(&mut self, offset: u64, len: usize) -> Result<String, Error> {
        let b = try!(self.r_bytes(offset, len));
        Ok(try!(str::from_utf8(b.as_slice())).to_string())
    }


    fn fill(&mut self, start: Option<u64>, end: Option<u64>, val: u8) -> Result<(), Error> {
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

        let len = end_offset - start_offset;
        let buf = vec![val; len as usize];

        {
            let mut file = try!(self.file());
            try!(file.seek(SeekFrom::Start(start_offset as u64)));
            try!(file.write(buf.as_slice())); 
        }

        let mut buffer = try!(self.buffer());
        buffer.update(start_offset as u64, buf.as_slice());

        Ok(())
    }

    fn is_filled(&mut self, start: Option<u64>, end: Option<u64>, val: u8) -> Result<bool, Error> {
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

        let buffer = try!(self.buffer());
        let len = end_offset - start_offset;


        let data = try!(buffer.read(start_offset as u64, len as usize));

        for b in data.as_slice() {
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
        try!(FileBinaryStorage::check_params(
            expand_size,
            self.initial_capacity
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

        // We don't want to reallocate (or even reduce the capacity) if we already have enough,
        // so just do nothing and return Ok if we already have enough room
        if new_capacity <= self.capacity { return Ok(()) }

        // Allocate more disk space
        {
            let file = try!(self.file());
            match file.set_len(new_capacity as u64) {
                Ok(()) => {},
                Err(_) => {
                    return Err(Error::Assertion(AssertionError::new(binary_storage::ERR_STORAGE_ALLOC)));
                }
            };
        }



        // Set the new capacity 
        self.capacity = new_capacity;
        // Return Ok to indicate that allocation was successful
        Ok(())
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
mod file_binary_storage_tests {

    use std::fs;
    use std::fs::OpenOptions;
    use std::path::Path;
    use uuid::Uuid;

    use error::Error;
    use storage::binary_storage::tests;
    use storage::binary_storage::BinaryStorage;
    use storage::file_binary_storage::FileBinaryStorage;


    pub static BASE_PATH: &'static str = "./test_data/storage/file_binary_storage/";

    fn rnd_path() -> String {
        BASE_PATH.to_string() 
            + Uuid::new_v4().simple().to_string().as_str()
            + ".tmp"
    }

    fn rm_tmp(filename: String) {
        fs::remove_file(filename).unwrap()
    }

    fn get_storage() -> (FileBinaryStorage, String) {
        let path = rnd_path();
        let s = FileBinaryStorage::new(
            path.clone(),
            true,
            256,
            16, 
            16,
            512,
            false
        ).unwrap();
        (s, path)
    }

    fn get_storage_expand_size(expand_size: u64) -> (FileBinaryStorage, String) {
        let path = rnd_path();
        let s = FileBinaryStorage::new(
            path.clone(),
            true,
            256,
            16, 
            16,
            expand_size,
            false
        ).unwrap();
        (s, path)
    }


    // open(), close(), and is_open() tests 
    #[test]
    pub fn open_returns_err_when_already_open() {
        let (s, p) = get_storage();
        tests::open_returns_err_when_already_open(s);
        rm_tmp(p);
    }

    #[test]
    pub fn open_creates_file_when_allowed_and_file_does_not_exist() {
        let path = rnd_path();
        assert!(!Path::new(path.clone().as_str()).exists());
        let mut s = FileBinaryStorage::new(
            path.clone(),
            true,
            256,
            16, 
            16,
            512,
            false
        ).unwrap();
        assert!(!Path::new(path.clone().as_str()).exists());
        s.open().unwrap();
        assert!(Path::new(path.clone().as_str()).exists());
        s.close().unwrap();
        assert!(Path::new(path.clone().as_str()).exists());
        rm_tmp(path);
    }

    #[test]
    pub fn open_creates_file_with_initial_capacity() {
        let path = rnd_path();
        let mut s = FileBinaryStorage::new(
            path.clone(),
            true,
            256,
            16, 
            16,
            512,
            false
        ).unwrap();
        s.open().unwrap();
        s.close().unwrap();
        let f = OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(path.clone()).unwrap();
        assert_eq!(256, f.metadata().unwrap().len());
        rm_tmp(path);
    }

    #[test]
    pub fn open_returns_io_err_when_file_does_not_exist_and_creation_not_allowed() {
        let path = rnd_path();
        assert!(!Path::new(path.clone().as_str()).exists());
        let mut s = FileBinaryStorage::new(
            path.clone(),
            false,
            256,
            16, 
            16,
            512,
            true
        ).unwrap();
        assert!(
            match s.open().unwrap_err() {
                Error::Io(_) => true,
                _ => false
            }
        );
    }

    #[test]
    pub fn open_does_not_open_when_file_does_not_exist_and_creation_not_allowed() {
        let path = rnd_path();
        assert!(!Path::new(path.clone().as_str()).exists());
        let mut s = FileBinaryStorage::new(
            path.clone(),
            false,
            256,
            16, 
            16,
            512,
            true
        ).unwrap();
        s.open().unwrap_err();
        assert!(!s.is_open());
    }

    #[test]
    pub fn open_does_not_create_file_when_not_allowed_and_file_does_not_exist() {
        let path = rnd_path();
        assert!(!Path::new(path.clone().as_str()).exists());
        let mut s = FileBinaryStorage::new(
            path.clone(),
            false,
            256,
            16, 
            16,
            512,
            true
        ).unwrap();
        assert!(!Path::new(path.clone().as_str()).exists());
        s.open().unwrap_err();
        assert!(!Path::new(path.clone().as_str()).exists());
        assert!(!Path::new(path.clone().as_str()).exists());
    }

    #[test]
    pub fn close_returns_err_when_already_closed() {
        let (s, _) = get_storage();
        tests::close_returns_err_when_already_closed(s);
    }

    #[test]
    pub fn open_returns_ok_when_previously_closed() {
        let (s, p) = get_storage();
        tests::open_returns_ok_when_previously_closed(s);
        rm_tmp(p);
    }

    #[test]
    pub fn close_returns_ok_when_previously_open() {
        let (s, p) = get_storage();
        tests::close_returns_ok_when_previously_open(s);
        rm_tmp(p);
    }

    #[test]
    fn is_closed_when_new() {
        let (s, _) = get_storage();
        tests::is_closed_when_new(s);
    }

    #[test]
    fn is_open_after_open() {
        let (s, p) = get_storage();
        tests::is_open_after_open(s);
        rm_tmp(p);
    }

    #[test]
    fn is_closed_after_open_and_close() {
        let (s, p) = get_storage();
        tests::is_closed_after_open_and_close(s);
        rm_tmp(p);
    }

    // new() tests
    // TODO: Write these

    // w_i8() tests
    #[test]
    fn w_i8_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::w_i8_returns_err_when_closed(s);
    }

    #[test]
    fn w_i8_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::w_i8_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn w_i8_does_not_write_when_closed() {
        let (s, p) = get_storage();
        tests::w_i8_does_not_write_when_closed(s);
        rm_tmp(p);
    }

    #[test]
    fn w_i8_does_not_write_before_txn_boundary() {
        let (s, p) = get_storage();
        tests::w_i8_does_not_write_before_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn w_i8_over_capacity_expands_storage() {
        let (s, p) = get_storage();
        tests::w_i8_over_capacity_expands_storage(s);
        rm_tmp(p);
    }

    // w_i16() tests
    #[test]
    fn w_i16_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::w_i16_returns_err_when_closed(s);
    }

    #[test]
    fn w_i16_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::w_i16_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn w_i16_does_not_write_when_closed() {
        let (s, p) = get_storage();
        tests::w_i16_does_not_write_when_closed(s);
        rm_tmp(p);
    }

    #[test]
    fn w_i16_does_not_write_before_txn_boundary() {
        let (s, p) = get_storage();
        tests::w_i16_does_not_write_before_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn w_i16_over_capacity_expands_storage() {
        let (s, p) = get_storage();
        tests::w_i16_over_capacity_expands_storage(s);
        rm_tmp(p);
    }

    // w_i32() tests
    #[test]
    fn w_i32_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::w_i32_returns_err_when_closed(s);
    }

    #[test]
    fn w_i32_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::w_i32_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn w_i32_does_not_write_when_closed() {
        let (s, p) = get_storage();
        tests::w_i32_does_not_write_when_closed(s);
        rm_tmp(p);
    }

    #[test]
    fn w_i32_does_not_write_before_txn_boundary() {
        let (s, p) = get_storage();
        tests::w_i32_does_not_write_before_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn w_i32_over_capacity_expands_storage() {
        let (s, p) = get_storage();
        tests::w_i32_over_capacity_expands_storage(s);
        rm_tmp(p);
    }

    // w_i64() tests
    #[test]
    fn w_i64_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::w_i64_returns_err_when_closed(s);
    }

    #[test]
    fn w_i64_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::w_i64_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn w_i64_does_not_write_when_closed() {
        let (s, p) = get_storage();
        tests::w_i64_does_not_write_when_closed(s);
        rm_tmp(p);
    }

    #[test]
    fn w_i64_does_not_write_before_txn_boundary() {
        let (s, p) = get_storage();
        tests::w_i64_does_not_write_before_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn w_i64_over_capacity_expands_storage() {
        let (s, p) = get_storage();
        tests::w_i64_over_capacity_expands_storage(s);
        rm_tmp(p);
    }

    // w_u8() tests
    #[test]
    fn w_u8_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::w_u8_returns_err_when_closed(s);
    }

    #[test]
    fn w_u8_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::w_u8_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn w_u8_does_not_write_when_closed() {
        let (s, p) = get_storage();
        tests::w_u8_does_not_write_when_closed(s);
        rm_tmp(p);
    }

    #[test]
    fn w_u8_does_not_write_before_txn_boundary() {
        let (s, p) = get_storage();
        tests::w_u8_does_not_write_before_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn w_u8_over_capacity_expands_storage() {
        let (s, p) = get_storage();
        tests::w_u8_over_capacity_expands_storage(s);
        rm_tmp(p);
    }

    // w_u16() tests
    #[test]
    fn w_u16_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::w_u16_returns_err_when_closed(s);
    }

    #[test]
    fn w_u16_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::w_u16_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn w_u16_does_not_write_when_closed() {
        let (s, p) = get_storage();
        tests::w_u16_does_not_write_when_closed(s);
        rm_tmp(p);
    }

    #[test]
    fn w_u16_does_not_write_before_txn_boundary() {
        let (s, p) = get_storage();
        tests::w_u16_does_not_write_before_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn w_u16_over_capacity_expands_storage() {
        let (s, p) = get_storage();
        tests::w_u16_over_capacity_expands_storage(s);
        rm_tmp(p);
    }

    // w_u32() tests
    #[test]
    fn w_u32_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::w_u32_returns_err_when_closed(s);
    }

    #[test]
    fn w_u32_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::w_u32_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn w_u32_does_not_write_when_closed() {
        let (s, p) = get_storage();
        tests::w_u32_does_not_write_when_closed(s);
        rm_tmp(p);
    }

    #[test]
    fn w_u32_does_not_write_before_txn_boundary() {
        let (s, p) = get_storage();
        tests::w_u32_does_not_write_before_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn w_u32_over_capacity_expands_storage() {
        let (s, p) = get_storage();
        tests::w_u32_over_capacity_expands_storage(s);
        rm_tmp(p);
    }

    // w_u64() tests
    #[test]
    fn w_u64_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::w_u64_returns_err_when_closed(s);
    }

    #[test]
    fn w_u64_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::w_u64_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn w_u64_does_not_write_when_closed() {
        let (s, p) = get_storage();
        tests::w_u64_does_not_write_when_closed(s);
        rm_tmp(p);
    }

    #[test]
    fn w_u64_does_not_write_before_txn_boundary() {
        let (s, p) = get_storage();
        tests::w_u64_does_not_write_before_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn w_u64_over_capacity_expands_storage() {
        let (s, p) = get_storage();
        tests::w_u64_over_capacity_expands_storage(s);
        rm_tmp(p);
    }

    // w_f32() tests
    #[test]
    fn w_f32_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::w_f32_returns_err_when_closed(s);
    }

    #[test]
    fn w_f32_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::w_f32_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn w_f32_does_not_write_when_closed() {
        let (s, p) = get_storage();
        tests::w_f32_does_not_write_when_closed(s);
        rm_tmp(p);
    }

    #[test]
    fn w_f32_does_not_write_before_txn_boundary() {
        let (s, p) = get_storage();
        tests::w_f32_does_not_write_before_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn w_f32_over_capacity_expands_storage() {
        let (s, p) = get_storage();
        tests::w_f32_over_capacity_expands_storage(s);
        rm_tmp(p);
    }

    // w_f64() tests
    #[test]
    fn w_f64_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::w_f64_returns_err_when_closed(s);
    }

    #[test]
    fn w_f64_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::w_f64_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn w_f64_does_not_write_when_closed() {
        let (s, p) = get_storage();
        tests::w_f64_does_not_write_when_closed(s);
        rm_tmp(p);
    }

    #[test]
    fn w_f64_does_not_write_before_txn_boundary() {
        let (s, p) = get_storage();
        tests::w_f64_does_not_write_before_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn w_f64_over_capacity_expands_storage() {
        let (s, p) = get_storage();
        tests::w_f64_over_capacity_expands_storage(s);
        rm_tmp(p);
    }

    // w_bool() tests
    #[test]
    fn w_bool_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::w_bool_returns_err_when_closed(s);
    }

    #[test]
    fn w_bool_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::w_bool_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn w_bool_does_not_write_when_closed() {
        let (s, p) = get_storage();
        tests::w_bool_does_not_write_when_closed(s);
        rm_tmp(p);
    }

    #[test]
    fn w_bool_does_not_write_before_txn_boundary() {
        let (s, p) = get_storage();
        tests::w_bool_does_not_write_before_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn w_bool_over_capacity_expands_storage() {
        let (s, p) = get_storage();
        tests::w_bool_over_capacity_expands_storage(s);
        rm_tmp(p);
    }

    // w_bytes() tests
    #[test]
    fn w_bytes_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::w_bytes_returns_err_when_closed(s);
    }

    #[test]
    fn w_bytes_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::w_bytes_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn w_bytes_does_not_write_when_closed() {
        let (s, p) = get_storage();
        tests::w_bytes_does_not_write_when_closed(s);
        rm_tmp(p);
    }

    #[test]
    fn w_bytes_does_not_write_before_txn_boundary() {
        let (s, p) = get_storage();
        tests::w_bytes_does_not_write_before_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn w_bytes_over_capacity_expands_storage() {
        let (s, p) = get_storage();
        tests::w_bytes_over_capacity_expands_storage(s);
        rm_tmp(p);
    }

    #[test]
    fn w_bytes_over_capacity_expands_storage_multiple_times() {
        let (s, p) = get_storage_expand_size(4);
        tests::w_bytes_over_capacity_expands_storage_multiple_times(s);
        rm_tmp(p);
    }

    // w_str() tests
    #[test]
    fn w_str_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::w_str_returns_err_when_closed(s);
    }

    #[test]
    fn w_str_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::w_str_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn w_str_does_not_write_when_closed() {
        let (s, p) = get_storage();
        tests::w_str_does_not_write_when_closed(s);
        rm_tmp(p);
    }

    #[test]
    fn w_str_does_not_write_before_txn_boundary() {
        let (s, p) = get_storage();
        tests::w_str_does_not_write_before_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn w_str_over_capacity_expands_storage() {
        let (s, p) = get_storage();
        tests::w_str_over_capacity_expands_storage(s);
        rm_tmp(p);
    }

    #[test]
    fn w_str_over_capacity_expands_storage_multiple_times() {
        let (s, p) = get_storage_expand_size(4);
        tests::w_str_over_capacity_expands_storage_multiple_times(s);
        rm_tmp(p);
    }

    // r_i8() tests
    #[test]
    fn r_i8_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::r_i8_returns_err_when_closed(s);
    }

    #[test]
    fn r_i8_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::r_i8_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i8_reads_zero_from_unwritten_storage() {
        let (s, p) = get_storage();
        tests::r_i8_reads_zero_from_unwritten_storage(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i8_reads_written_data() {
        let (s, p) = get_storage();
        tests::r_i8_reads_written_data(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i8_does_not_read_past_txn_boundary() {
        let (s, p) = get_storage();
        tests::r_i8_does_not_read_past_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i8_does_not_read_past_capacity() {
        let (s, p) = get_storage();
        tests::r_i8_does_not_read_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i8_result_is_not_mutated_on_subsequent_write() {
        let (s, p) = get_storage();
        tests::r_i8_result_is_not_mutated_on_subsequent_write(s);
        rm_tmp(p);
    }

    // r_i16() tests
    #[test]
    fn r_i16_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::r_i16_returns_err_when_closed(s);
    }

    #[test]
    fn r_i16_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::r_i16_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i16_reads_zero_from_unwritten_storage() {
        let (s, p) = get_storage();
        tests::r_i16_reads_zero_from_unwritten_storage(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i16_reads_written_data() {
        let (s, p) = get_storage();
        tests::r_i16_reads_written_data(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i16_does_not_read_past_txn_boundary() {
        let (s, p) = get_storage();
        tests::r_i16_does_not_read_past_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i16_does_not_read_past_capacity() {
        let (s, p) = get_storage();
        tests::r_i16_does_not_read_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i16_result_is_not_mutated_on_subsequent_write() {
        let (s, p) = get_storage();
        tests::r_i16_result_is_not_mutated_on_subsequent_write(s);
        rm_tmp(p);
    }

    // r_i32() tests
    #[test]
    fn r_i32_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::r_i32_returns_err_when_closed(s);
    }

    #[test]
    fn r_i32_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::r_i32_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i32_reads_zero_from_unwritten_storage() {
        let (s, p) = get_storage();
        tests::r_i32_reads_zero_from_unwritten_storage(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i32_reads_written_data() {
        let (s, p) = get_storage();
        tests::r_i32_reads_written_data(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i32_does_not_read_past_txn_boundary() {
        let (s, p) = get_storage();
        tests::r_i32_does_not_read_past_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i32_does_not_read_past_capacity() {
        let (s, p) = get_storage();
        tests::r_i32_does_not_read_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i32_result_is_not_mutated_on_subsequent_write() {
        let (s, p) = get_storage();
        tests::r_i32_result_is_not_mutated_on_subsequent_write(s);
        rm_tmp(p);
    }

    // r_i64() tests
    #[test]
    fn r_i64_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::r_i64_returns_err_when_closed(s);
    }

    #[test]
    fn r_i64_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::r_i64_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i64_reads_zero_from_unwritten_storage() {
        let (s, p) = get_storage();
        tests::r_i64_reads_zero_from_unwritten_storage(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i64_reads_written_data() {
        let (s, p) = get_storage();
        tests::r_i64_reads_written_data(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i64_does_not_read_past_txn_boundary() {
        let (s, p) = get_storage();
        tests::r_i64_does_not_read_past_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i64_does_not_read_past_capacity() {
        let (s, p) = get_storage();
        tests::r_i64_does_not_read_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn r_i64_result_is_not_mutated_on_subsequent_write() {
        let (s, p) = get_storage();
        tests::r_i64_result_is_not_mutated_on_subsequent_write(s);
        rm_tmp(p);
    }

    // r_u8() tests
    #[test]
    fn r_u8_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::r_u8_returns_err_when_closed(s);
    }

    #[test]
    fn r_u8_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::r_u8_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u8_reads_zero_from_unwritten_storage() {
        let (s, p) = get_storage();
        tests::r_u8_reads_zero_from_unwritten_storage(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u8_reads_written_data() {
        let (s, p) = get_storage();
        tests::r_u8_reads_written_data(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u8_does_not_read_past_txn_boundary() {
        let (s, p) = get_storage();
        tests::r_u8_does_not_read_past_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u8_does_not_read_past_capacity() {
        let (s, p) = get_storage();
        tests::r_u8_does_not_read_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u8_result_is_not_mutated_on_subsequent_write() {
        let (s, p) = get_storage();
        tests::r_u8_result_is_not_mutated_on_subsequent_write(s);
        rm_tmp(p);
    }

    // r_u16() tests
    #[test]
    fn r_u16_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::r_u16_returns_err_when_closed(s);
    }

    #[test]
    fn r_u16_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::r_u16_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u16_reads_zero_from_unwritten_storage() {
        let (s, p) = get_storage();
        tests::r_u16_reads_zero_from_unwritten_storage(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u16_reads_written_data() {
        let (s, p) = get_storage();
        tests::r_u16_reads_written_data(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u16_does_not_read_past_txn_boundary() {
        let (s, p) = get_storage();
        tests::r_u16_does_not_read_past_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u16_does_not_read_past_capacity() {
        let (s, p) = get_storage();
        tests::r_u16_does_not_read_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u16_result_is_not_mutated_on_subsequent_write() {
        let (s, p) = get_storage();
        tests::r_u16_result_is_not_mutated_on_subsequent_write(s);
        rm_tmp(p);
    }

    // r_u32() tests
    #[test]
    fn r_u32_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::r_u32_returns_err_when_closed(s);
    }

    #[test]
    fn r_u32_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::r_u32_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u32_reads_zero_from_unwritten_storage() {
        let (s, p) = get_storage();
        tests::r_u32_reads_zero_from_unwritten_storage(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u32_reads_written_data() {
        let (s, p) = get_storage();
        tests::r_u32_reads_written_data(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u32_does_not_read_past_txn_boundary() {
        let (s, p) = get_storage();
        tests::r_u32_does_not_read_past_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u32_does_not_read_past_capacity() {
        let (s, p) = get_storage();
        tests::r_u32_does_not_read_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u32_result_is_not_mutated_on_subsequent_write() {
        let (s, p) = get_storage();
        tests::r_u32_result_is_not_mutated_on_subsequent_write(s);
        rm_tmp(p);
    }

    // r_u64() tests
    #[test]
    fn r_u64_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::r_u64_returns_err_when_closed(s);
    }

    #[test]
    fn r_u64_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::r_u64_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u64_reads_zero_from_unwritten_storage() {
        let (s, p) = get_storage();
        tests::r_u64_reads_zero_from_unwritten_storage(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u64_reads_written_data() {
        let (s, p) = get_storage();
        tests::r_u64_reads_written_data(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u64_does_not_read_past_txn_boundary() {
        let (s, p) = get_storage();
        tests::r_u64_does_not_read_past_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u64_does_not_read_past_capacity() {
        let (s, p) = get_storage();
        tests::r_u64_does_not_read_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn r_u64_result_is_not_mutated_on_subsequent_write() {
        let (s, p) = get_storage();
        tests::r_u64_result_is_not_mutated_on_subsequent_write(s);
        rm_tmp(p);
    }

    // r_f32() tests
    #[test]
    fn r_f32_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::r_f32_returns_err_when_closed(s);
    }

    #[test]
    fn r_f32_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::r_f32_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn r_f32_reads_zero_from_unwritten_storage() {
        let (s, p) = get_storage();
        tests::r_f32_reads_zero_from_unwritten_storage(s);
        rm_tmp(p);
    }

    #[test]
    fn r_f32_reads_written_data() {
        let (s, p) = get_storage();
        tests::r_f32_reads_written_data(s);
        rm_tmp(p);
    }

    #[test]
    fn r_f32_does_not_read_past_txn_boundary() {
        let (s, p) = get_storage();
        tests::r_f32_does_not_read_past_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn r_f32_does_not_read_past_capacity() {
        let (s, p) = get_storage();
        tests::r_f32_does_not_read_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn r_f32_result_is_not_mutated_on_subsequent_write() {
        let (s, p) = get_storage();
        tests::r_f32_result_is_not_mutated_on_subsequent_write(s);
        rm_tmp(p);
    }

    // r_f64() tests
    #[test]
    fn r_f64_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::r_f64_returns_err_when_closed(s);
    }

    #[test]
    fn r_f64_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::r_f64_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn r_f64_reads_zero_from_unwritten_storage() {
        let (s, p) = get_storage();
        tests::r_f64_reads_zero_from_unwritten_storage(s);
        rm_tmp(p);
    }

    #[test]
    fn r_f64_reads_written_data() {
        let (s, p) = get_storage();
        tests::r_f64_reads_written_data(s);
        rm_tmp(p);
    }

    #[test]
    fn r_f64_does_not_read_past_txn_boundary() {
        let (s, p) = get_storage();
        tests::r_f64_does_not_read_past_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn r_f64_does_not_read_past_capacity() {
        let (s, p) = get_storage();
        tests::r_f64_does_not_read_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn r_f64_result_is_not_mutated_on_subsequent_write() {
        let (s, p) = get_storage();
        tests::r_f64_result_is_not_mutated_on_subsequent_write(s);
        rm_tmp(p);
    }

    // r_bool() tests
    #[test]
    fn r_bool_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::r_bool_returns_err_when_closed(s);
    }

    #[test]
    fn r_bool_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::r_bool_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn r_bool_reads_false_from_unwritten_storage() {
        let (s, p) = get_storage();
        tests::r_bool_reads_false_from_unwritten_storage(s);
        rm_tmp(p);
    }

    #[test]
    fn r_bool_reads_written_data() {
        let (s, p) = get_storage();
        tests::r_bool_reads_written_data(s);
        rm_tmp(p);
    }

    #[test]
    fn r_bool_does_not_read_past_txn_boundary() {
        let (s, p) = get_storage();
        tests::r_bool_does_not_read_past_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn r_bool_does_not_read_past_capacity() {
        let (s, p) = get_storage();
        tests::r_bool_does_not_read_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn r_bool_result_is_not_mutated_on_subsequent_write() {
        let (s, p) = get_storage();
        tests::r_bool_result_is_not_mutated_on_subsequent_write(s);
        rm_tmp(p);
    }

    // r_bytes() tests
    #[test]
    fn r_bytes_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::r_bytes_returns_err_when_closed(s);
    }

    #[test]
    fn r_bytes_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::r_bytes_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn r_bytes_reads_zeros_from_unwritten_storage() {
        let (s, p) = get_storage();
        tests::r_bytes_reads_zeros_from_unwritten_storage(s);
        rm_tmp(p);
    }

    #[test]
    fn r_bytes_reads_written_data() {
        let (s, p) = get_storage();
        tests::r_bytes_reads_written_data(s);
        rm_tmp(p);
    }

    #[test]
    fn r_bytes_does_not_read_past_txn_boundary() {
        let (s, p) = get_storage();
        tests::r_bytes_does_not_read_past_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn r_bytes_does_not_read_past_capacity() {
        let (s, p) = get_storage();
        tests::r_bytes_does_not_read_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn r_bytes_result_is_not_mutated_on_subsequent_write() {
        let (s, p) = get_storage();
        tests::r_bytes_result_is_not_mutated_on_subsequent_write(s);
        rm_tmp(p);
    }

    // r_str() tests
    #[test]
    fn r_str_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::r_str_returns_err_when_closed(s);
    }

    #[test]
    fn r_str_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::r_str_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn r_str_reads_nulls_from_unwritten_storage() {
        let (s, p) = get_storage();
        tests::r_str_reads_nulls_from_unwritten_storage(s);
        rm_tmp(p);
    }

    #[test]
    fn r_str_reads_written_data() {
        let (s, p) = get_storage();
        tests::r_str_reads_written_data(s);
        rm_tmp(p);
    }

    #[test]
    fn r_str_does_not_read_past_txn_boundary() {
        let (s, p) = get_storage();
        tests::r_str_does_not_read_past_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn r_str_does_not_read_past_capacity() {
        let (s, p) = get_storage();
        tests::r_str_does_not_read_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn r_str_result_is_not_mutated_on_subsequent_write() {
        let (s, p) = get_storage();
        tests::r_str_result_is_not_mutated_on_subsequent_write(s);
        rm_tmp(p);
    }

    // fill() tests
    #[test]
    fn fill_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::fill_returns_err_when_closed(s);
    }

    #[test]
    fn fill_does_not_write_when_closed() {
        let (s, p) = get_storage();
        tests::fill_does_not_write_when_closed(s);
        rm_tmp(p);
    }

    #[test]
    fn fill_returns_ok_when_open() {
        let (s, p) = get_storage();
        tests::fill_returns_ok_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn fill_repeats_byte_in_storage_range() {
        let (s, p) = get_storage();
        tests::fill_repeats_byte_in_storage_range(s);
        rm_tmp(p);
    }

    #[test]
    fn fill_starts_from_beginning_when_start_offset_is_none() {
        let (s, p) = get_storage();
        tests::fill_starts_from_beginning_when_start_offset_is_none(s);
        rm_tmp(p);
    }

    #[test]
    fn fill_goes_to_end_when_end_offset_is_none() {
        let (s, p) = get_storage();
        tests::fill_goes_to_end_when_end_offset_is_none(s);
        rm_tmp(p);
    }

    #[test]
    fn fill_returns_err_when_end_offset_is_before_start_offset() {
        let (s, p) = get_storage();
        tests::fill_returns_err_when_end_offset_is_before_start_offset(s);
        rm_tmp(p);
    }

    #[test]
    fn fill_does_not_write_when_end_offset_is_before_start_offset() {
        let (s, p) = get_storage();
        tests::fill_does_not_write_when_end_offset_is_before_start_offset(s);
        rm_tmp(p);
    }

    #[test]
    fn fill_returns_err_when_before_txn_boundary() {
        let (s, p) = get_storage();
        tests::fill_returns_err_when_before_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn fill_does_not_write_when_before_txn_boundary() {
        let (s, p) = get_storage();
        tests::fill_does_not_write_when_before_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn fill_returns_ok_when_after_txn_boundary() {
        let (s, p) = get_storage();
        tests::fill_returns_ok_when_after_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn fill_writes_when_after_txn_boundary() {
        let (s, p) = get_storage();
        tests::fill_writes_when_after_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn fill_returns_err_when_past_capacity() {
        let (s, p) = get_storage();
        tests::fill_returns_err_when_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn fill_does_not_write_when_past_capacity() {
        let (s, p) = get_storage();
        tests::fill_does_not_write_when_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn fill_does_not_expand_capacity() {
        let (s, p) = get_storage();
        tests::fill_does_not_expand_capacity(s);
        rm_tmp(p);
    }

    // assert_filled() tests
    #[test]
    fn is_filled_retuns_err_when_closed() {
        let (s, _) = get_storage();
        tests::is_filled_retuns_err_when_closed(s);
    }

    #[test]
    fn is_filled_returns_err_when_start_offset_past_capacity() {
        let (s, p) = get_storage();
        tests::is_filled_returns_err_when_start_offset_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn is_filled_returns_err_when_end_offset_at_or_before_start_offset() {
        let (s, p) = get_storage();
        tests::is_filled_returns_err_when_end_offset_at_or_before_start_offset(s);
        rm_tmp(p);
    }

    #[test]
    fn is_filled_returns_err_when_end_offset_past_capacity() {
        let (s, p) = get_storage();
        tests::is_filled_returns_err_when_end_offset_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn is_filled_checks_whether_all_bytes_in_range_match_value() {
        let (s, p) = get_storage();
        tests::is_filled_checks_whether_all_bytes_in_range_match_value(s);
        rm_tmp(p);
    }

    #[test]
    fn is_filled_starts_from_start_offset() {
        let (s, p) = get_storage();
        tests::is_filled_starts_from_start_offset(s);
        rm_tmp(p);
    }

    #[test]
    fn is_filled_starts_from_beginning_when_start_offset_is_none() {
        let (s, p) = get_storage();
        tests::is_filled_starts_from_beginning_when_start_offset_is_none(s);
        rm_tmp(p);
    }

    #[test]
    fn is_filled_goes_to_end_offset() {
        let (s, p) = get_storage();
        tests::is_filled_goes_to_end_offset(s);
        rm_tmp(p);
    }

    #[test]
    fn is_filled_goes_to_end_when_end_offset_is_none() {
        let (s, p) = get_storage();
        tests::is_filled_goes_to_end_when_end_offset_is_none(s);
        rm_tmp(p);
    }

    // get_use_txn_boundary(), set_use_txn_boundary(), get_txn_boundary(), and set_txn_boundary() tests
    #[test]
    fn set_use_txn_boundary_changes_value() {
        let (s, _) = get_storage();
        tests::set_use_txn_boundary_changes_value(s);
    }

    #[test]
    fn set_use_txn_boundary_resets_boundary_to_zero_when_txn_boundary_turned_off() {
        let (s, p) = get_storage();
        tests::set_use_txn_boundary_resets_boundary_to_zero_when_txn_boundary_turned_off(s);
        rm_tmp(p);
    }

    #[test]
    fn get_txn_boundary_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::get_txn_boundary_returns_err_when_closed(s);
    }

    #[test]
    fn get_txn_boundary_returns_err_when_not_using_txn_boundary() {
        let (s, p) = get_storage();
        tests::get_txn_boundary_returns_err_when_not_using_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn get_txn_boundary_starts_at_0() {
        let (s, p) = get_storage();
        tests::get_txn_boundary_starts_at_0(s);
        rm_tmp(p);
    }

    #[test]
    fn set_txn_boundary_returns_err_when_not_using_txn_boundary() {
        let (s, p) = get_storage();
        tests::set_txn_boundary_returns_err_when_not_using_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn set_txn_boundary_does_not_change_boundary_when_not_using_txn_boundary() {
        let (s, p) = get_storage();
        tests::set_txn_boundary_does_not_change_boundary_when_not_using_txn_boundary(s);
        rm_tmp(p);
    }

    #[test]
    fn set_txn_boundary_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::set_txn_boundary_returns_err_when_closed(s);
    }

    #[test]
    fn set_txn_boundary_does_not_change_boundary_when_closed() {
        let (s, p) = get_storage();
        tests::set_txn_boundary_does_not_change_boundary_when_closed(s);
        rm_tmp(p);
    }

    #[test]
    fn set_txn_boundary_returns_err_when_past_capacity() {
        let (s, p) = get_storage();
        tests::set_txn_boundary_returns_err_when_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn set_txn_boundary_does_not_change_boundary_when_past_capacity() {
        let (s, p) = get_storage();
        tests::set_txn_boundary_does_not_change_boundary_when_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn set_txn_boundary_does_not_expand_capacity_when_past_capacity() {
        let (s, p) = get_storage();
        tests::set_txn_boundary_does_not_expand_capacity_when_past_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn set_txn_boundary_changes_boundary() {
        let (s, p) = get_storage();
        tests::set_txn_boundary_changes_boundary(s);
        rm_tmp(p);
    }

    // get_expand_size() and set_expand_size() tests
    #[test]
    fn get_expand_size_returns_initial_expand_size() {
        let (s, _) = get_storage();
        tests::get_expand_size_returns_initial_expand_size(s);
    }

    #[test]
    fn set_expand_size_returns_err_when_expand_size_is_zero() {
        let (s, _) = get_storage();
        tests::set_expand_size_returns_err_when_expand_size_is_zero(s);
    }

    #[test]
    fn set_expand_size_does_not_change_expand_size_when_expand_size_is_zero() {
        let (s, _) = get_storage();
        tests::set_expand_size_does_not_change_expand_size_when_expand_size_is_zero(s);
    }

    #[test]
    fn set_expand_size_returns_err_when_expand_size_is_not_power_of_2() {
        let (s, _) = get_storage();
        tests::set_expand_size_returns_err_when_expand_size_is_not_power_of_2(s);
    }

    #[test]
    fn set_expand_size_does_not_change_expand_size_when_expand_size_is_not_power_of_2() {
        let (s, _) = get_storage();
        tests::set_expand_size_does_not_change_expand_size_when_expand_size_is_not_power_of_2(s);
    }

    #[test]
    fn set_expand_size_returns_true_when_checks_pass() {
        let (s, _) = get_storage();
        tests::set_expand_size_returns_true_when_checks_pass(s);
    }

    #[test]
    fn set_expand_size_changes_expand_size_when_checks_pass() {
        let (s, _) = get_storage();
        tests::set_expand_size_changes_expand_size_when_checks_pass(s);
    }

    #[test]
    fn capacity_increases_to_increments_of_last_set_expand_size() {
        let (s, p) = get_storage();
        tests::capacity_increases_to_increments_of_last_set_expand_size(s);
        rm_tmp(p);
    }

    // get_capacity() tests
    #[test]
    fn get_capacity_returns_err_when_closed() {
        let (s, p) = get_storage();
        tests::get_capacity_returns_err_when_closed(s);
        rm_tmp(p);
    }

    #[test]
    fn get_capacity_returns_initial_capacity_when_open() {
        let (s, p) = get_storage();
        tests::get_capacity_returns_initial_capacity_when_open(s);
        rm_tmp(p);
    }

    #[test]
    fn get_capacity_returns_new_capacity_after_expansion() {
        let (s, p) = get_storage();
        tests::get_capacity_returns_new_capacity_after_expansion(s);
        rm_tmp(p);
    }

    // expand() tests
    #[test]
    fn expand_returns_err_when_closed() {
        let (s, _) = get_storage();
        tests::expand_returns_err_when_closed(s);
    }

    #[test]
    fn expand_does_not_change_capacity_when_closed() {
        let (s, p) = get_storage();
        tests::expand_does_not_change_capacity_when_closed(s);
        rm_tmp(p);
    }

    #[test]
    fn expand_returns_ok_when_already_has_capacity() {
        let (s, p) = get_storage();
        tests::expand_returns_ok_when_already_has_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn expand_does_not_change_capacity_when_already_has_capacity() {
        let (s, p) = get_storage();
        tests::expand_does_not_change_capacity_when_already_has_capacity(s);
        rm_tmp(p);
    }

    #[test]
    fn expand_returns_err_when_allocation_arithmetic_overflows() {
        let (s, p) = get_storage();
        tests::expand_returns_err_when_allocation_arithmetic_overflows(s);
        rm_tmp(p);
    }

    #[test]
    fn expand_does_not_change_capacity_when_allocation_arithmetic_overflows() {
        let (s, p) = get_storage();
        tests::expand_does_not_change_capacity_when_allocation_arithmetic_overflows(s);
        rm_tmp(p);
    }

    #[test]
    fn expand_returns_err_when_allocation_fails() {
        let (s, p) = get_storage();
        tests::expand_returns_err_when_allocation_fails(s);
        rm_tmp(p);
    }

    #[test]
    fn expand_does_not_change_capacity_when_allocation_fails() {
        let (s, p) = get_storage();
        tests::expand_does_not_change_capacity_when_allocation_fails(s);
        rm_tmp(p);
    }

    #[test]
    fn expand_returns_ok_when_successful() {
        let (s, p) = get_storage();
        tests::expand_returns_ok_when_successful(s);
        rm_tmp(p);
    }

    #[test]
    fn expand_changes_capacity_by_expand_size_when_successful() {
        let (s, p) = get_storage();
        tests::expand_changes_capacity_by_expand_size_when_successful(s);
        rm_tmp(p);
    }

    #[test]
    fn expand_changes_capacity_by_multiples_of_expand_size_when_successful() {
        let (s, p) = get_storage();
        tests::expand_changes_capacity_by_multiples_of_expand_size_when_successful(s);
        rm_tmp(p);
    }

}
