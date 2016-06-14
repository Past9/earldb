use std::fs::{ File, OpenOptions };
use std::path::Path;
use std::mem;
use std::io::{ Cursor, Write, Seek, SeekFrom };
use std::str;

use byteorder::{ BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt };

use storage::binary_storage;
use storage::binary_storage::BinaryStorage;
use storage::file_synced_buffer::FileSyncedBuffer;
use error::{ Error, AssertionError };


pub static ERR_NO_FILE: &'static str = 
    "File has not been opened";

pub struct FileBinaryStorage {
    path: String,
    create: bool,
    file: Option<File>,
    buffer: Option<FileSyncedBuffer>,
    buffer_page_size: u32,
    buffer_max_pages: u32,
    is_open: bool,
    initial_capacity: usize,
    capacity: usize,
    expand_size: usize,
    use_txn_boundary: bool,
    txn_boundary: usize
}
impl FileBinaryStorage {

    pub fn new(
        path: String,
        create: bool,
        initial_capacity: usize,
        buffer_page_size: u32,
        buffer_max_pages: u32,
        expand_size: usize,
        use_txn_boundary: bool
    ) -> FileBinaryStorage {
        FileBinaryStorage {
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
        }
    }

    fn write<T>(&mut self, offset: usize, data: &[u8]) -> Result<(), Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));
        try!(AssertionError::assert_not(
            self.use_txn_boundary && offset < self.txn_boundary,
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY
        ));
        try!(self.expand(offset + mem::size_of::<T>()));

        {
            let mut file = try!(self.file());
            try!(file.seek(SeekFrom::Start(offset as u64)));
            try!(file.write(data)); 
        }

        let mut buffer = try!(self.buffer());

        buffer.update(offset as u64, data);

        Ok(())
    }

    fn read<T: Copy>(&mut self, offset: usize) -> Result<Vec<u8>, Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));
        try!(AssertionError::assert_not(
            self.use_txn_boundary && (offset + mem::size_of::<T>()) > self.txn_boundary,
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY
        ));
        try!(AssertionError::assert(
            offset + mem::size_of::<T>() <= self.capacity, 
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

        self.capacity = try!(write_file.metadata()).len() as usize;

        let read_file = try!(
            OpenOptions::new()
                .read(true)
                .open(self.path.clone())
        );

        let buffer = FileSyncedBuffer::new(
            read_file, 
            self.buffer_page_size, 
            self.buffer_max_pages, 
            8
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

    fn w_i8(&mut self, offset: usize, data: i8) -> Result<(), Error> { 
        self.write::<i8>(offset, vec!(data as u8).as_slice())
    }

    fn w_i16(&mut self, offset: usize, data: i16) -> Result<(), Error> { 
        let mut buf = vec![];
        buf.write_i16::<LittleEndian>(data);
        self.write::<i16>(offset, buf.as_slice())
    }

    fn w_i32(&mut self, offset: usize, data: i32) -> Result<(), Error> { 
        let mut buf = vec![];
        buf.write_i32::<LittleEndian>(data);
        self.write::<i32>(offset, buf.as_slice())
    }

    fn w_i64(&mut self, offset: usize, data: i64) -> Result<(), Error> { 
        let mut buf = vec![];
        buf.write_i64::<LittleEndian>(data);
        self.write::<i64>(offset, buf.as_slice())
    }

    fn w_u8(&mut self, offset: usize, data: u8) -> Result<(), Error> { 
        self.write::<u8>(offset, vec!(data).as_slice())
    }

    fn w_u16(&mut self, offset: usize, data: u16) -> Result<(), Error> { 
        let mut buf = vec![];
        buf.write_u16::<LittleEndian>(data);
        self.write::<u16>(offset, buf.as_slice())
    }

    fn w_u32(&mut self, offset: usize, data: u32) -> Result<(), Error> { 
        let mut buf = vec![];
        buf.write_u32::<LittleEndian>(data);
        self.write::<u32>(offset, buf.as_slice())
    }

    fn w_u64(&mut self, offset: usize, data: u64) -> Result<(), Error> { 
        let mut buf = vec![];
        buf.write_u64::<LittleEndian>(data);
        self.write::<u64>(offset, buf.as_slice())
    }

    fn w_f32(&mut self, offset: usize, data: f32) -> Result<(), Error> { 
        let mut buf = vec![];
        buf.write_f32::<LittleEndian>(data);
        self.write::<f32>(offset, buf.as_slice())
    }

    fn w_f64(&mut self, offset: usize, data: f64) -> Result<(), Error> { 
        let mut buf = vec![];
        buf.write_f64::<LittleEndian>(data);
        self.write::<f64>(offset, buf.as_slice())
    }

    fn w_bool(&mut self, offset: usize, data: bool) -> Result<(), Error> { 
        self.write::<bool>(offset, vec!(data as u8).as_slice())
    }

    fn w_bytes(&mut self, offset: usize, data: &[u8]) -> Result<(), Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));
        try!(AssertionError::assert_not(
            self.use_txn_boundary && offset < self.txn_boundary,
            binary_storage::ERR_WRITE_BEFORE_TXN_BOUNDARY
        ));
        try!(self.expand(offset + data.len()));

        {
            let mut file = try!(self.file());
            try!(file.seek(SeekFrom::Start(offset as u64)));
            try!(file.write(data)); 
        }

        let mut buffer = try!(self.buffer());

        buffer.update(offset as u64, data);

        Ok(())
    }

    fn w_str(&mut self, offset: usize, data: &str) -> Result<(), Error> { 
        self.w_bytes(offset, data.as_bytes()) 
    }


    fn r_i8(&mut self, offset: usize) -> Result<i8, Error> { 
        Ok(*(try!(self.read::<i8>(offset)).first().unwrap()) as i8)
    }

    fn r_i16(&mut self, offset: usize) -> Result<i16, Error> { 
        let data = try!(self.read::<i16>(offset));
        let mut rdr = Cursor::new(data);
        Ok(try!(rdr.read_i16::<LittleEndian>()))
    }

    fn r_i32(&mut self, offset: usize) -> Result<i32, Error> { 
        let data = try!(self.read::<i32>(offset));
        let mut rdr = Cursor::new(data);
        Ok(try!(rdr.read_i32::<LittleEndian>()))
    }

    fn r_i64(&mut self, offset: usize) -> Result<i64, Error> { 
        let data = try!(self.read::<i64>(offset));
        let mut rdr = Cursor::new(data);
        Ok(try!(rdr.read_i64::<LittleEndian>()))
    }

    fn r_u8(&mut self, offset: usize) -> Result<u8, Error> { 
        Ok(*(try!(self.read::<u8>(offset)).first().unwrap()))
    }

    fn r_u16(&mut self, offset: usize) -> Result<u16, Error> { 
        let data = try!(self.read::<u16>(offset));
        let mut rdr = Cursor::new(data);
        Ok(try!(rdr.read_u16::<LittleEndian>()))
    }

    fn r_u32(&mut self, offset: usize) -> Result<u32, Error> { 
        let data = try!(self.read::<u32>(offset));
        let mut rdr = Cursor::new(data);
        Ok(try!(rdr.read_u32::<LittleEndian>()))
    }

    fn r_u64(&mut self, offset: usize) -> Result<u64, Error> { 
        let data = try!(self.read::<u64>(offset));
        let mut rdr = Cursor::new(data);
        Ok(try!(rdr.read_u64::<LittleEndian>()))
    }

    fn r_f32(&mut self, offset: usize) -> Result<f32, Error> { 
        let data = try!(self.read::<f32>(offset));
        let mut rdr = Cursor::new(data);
        Ok(try!(rdr.read_f32::<LittleEndian>()))
    }

    fn r_f64(&mut self, offset: usize) -> Result<f64, Error> { 
        let data = try!(self.read::<f64>(offset));
        let mut rdr = Cursor::new(data);
        Ok(try!(rdr.read_f64::<LittleEndian>()))
    }

    fn r_bool(&mut self, offset: usize) -> Result<bool, Error> { 
        let byte = *try!(self.read::<bool>(offset)).first().unwrap();
        match byte {
            0 => Ok(false),
            _ => Ok(true)
        }
    }

    fn r_bytes(&mut self, offset: usize, len: usize) -> Result<Vec<u8>, Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));
        try!(AssertionError::assert_not(
            self.use_txn_boundary && (offset + len) > self.txn_boundary,
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY
        ));
        try!(AssertionError::assert(
            offset + len <= self.capacity, 
            binary_storage::ERR_READ_PAST_END
        ));

        let buffer = try!(self.buffer());
        let data = try!(buffer.read(offset as u64, len));
        Ok(data)
    }

    fn r_str(&mut self, offset: usize, len: usize) -> Result<String, Error> {
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

        let len = end_offset - start_offset;
        let buf = vec![val; len];

        {
            let mut file = try!(self.file());
            try!(file.seek(SeekFrom::Start(start_offset as u64)));
            try!(file.write(buf.as_slice())); 
        }

        let mut buffer = try!(self.buffer());

        buffer.update(start_offset as u64, buf.as_slice());

        Ok(())
    }

    fn is_filled(&mut self, start: Option<usize>, end: Option<usize>, val: u8) -> Result<bool, Error> {
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
        let data = try!(buffer.read(start_offset as u64, end_offset - start_offset));

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
        self.expand_size = expand_size;
        Ok(())
    }


    fn expand(&mut self, min_capacity: usize) -> Result<(), Error> {
        try!(AssertionError::assert(self.is_open, binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED));

        // Determine the new size of the journal in multiples of expand_size
        let expand_increments = (min_capacity as f64 / self.expand_size as f64).ceil() as usize;
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
        try!(self.file()).set_len(new_capacity as u64);

        // Set the new capacity 
        self.capacity = new_capacity;
        // Return Ok to indicate that allocation was successful
        Ok(())
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
mod file_binary_storage_tests {

    use std::fs;
    use uuid::{ Uuid, UuidVersion };

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

    fn get_storage() -> FileBinaryStorage {
        FileBinaryStorage::new(
            rnd_path(),
            true,
            256,
            16, 
            16,
            512,
            false
        )
    }


    // open(), close(), and is_open() tests 
    #[test]
    pub fn open_returns_err_when_already_open() {
        tests::open_returns_err_when_already_open(
            get_storage()
        );
    }

    #[test]
    pub fn close_returns_err_when_already_closed() {
        tests::close_returns_err_when_already_closed(
            get_storage()
        );
    }

    #[test]
    pub fn open_returns_ok_when_previously_closed() {
        tests::open_returns_ok_when_previously_closed(
            get_storage()
        );
    }

    #[test]
    pub fn close_returns_ok_when_previously_open() {
        tests::close_returns_ok_when_previously_open(
            get_storage()
        );
    }

    #[test]
    fn is_closed_when_new() {
        tests::is_closed_when_new(
            get_storage()
        );
    }

    #[test]
    fn is_open_after_open() {
        tests::is_open_after_open(
            get_storage()
        );
    }

    #[test]
    fn is_closed_after_open_and_close() {
        tests::is_closed_after_open_and_close(
            get_storage()
        );
    }

    // new() tests
    // TODO: Write these

    // w_i8() tests
    #[test]
    fn w_i8_returns_err_when_closed() {
        tests::w_i8_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_i8_returns_ok_when_open() {
        tests::w_i8_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_i8_does_not_write_when_closed() {
        tests::w_i8_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_i8_does_not_write_before_txn_boundary() {
        tests::w_i8_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_i8_over_capacity_expands_storage() {
        tests::w_i8_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_i16() tests
    #[test]
    fn w_i16_returns_err_when_closed() {
        tests::w_i16_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_i16_returns_ok_when_open() {
        tests::w_i16_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_i16_does_not_write_when_closed() {
        tests::w_i16_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_i16_does_not_write_before_txn_boundary() {
        tests::w_i16_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_i16_over_capacity_expands_storage() {
        tests::w_i16_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_i32() tests
    #[test]
    fn w_i32_returns_err_when_closed() {
        tests::w_i32_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_i32_returns_ok_when_open() {
        tests::w_i32_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_i32_does_not_write_when_closed() {
        tests::w_i32_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_i32_does_not_write_before_txn_boundary() {
        tests::w_i32_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_i32_over_capacity_expands_storage() {
        tests::w_i32_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_i64() tests
    #[test]
    fn w_i64_returns_err_when_closed() {
        tests::w_i64_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_i64_returns_ok_when_open() {
        tests::w_i64_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_i64_does_not_write_when_closed() {
        tests::w_i64_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_i64_does_not_write_before_txn_boundary() {
        tests::w_i64_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_i64_over_capacity_expands_storage() {
        tests::w_i64_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_u8() tests
    #[test]
    fn w_u8_returns_err_when_closed() {
        tests::w_u8_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_u8_returns_ok_when_open() {
        tests::w_u8_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_u8_does_not_write_when_closed() {
        tests::w_u8_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_u8_does_not_write_before_txn_boundary() {
        tests::w_u8_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_u8_over_capacity_expands_storage() {
        tests::w_u8_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_u16() tests
    #[test]
    fn w_u16_returns_err_when_closed() {
        tests::w_u16_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_u16_returns_ok_when_open() {
        tests::w_u16_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_u16_does_not_write_when_closed() {
        tests::w_u16_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_u16_does_not_write_before_txn_boundary() {
        tests::w_u16_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_u16_over_capacity_expands_storage() {
        tests::w_u16_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_u32() tests
    #[test]
    fn w_u32_returns_err_when_closed() {
        tests::w_u32_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_u32_returns_ok_when_open() {
        tests::w_u32_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_u32_does_not_write_when_closed() {
        tests::w_u32_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_u32_does_not_write_before_txn_boundary() {
        tests::w_u32_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_u32_over_capacity_expands_storage() {
        tests::w_u32_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_u64() tests
    #[test]
    fn w_u64_returns_err_when_closed() {
        tests::w_u64_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_u64_returns_ok_when_open() {
        tests::w_u64_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_u64_does_not_write_when_closed() {
        tests::w_u64_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_u64_does_not_write_before_txn_boundary() {
        tests::w_u64_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_u64_over_capacity_expands_storage() {
        tests::w_u64_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_f32() tests
    #[test]
    fn w_f32_returns_err_when_closed() {
        tests::w_f32_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_f32_returns_ok_when_open() {
        tests::w_f32_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_f32_does_not_write_when_closed() {
        tests::w_f32_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_f32_does_not_write_before_txn_boundary() {
        tests::w_f32_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_f32_over_capacity_expands_storage() {
        tests::w_f32_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_f64() tests
    #[test]
    fn w_f64_returns_err_when_closed() {
        tests::w_f64_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_f64_returns_ok_when_open() {
        tests::w_f64_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_f64_does_not_write_when_closed() {
        tests::w_f64_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_f64_does_not_write_before_txn_boundary() {
        tests::w_f64_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_f64_over_capacity_expands_storage() {
        tests::w_f64_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_bool() tests
    #[test]
    fn w_bool_returns_err_when_closed() {
        tests::w_bool_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_bool_returns_ok_when_open() {
        tests::w_bool_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_bool_does_not_write_when_closed() {
        tests::w_bool_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_bool_does_not_write_before_txn_boundary() {
        tests::w_bool_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_bool_over_capacity_expands_storage() {
        tests::w_bool_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_bytes() tests
    #[test]
    fn w_bytes_returns_err_when_closed() {
        tests::w_bytes_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_bytes_returns_ok_when_open() {
        tests::w_bytes_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_bytes_does_not_write_when_closed() {
        tests::w_bytes_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_bytes_does_not_write_before_txn_boundary() {
        tests::w_bytes_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_bytes_over_capacity_expands_storage() {
        tests::w_bytes_over_capacity_expands_storage(
            get_storage()
        );
    }

    #[test]
    fn w_bytes_over_capacity_expands_storage_multiple_times() {
        tests::w_bytes_over_capacity_expands_storage_multiple_times(
            get_storage()
        );
    }

    // w_str() tests
    #[test]
    fn w_str_returns_err_when_closed() {
        tests::w_str_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_str_returns_ok_when_open() {
        tests::w_str_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_str_does_not_write_when_closed() {
        tests::w_str_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_str_does_not_write_before_txn_boundary() {
        tests::w_str_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_str_over_capacity_expands_storage() {
        tests::w_str_over_capacity_expands_storage(
            get_storage()
        );
    }

    #[test]
    fn w_str_over_capacity_expands_storage_multiple_times() {
        tests::w_str_over_capacity_expands_storage_multiple_times(
            get_storage()
        );
    }

    // r_i8() tests
    #[test]
    fn r_i8_returns_err_when_closed() {
        tests::r_i8_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_i8_returns_ok_when_open() {
        tests::r_i8_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_i8_reads_zero_from_unwritten_storage() {
        tests::r_i8_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_i8_reads_written_data() {
        tests::r_i8_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_i8_does_not_read_past_txn_boundary() {
        tests::r_i8_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_i8_does_not_read_past_capacity() {
        tests::r_i8_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_i8_result_is_not_mutated_on_subsequent_write() {
        tests::r_i8_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_i16() tests
    #[test]
    fn r_i16_returns_err_when_closed() {
        tests::r_i16_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_i16_returns_ok_when_open() {
        tests::r_i16_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_i16_reads_zero_from_unwritten_storage() {
        tests::r_i16_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_i16_reads_written_data() {
        tests::r_i16_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_i16_does_not_read_past_txn_boundary() {
        tests::r_i16_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_i16_does_not_read_past_capacity() {
        tests::r_i16_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_i16_result_is_not_mutated_on_subsequent_write() {
        tests::r_i16_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_i32() tests
    #[test]
    fn r_i32_returns_err_when_closed() {
        tests::r_i32_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_i32_returns_ok_when_open() {
        tests::r_i32_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_i32_reads_zero_from_unwritten_storage() {
        tests::r_i32_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_i32_reads_written_data() {
        tests::r_i32_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_i32_does_not_read_past_txn_boundary() {
        tests::r_i32_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_i32_does_not_read_past_capacity() {
        tests::r_i32_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_i32_result_is_not_mutated_on_subsequent_write() {
        tests::r_i32_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_i64() tests
    #[test]
    fn r_i64_returns_err_when_closed() {
        tests::r_i64_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_i64_returns_ok_when_open() {
        tests::r_i64_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_i64_reads_zero_from_unwritten_storage() {
        tests::r_i64_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_i64_reads_written_data() {
        tests::r_i64_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_i64_does_not_read_past_txn_boundary() {
        tests::r_i64_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_i64_does_not_read_past_capacity() {
        tests::r_i64_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_i64_result_is_not_mutated_on_subsequent_write() {
        tests::r_i64_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_u8() tests
    #[test]
    fn r_u8_returns_err_when_closed() {
        tests::r_u8_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_u8_returns_ok_when_open() {
        tests::r_u8_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_u8_reads_zero_from_unwritten_storage() {
        tests::r_u8_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_u8_reads_written_data() {
        tests::r_u8_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_u8_does_not_read_past_txn_boundary() {
        tests::r_u8_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_u8_does_not_read_past_capacity() {
        tests::r_u8_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_u8_result_is_not_mutated_on_subsequent_write() {
        tests::r_u8_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_u16() tests
    #[test]
    fn r_u16_returns_err_when_closed() {
        tests::r_u16_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_u16_returns_ok_when_open() {
        tests::r_u16_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_u16_reads_zero_from_unwritten_storage() {
        tests::r_u16_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_u16_reads_written_data() {
        tests::r_u16_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_u16_does_not_read_past_txn_boundary() {
        tests::r_u16_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_u16_does_not_read_past_capacity() {
        tests::r_u16_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_u16_result_is_not_mutated_on_subsequent_write() {
        tests::r_u16_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_u32() tests
    #[test]
    fn r_u32_returns_err_when_closed() {
        tests::r_u32_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_u32_returns_ok_when_open() {
        tests::r_u32_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_u32_reads_zero_from_unwritten_storage() {
        tests::r_u32_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_u32_reads_written_data() {
        tests::r_u32_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_u32_does_not_read_past_txn_boundary() {
        tests::r_u32_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_u32_does_not_read_past_capacity() {
        tests::r_u32_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_u32_result_is_not_mutated_on_subsequent_write() {
        tests::r_u32_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_u64() tests
    #[test]
    fn r_u64_returns_err_when_closed() {
        tests::r_u64_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_u64_returns_ok_when_open() {
        tests::r_u64_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_u64_reads_zero_from_unwritten_storage() {
        tests::r_u64_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_u64_reads_written_data() {
        tests::r_u64_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_u64_does_not_read_past_txn_boundary() {
        tests::r_u64_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_u64_does_not_read_past_capacity() {
        tests::r_u64_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_u64_result_is_not_mutated_on_subsequent_write() {
        tests::r_u64_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_f32() tests
    #[test]
    fn r_f32_returns_err_when_closed() {
        tests::r_f32_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_f32_returns_ok_when_open() {
        tests::r_f32_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_f32_reads_zero_from_unwritten_storage() {
        tests::r_f32_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_f32_reads_written_data() {
        tests::r_f32_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_f32_does_not_read_past_txn_boundary() {
        tests::r_f32_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_f32_does_not_read_past_capacity() {
        tests::r_f32_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_f32_result_is_not_mutated_on_subsequent_write() {
        tests::r_f32_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_f64() tests
    #[test]
    fn r_f64_returns_err_when_closed() {
        tests::r_f64_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_f64_returns_ok_when_open() {
        tests::r_f64_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_f64_reads_zero_from_unwritten_storage() {
        tests::r_f64_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_f64_reads_written_data() {
        tests::r_f64_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_f64_does_not_read_past_txn_boundary() {
        tests::r_f64_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_f64_does_not_read_past_capacity() {
        tests::r_f64_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_f64_result_is_not_mutated_on_subsequent_write() {
        tests::r_f64_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_bool() tests
    #[test]
    fn r_bool_returns_err_when_closed() {
        tests::r_bool_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_bool_returns_ok_when_open() {
        tests::r_bool_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_bool_reads_false_from_unwritten_storage() {
        tests::r_bool_reads_false_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_bool_reads_written_data() {
        tests::r_bool_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_bool_does_not_read_past_txn_boundary() {
        tests::r_bool_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_bool_does_not_read_past_capacity() {
        tests::r_bool_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_bool_result_is_not_mutated_on_subsequent_write() {
        tests::r_bool_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_bytes() tests
    #[test]
    fn r_bytes_returns_err_when_closed() {
        tests::r_bytes_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_bytes_returns_ok_when_open() {
        tests::r_bytes_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_bytes_reads_zeros_from_unwritten_storage() {
        tests::r_bytes_reads_zeros_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_bytes_reads_written_data() {
        tests::r_bytes_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_bytes_does_not_read_past_txn_boundary() {
        tests::r_bytes_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_bytes_does_not_read_past_capacity() {
        tests::r_bytes_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_bytes_result_is_not_mutated_on_subsequent_write() {
        tests::r_bytes_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_str() tests
    #[test]
    fn r_str_returns_err_when_closed() {
        tests::r_str_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_str_returns_ok_when_open() {
        tests::r_str_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_str_reads_nulls_from_unwritten_storage() {
        tests::r_str_reads_nulls_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_str_reads_written_data() {
        tests::r_str_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_str_does_not_read_past_txn_boundary() {
        tests::r_str_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_str_does_not_read_past_capacity() {
        tests::r_str_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_str_result_is_not_mutated_on_subsequent_write() {
        tests::r_str_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // fill() tests
    #[test]
    fn fill_returns_err_when_closed() {
        tests::fill_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn fill_does_not_write_when_closed() {
        tests::fill_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn fill_returns_ok_when_open() {
        tests::fill_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn fill_repeats_byte_in_storage_range() {
        tests::fill_repeats_byte_in_storage_range(
            get_storage()
        );
    }

    #[test]
    fn fill_starts_from_beginning_when_start_offset_is_none() {
        tests::fill_starts_from_beginning_when_start_offset_is_none(
            get_storage()
        );
    }

    #[test]
    fn fill_goes_to_end_when_end_offset_is_none() {
        tests::fill_goes_to_end_when_end_offset_is_none(
            get_storage()
        );
    }

    #[test]
    fn fill_returns_err_when_end_offset_is_before_start_offset() {
        tests::fill_returns_err_when_end_offset_is_before_start_offset(
            get_storage()
        );
    }

    #[test]
    fn fill_does_not_write_when_end_offset_is_before_start_offset() {
        tests::fill_does_not_write_when_end_offset_is_before_start_offset(
            get_storage()
        );
    }

    #[test]
    fn fill_returns_err_when_before_txn_boundary() {
        tests::fill_returns_err_when_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn fill_does_not_write_when_before_txn_boundary() {
        tests::fill_does_not_write_when_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn fill_returns_ok_when_after_txn_boundary() {
        tests::fill_returns_ok_when_after_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn fill_writes_when_after_txn_boundary() {
        tests::fill_writes_when_after_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn fill_returns_err_when_past_capacity() {
        tests::fill_returns_err_when_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn fill_does_not_write_when_past_capacity() {
        tests::fill_does_not_write_when_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn fill_does_not_expand_capacity() {
        tests::fill_does_not_expand_capacity(
            get_storage()
        );
    }

    // assert_filled() tests
    #[test]
    fn is_filled_retuns_err_when_closed() {
        tests::is_filled_retuns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn is_filled_returns_err_when_start_offset_past_capacity() {
        tests::is_filled_returns_err_when_start_offset_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn is_filled_returns_err_when_end_offset_at_or_before_start_offset() {
        tests::is_filled_returns_err_when_end_offset_at_or_before_start_offset(
            get_storage()
        );
    }

    #[test]
    fn is_filled_returns_err_when_end_offset_past_capacity() {
        tests::is_filled_returns_err_when_end_offset_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn is_filled_checks_whether_all_bytes_in_range_match_value() {
        tests::is_filled_checks_whether_all_bytes_in_range_match_value(
            get_storage()
        );
    }

    #[test]
    fn is_filled_starts_from_start_offset() {
        tests::is_filled_starts_from_start_offset(
            get_storage()
        );
    }

    #[test]
    fn is_filled_starts_from_beginning_when_start_offset_is_none() {
        tests::is_filled_starts_from_beginning_when_start_offset_is_none(
            get_storage()
        );
    }

    #[test]
    fn is_filled_goes_to_end_offset() {
        tests::is_filled_goes_to_end_offset(
            get_storage()
        );
    }

    #[test]
    fn is_filled_goes_to_end_when_end_offset_is_none() {
        tests::is_filled_goes_to_end_when_end_offset_is_none(
            get_storage()
        );
    }

    // get_use_txn_boundary(), set_use_txn_boundary(), get_txn_boundary(), and set_txn_boundary() tests
    #[test]
    fn set_use_txn_boundary_changes_value() {
        tests::set_use_txn_boundary_changes_value(
            get_storage()
        );
    }

    #[test]
    fn set_use_txn_boundary_resets_boundary_to_zero_when_txn_boundary_turned_off() {
        tests::set_use_txn_boundary_resets_boundary_to_zero_when_txn_boundary_turned_off(
            get_storage()
        );
    }

    #[test]
    fn get_txn_boundary_returns_err_when_closed() {
        tests::get_txn_boundary_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn get_txn_boundary_returns_err_when_not_using_txn_boundary() {
        tests::get_txn_boundary_returns_err_when_not_using_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn get_txn_boundary_starts_at_0() {
        tests::get_txn_boundary_starts_at_0(
            get_storage()
        );
    }

    #[test]
    fn set_txn_boundary_returns_err_when_not_using_txn_boundary() {
        tests::set_txn_boundary_returns_err_when_not_using_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn set_txn_boundary_does_not_change_boundary_when_not_using_txn_boundary() {
        tests::set_txn_boundary_does_not_change_boundary_when_not_using_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn set_txn_boundary_returns_err_when_closed() {
        tests::set_txn_boundary_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn set_txn_boundary_does_not_change_boundary_when_closed() {
        tests::set_txn_boundary_does_not_change_boundary_when_closed(
            get_storage()
        );
    }

    #[test]
    fn set_txn_boundary_returns_err_when_past_capacity() {
        tests::set_txn_boundary_returns_err_when_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn set_txn_boundary_does_not_change_boundary_when_past_capacity() {
        tests::set_txn_boundary_does_not_change_boundary_when_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn set_txn_boundary_does_not_expand_capacity_when_past_capacity() {
        tests::set_txn_boundary_does_not_expand_capacity_when_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn set_txn_boundary_changes_boundary() {
        tests::set_txn_boundary_changes_boundary(
            get_storage()
        );
    }

    // get_expand_size() and set_expand_size() tests
    #[test]
    fn get_expand_size_returns_initial_expand_size() {
        tests::get_expand_size_returns_initial_expand_size(
            get_storage()
        );
    }

    #[test]
    fn set_expand_size_returns_err_when_expand_size_is_zero() {
        tests::set_expand_size_returns_err_when_expand_size_is_zero(
            get_storage()
        );
    }

    #[test]
    fn set_expand_size_does_not_change_expand_size_when_expand_size_is_zero() {
        tests::set_expand_size_does_not_change_expand_size_when_expand_size_is_zero(
            get_storage()
        );
    }

    #[test]
    fn set_expand_size_returns_err_when_expand_size_is_not_power_of_2() {
        tests::set_expand_size_returns_err_when_expand_size_is_not_power_of_2(
            get_storage()
        );
    }

    #[test]
    fn set_expand_size_does_not_change_expand_size_when_expand_size_is_not_power_of_2() {
        tests::set_expand_size_does_not_change_expand_size_when_expand_size_is_not_power_of_2(
            get_storage()
        );
    }

    #[test]
    fn set_expand_size_returns_true_when_checks_pass() {
        tests::set_expand_size_returns_true_when_checks_pass(
            get_storage()
        );
    }

    #[test]
    fn set_expand_size_changes_expand_size_when_checks_pass() {
        tests::set_expand_size_changes_expand_size_when_checks_pass(
            get_storage()
        );
    }

    #[test]
    fn capacity_increases_to_increments_of_last_set_expand_size() {
        tests::capacity_increases_to_increments_of_last_set_expand_size(
            get_storage()
        );
    }

    // get_capacity() tests
    #[test]
    fn get_capacity_returns_err_when_closed() {
        tests::get_capacity_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn get_capacity_returns_initial_capacity_when_open() {
        tests::get_capacity_returns_initial_capacity_when_open(
            get_storage()
        );
    }

    #[test]
    fn get_capacity_returns_new_capacity_after_expansion() {
        tests::get_capacity_returns_new_capacity_after_expansion(
            get_storage()
        );
    }

    // expand() tests
    #[test]
    fn expand_returns_err_when_closed() {
        tests::expand_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn expand_does_not_change_capacity_when_closed() {
        tests::expand_does_not_change_capacity_when_closed(
            get_storage()
        );
    }

    #[test]
    fn expand_returns_ok_when_already_has_capacity() {
        tests::expand_returns_ok_when_already_has_capacity(
            get_storage()
        );
    }

    #[test]
    fn expand_does_not_change_capacity_when_already_has_capacity() {
        tests::expand_does_not_change_capacity_when_already_has_capacity(
            get_storage()
        );
    }

    #[test]
    fn expand_returns_err_when_allocation_arithmetic_overflows() {
        tests::expand_returns_err_when_allocation_arithmetic_overflows(
            get_storage()
        );
    }

    #[test]
    fn expand_does_not_change_capacity_when_allocation_arithmetic_overflows() {
        tests::expand_does_not_change_capacity_when_allocation_arithmetic_overflows(
            get_storage()
        );
    }

    #[test]
    fn expand_returns_err_when_allocation_fails() {
        tests::expand_returns_err_when_allocation_fails(
            get_storage()
        );
    }

    #[test]
    fn expand_does_not_change_capacity_when_allocation_fails() {
        tests::expand_does_not_change_capacity_when_allocation_fails(
            get_storage()
        );
    }

    #[test]
    fn expand_returns_ok_when_successful() {
        tests::expand_returns_ok_when_successful(
            get_storage()
        );
    }

    #[test]
    fn expand_changes_capacity_by_expand_size_when_successful() {
        tests::expand_changes_capacity_by_expand_size_when_successful(
            get_storage()
        );
    }

    #[test]
    fn expand_changes_capacity_by_multiples_of_expand_size_when_successful() {
        tests::expand_changes_capacity_by_multiples_of_expand_size_when_successful(
            get_storage()
        );
    }

}

