use std::mem::size_of;
use error::{ Error, AssertionError };
use storage::binary_storage;
use storage::binary_storage::BinaryStorage;

pub static ERR_WRITE_BEFORE_TXN_BOUNDARY: & 'static str = 
    "Cannot write before transaction boundary";
pub static ERR_READ_AFTER_TXN_BOUNDARY: & 'static str = 
    "Cannot read after transaction boundary";
pub static ERR_SET_TXN_BOUNDARY_PAST_END: & 'static str = 
    "Cannot set transaction boundary past end of allocated storage";

pub struct TransactionalStorage<T: BinaryStorage + Sized> {
    storage: T,
    txn_boundary: u64,
    check_on_read: bool
}
impl<T: BinaryStorage + Sized> TransactionalStorage<T> {

    pub fn new(mut storage: T) -> TransactionalStorage<T> {
        TransactionalStorage {
            storage: storage,
            txn_boundary: 0,
            check_on_read: true
        }
    }

    fn check_boundary_for_read(&self, offset: u64, len: usize) -> Result<(), Error> {
        Ok(try!(AssertionError::assert(
            !self.storage.is_open() || 
            !self.check_on_read || 
            offset + (len as u64) <= self.txn_boundary,
            ERR_READ_AFTER_TXN_BOUNDARY
        )))
    }

    fn check_boundary_for_write(&self, offset: u64) -> Result<(), Error> {
        Ok(try!(AssertionError::assert(
            !self.storage.is_open() || 
            offset >= self.txn_boundary,
            ERR_WRITE_BEFORE_TXN_BOUNDARY
        )))
    }

    pub fn get_txn_boundary(&self) -> Result<u64, Error> {
        try!(AssertionError::assert(
            self.is_open(), 
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED
        ));
        Ok(self.txn_boundary)
    }

    pub fn set_txn_boundary(&mut self, offset: u64) -> Result<(), Error> {
        try!(AssertionError::assert(
            self.is_open(), 
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED
        ));
        try!(AssertionError::assert(
            offset <= try!(self.storage.get_capacity()),
            ERR_SET_TXN_BOUNDARY_PAST_END
        ));
        self.txn_boundary = offset;
        Ok(())
    }

    pub fn get_check_on_read(&self) -> Result<bool, Error> {
        try!(AssertionError::assert(
            self.is_open(), 
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED
        ));
        Ok(self.check_on_read)
    }

    pub fn set_check_on_read(&mut self, check_on_read: bool) -> Result<(), Error> {
        try!(AssertionError::assert(
            self.is_open(), 
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED
        ));
        self.check_on_read = check_on_read;
        Ok(())
    }

}
impl<T: BinaryStorage + Sized> BinaryStorage for TransactionalStorage<T> {

    fn open(&mut self) -> Result<(), Error> {
        self.storage.open()
    }

    fn close(&mut self) -> Result<(), Error> {
        self.storage.close()
    }


    fn is_open(&self) -> bool {
        self.storage.is_open()
    }


    fn w_i8(&mut self, offset: u64, data: i8) -> Result<(), Error> {
        try!(self.check_boundary_for_write(offset));
        self.storage.w_i8(offset, data)
    }

    fn w_i16(&mut self, offset: u64, data: i16) -> Result<(), Error> {
        try!(self.check_boundary_for_write(offset));
        self.storage.w_i16(offset, data)
    }

    fn w_i32(&mut self, offset: u64, data: i32) -> Result<(), Error> {
        try!(self.check_boundary_for_write(offset));
        self.storage.w_i32(offset, data)
    }

    fn w_i64(&mut self, offset: u64, data: i64) -> Result<(), Error> {
        try!(self.check_boundary_for_write(offset));
        self.storage.w_i64(offset, data)
    }


    fn w_u8(&mut self, offset: u64, data: u8) -> Result<(), Error> {
        try!(self.check_boundary_for_write(offset));
        self.storage.w_u8(offset, data)
    }

    fn w_u16(&mut self, offset: u64, data: u16) -> Result<(), Error> {
        try!(self.check_boundary_for_write(offset));
        self.storage.w_u16(offset, data)
    }

    fn w_u32(&mut self, offset: u64, data: u32) -> Result<(), Error> {
        try!(self.check_boundary_for_write(offset));
        self.storage.w_u32(offset, data)
    }

    fn w_u64(&mut self, offset: u64, data: u64) -> Result<(), Error> {
        try!(self.check_boundary_for_write(offset));
        self.storage.w_u64(offset, data)
    }


    fn w_f32(&mut self, offset: u64, data: f32) -> Result<(), Error> {
        try!(self.check_boundary_for_write(offset));
        self.storage.w_f32(offset, data)
    }

    fn w_f64(&mut self, offset: u64, data: f64) -> Result<(), Error> {
        try!(self.check_boundary_for_write(offset));
        self.storage.w_f64(offset, data)
    }


    fn w_bool(&mut self, offset: u64, data: bool) -> Result<(), Error> {
        try!(self.check_boundary_for_write(offset));
        self.storage.w_bool(offset, data)
    }


    fn w_bytes(&mut self, offset: u64, data: &[u8]) -> Result<(), Error> {
        try!(self.check_boundary_for_write(offset));
        self.storage.w_bytes(offset, data)
    }

    fn w_str(&mut self, offset: u64, data: &str) -> Result<(), Error> {
        try!(self.check_boundary_for_write(offset));
        self.storage.w_str(offset, data)
    }



    fn r_i8(&mut self, offset: u64) -> Result<i8, Error> {
        try!(self.check_boundary_for_read(offset, size_of::<i8>()));
        self.storage.r_i8(offset)
    }

    fn r_i16(&mut self, offset: u64) -> Result<i16, Error> {
        try!(self.check_boundary_for_read(offset, size_of::<i16>()));
        self.storage.r_i16(offset)
    }

    fn r_i32(&mut self, offset: u64) -> Result<i32, Error> {
        try!(self.check_boundary_for_read(offset, size_of::<i32>()));
        self.storage.r_i32(offset)
    }

    fn r_i64(&mut self, offset: u64) -> Result<i64, Error> {
        try!(self.check_boundary_for_read(offset, size_of::<i64>()));
        self.storage.r_i64(offset)
    }


    fn r_u8(&mut self, offset: u64) -> Result<u8, Error> {
        try!(self.check_boundary_for_read(offset, size_of::<u8>()));
        self.storage.r_u8(offset)
    }

    fn r_u16(&mut self, offset: u64) -> Result<u16, Error> {
        try!(self.check_boundary_for_read(offset, size_of::<u16>()));
        self.storage.r_u16(offset)
    }

    fn r_u32(&mut self, offset: u64) -> Result<u32, Error> {
        try!(self.check_boundary_for_read(offset, size_of::<u32>()));
        self.storage.r_u32(offset)
    }

    fn r_u64(&mut self, offset: u64) -> Result<u64, Error> {
        try!(self.check_boundary_for_read(offset, size_of::<u64>()));
        self.storage.r_u64(offset)
    }


    fn r_f32(&mut self, offset: u64) -> Result<f32, Error> {
        try!(self.check_boundary_for_read(offset, size_of::<f32>()));
        self.storage.r_f32(offset)
    }

    fn r_f64(&mut self, offset: u64) -> Result<f64, Error> {
        try!(self.check_boundary_for_read(offset, size_of::<f64>()));
        self.storage.r_f64(offset)
    }


    fn r_bool(&mut self, offset: u64) -> Result<bool, Error> {
        try!(self.check_boundary_for_read(offset, size_of::<bool>()));
        self.storage.r_bool(offset)
    }


    fn r_bytes(&mut self, offset: u64, len: usize) -> Result<Vec<u8>, Error> {
        try!(self.check_boundary_for_read(offset, len));
        self.storage.r_bytes(offset, len)
    }

    fn r_str(&mut self, offset: u64, len: usize) -> Result<String, Error> {
        try!(self.check_boundary_for_read(offset, len));
        self.storage.r_str(offset, len)
    }


    fn fill(&mut self, start: Option<u64>, end: Option<u64>, val: u8) -> Result<(), Error> {
        match start {
            None => try!(self.check_boundary_for_write(0)),
            Some(s) => try!(self.check_boundary_for_write(s))
        };

        match end {
            None => try!(self.check_boundary_for_write(try!(self.storage.get_capacity()))),
            Some(e) => try!(self.check_boundary_for_write(e))
        };

        self.storage.fill(start, end, val)
    }

    fn is_filled(&mut self, start: Option<u64>, end: Option<u64>, val: u8) -> Result<bool, Error> {
        self.storage.is_filled(start, end, val)
    }


    fn get_expand_size(&self) -> u64 {
        self.storage.get_expand_size()
    }

    fn set_expand_size(&mut self, expand_size: u64) -> Result<(), Error> {
        self.storage.set_expand_size(expand_size)
    }


    fn get_capacity(&self) -> Result<u64, Error> {
        self.storage.get_capacity()
    }


    fn expand(&mut self, min_capacity: u64) -> Result<(), Error> {
        self.storage.expand(min_capacity)
    }


}
