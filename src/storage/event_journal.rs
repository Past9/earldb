use std::mem;

use error::{ Error, AssertionError };
use storage::journal::Journal;
use storage::binary_storage::BinaryStorage;

pub static ERR_WRITE_IN_PROGRESS: & 'static str =
    "Cannot perform this operation while an uncommitted write is in progress";
pub static ERR_WRITE_NOT_IN_PROGRESS: & 'static str =
    "Cannot perform this operation when no write is in progress";
pub static ERR_NOTHING_TO_WRITE: & 'static str =
    "Cannot write 0 bytes";

pub struct EventJournal<T: BinaryStorage + Sized> {
    storage: T,
    read_offset: u64,
    write_offset: u64,
    is_writing: bool,
    uncommitted_size: u64
}
impl<T: BinaryStorage + Sized> EventJournal<T> {

    pub fn new(mut storage: T) -> EventJournal<T> {
        storage.set_use_txn_boundary(true);
        EventJournal {
            storage: storage,
            read_offset: 0,
            write_offset: 0,
            is_writing: false,
            uncommitted_size: 0
        }
    }

}
impl<T: BinaryStorage + Sized> Journal for EventJournal<T> {

    fn open(&mut self) -> Result<(), Error> {
        self.storage.open()
        // TODO: Verify data, set txn boundary
    }

    fn close(&mut self) -> Result<(), Error> {
        match self.storage.close() {
            Ok(()) => {
                self.is_writing = false;
                self.read_offset = 0;
                self.write_offset = 0;
                Ok(())
            },
            Err(e) => Err(e)
        }
    }

    fn is_open(&self) -> bool {
        self.storage.is_open()
    }

    fn reset(&mut self) {
        self.read_offset = 0;
    }

    fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        // TODO: constrain data size
        try!(AssertionError::assert_not(self.is_writing, ERR_WRITE_IN_PROGRESS));
        try!(AssertionError::assert(data.len() > 0, ERR_NOTHING_TO_WRITE));

        self.is_writing = true;
        let len = data.len();

        match self.storage.w_u8(self.write_offset, 0x02) {
            Ok(()) =>  {
                self.write_offset += mem::size_of::<u8>() as u64;
                self.uncommitted_size = mem::size_of::<u8>() as u64;
            },
            Err(e) => match self.discard() {
                Ok(()) => return Err(e),
                Err(d) => return Err(d)
            }
        };

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

        Ok(())
    }

    fn commit(&mut self) -> Result<(), Error> {
        try!(AssertionError::assert(self.is_writing, ERR_WRITE_NOT_IN_PROGRESS));

        match self.storage.w_u8(self.write_offset, 0x03) {
            Ok(()) =>  {
                self.write_offset += mem::size_of::<u8>() as u64;
                self.uncommitted_size += mem::size_of::<u8>() as u64;
            },
            Err(e) => match self.discard() {
                Ok(()) => return Err(e),
                Err(d) => return Err(d)
            }
        };

        match self.storage.set_txn_boundary(self.write_offset) {
            Ok(()) => {
                self.write_offset = 0;
                self.uncommitted_size = 0;
                self.is_writing = false;
            },
            Err(e) => match self.discard() {
                Ok(()) => return Err(e),
                Err(d) => return Err(d)
            }
        };

        unimplemented!();
    }

    fn discard(&mut self) -> Result<(), Error> {
        try!(AssertionError::assert(self.is_writing, ERR_WRITE_NOT_IN_PROGRESS));

        match self.storage.set_txn_boundary(self.write_offset) {
            Ok(()) => {
                self.write_offset -= self.uncommitted_size;
                self.uncommitted_size = 0;
                self.is_writing = false;
                Ok(())
            },
            Err(e) => Err(e)
        }
    }

    fn is_writing(&self) -> bool {
        self.is_writing
    }


    fn capacity(&self) -> Result<u64, Error> {
        self.storage.get_capacity()
    }

}
