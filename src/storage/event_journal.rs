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

        Ok(())

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

    fn reset(&mut self) {
        self.read_offset = 0;
    }

    fn has_start(&mut self) -> Result<bool, Error> {
        Ok(
            0x02 == try!(self.storage.r_u8(self.read_offset)) 
        )
    }

    fn has_end(&mut self) -> Result<bool, Error> {
        let len = try!(self.storage.r_u32(self.read_offset + mem::size_of::<u8>() as u64));
        Ok(
            0x03 == try!(self.storage.r_u8(
                self.read_offset + 
                    mem::size_of::<u8>() as u64 + 
                    mem::size_of::<u32>() as u64 + 
                    len as u64
            ))
        )
    }

    fn read(&mut self) -> Result<Vec<u8>, Error> {
        let len = try!(self.storage.r_u32(self.read_offset + mem::size_of::<u8>() as u64)) as usize;
        self.storage.r_bytes(
            self.read_offset + 
                mem::size_of::<u8>() as u64 + 
                mem::size_of::<u32>() as u64,
            len 
        )
    }

    fn jump_to(&mut self, offset: u64, back_on_fail: bool) -> Result<bool, Error> {
        let old_offset = self.read_offset;
        self.read_offset = offset;

        match self.has_start() {
            Ok(v) => {
                if !v {
                    self.read_offset = old_offset;
                    return Ok(v);
                }
            },
            Err(e) => {
                self.read_offset = old_offset;
                return Err(e);
            }
        };

        match self.has_end() {
            Ok(v) => {
                if !v {
                    self.read_offset = old_offset;
                    return Ok(v);
                }
            },
            Err(e) => {
                self.read_offset = old_offset;
                return Err(e);
            }
        };

        Ok(true)
    }

    fn next(&mut self) -> Result<Option<Vec<u8>>, Error> {

        if !try!(self.has_start()) || !try!(self.has_end()) { return Ok(None) }

        let res = try!(self.read());

        let new_offset = self.read_offset + 
            mem::size_of::<u8>() as u64 + 
            mem::size_of::<u32>() as u64 + 
            res.len() as u64 +
            mem::size_of::<u8>() as u64;

        try!(self.jump_to(new_offset, false));

        Ok(Some(res))
    }

    fn read_offset(&self) -> u64 {
        self.read_offset
    }

    fn write_offset(&self) -> u64 {
        self.write_offset
    }

    fn capacity(&self) -> Result<u64, Error> {
        self.storage.get_capacity()
    }

}


#[cfg(test)]
mod event_journal_tests {

    use std::error::Error;
    use storage::journal::Journal;
    use storage::event_journal::EventJournal;
    use storage::binary_storage;
    use storage::binary_storage::BinaryStorage;
    use storage::memory_binary_storage::MemoryBinaryStorage;

    // new() tests


    // open(), close(), an is_open() tests
    #[test]
    pub fn is_closed_by_default() {
        let j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        assert!(!j.is_open());
    }

    #[test]
    pub fn close_returns_err_when_already_closed() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            j.close().unwrap_err().description()
        );
    }

    #[test]
    pub fn open_returns_ok_when_previously_closed() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        assert!(j.open().is_ok());
    }

    #[test]
    pub fn open_returns_err_when_previously_open() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_OPEN,
            j.open().unwrap_err().description()
        );
    }

    #[test]
    pub fn close_returns_ok_when_previously_open() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        assert!(j.close().is_ok());
    }

    #[test]
    pub fn is_open_after_open() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        assert!(j.is_open());
    }

    #[test]
    pub fn is_closed_after_open_and_close() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.close().unwrap();
        assert!(!j.is_open());
    }

    // write(), commit(), and discard() tests
    // TODO: write these

    // is_writing() tests
    // TODO: write these

    // reset() tests
    // TODO: write these

    // has_start() tests
    // TODO: write these

    // has_end() tests
    // TODO: write these

    // read() tests
    // TODO: write these

    // jump_to() tests
    // TODO: write these

    // next() tests
    // TODO: write these

    // read_offset() tests
    // TODO: write these

    // write_offset() tests
    // TODO: write these

    // capacity() tests
    // TODO: write these



}
