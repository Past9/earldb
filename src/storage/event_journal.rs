use std::mem;

use error::{ Error, AssertionError };
use storage::journal;
use storage::journal::Journal;
use storage::binary_storage::BinaryStorage;

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
        self.storage.close()
    }

    fn is_open(&self) -> bool {
        self.storage.is_open()
    }

    fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        // TODO: constrain data size
        try!(AssertionError::assert_not(self.is_writing, journal::ERR_WRITE_IN_PROGRESS));
        try!(AssertionError::assert(data.len() > 0, journal::ERR_NOTHING_TO_WRITE));

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
        try!(AssertionError::assert(self.is_writing, journal::ERR_WRITE_NOT_IN_PROGRESS));

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
        try!(AssertionError::assert(self.is_writing, journal::ERR_WRITE_NOT_IN_PROGRESS));

        match self.storage.set_txn_boundary(self.write_offset - self.uncommitted_size) {
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

    fn jump_to(&mut self, offset: u64) -> Result<(), Error> {
        self.read_offset = offset;

        match self.has_start() {
            Ok(v) => {
                if !v {
                    return Err(Error::from(AssertionError::new(journal::ERR_NO_COMMITTED_RECORD)));
                }
            },
            Err(e) => {
                return Err(e);
            }
        };

        match self.has_end() {
            Ok(v) => {
                if !v {
                    return Err(Error::from(AssertionError::new(journal::ERR_NO_COMMITTED_RECORD)));
                }
            },
            Err(e) => {
                return Err(e);
            }
        };

        Ok(())
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

    fn txn_boundary(&self) -> Result<u64, Error> {
        self.storage.get_txn_boundary()
    }

}
impl<T: BinaryStorage + Sized> Iterator for EventJournal<T> {

    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Vec<u8>> {

        match self.has_start() {
            Ok(h) => if !h { return None },
            Err(_) => return None
        };

        match self.has_end() {
            Ok(h) => if !h { return None },
            Err(_) => return None
        };

        match self.read() {
            Ok(v) => {

                let new_offset = self.read_offset + 
                    mem::size_of::<u8>() as u64 + 
                    mem::size_of::<u32>() as u64 + 
                    v.len() as u64 +
                    mem::size_of::<u8>() as u64;

                self.jump_to(new_offset);

                Some(v)

            },
            Err(_) => None
        }

    }
}



#[cfg(test)]
mod event_journal_tests {

    use std::error::Error;
    use storage::journal;
    use storage::journal::Journal;
    use storage::event_journal;
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
    #[test]
    pub fn write_returns_err_when_closed() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            j.write(&[0x0, 0x1, 0x2]).unwrap_err().description()
        );
    }

    #[test]
    pub fn write_returns_ok_when_open() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        assert!(j.write(&[0x0, 0x1, 0x2]).is_ok());
    }

    #[test]
    pub fn write_returns_err_when_uncommitted_data() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        assert_eq!(
            journal::ERR_WRITE_IN_PROGRESS,
            j.write(&[0x0, 0x1, 0x2]).unwrap_err().description()
        );
    }

    #[test]
    pub fn write_returns_ok_after_commit() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        assert!(j.write(&[0x0, 0x1, 0x2]).is_ok());
    }

    #[test]
    pub fn commit_returns_err_when_closed() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.close().unwrap();
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            j.commit().unwrap_err().description()
        );
    }

    #[test]
    pub fn commit_returns_err_when_no_data() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        assert_eq!(
            journal::ERR_WRITE_NOT_IN_PROGRESS,
            j.commit().unwrap_err().description()
        );
    }

    #[test]
    pub fn commit_returns_ok_when_uncommitted_data() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        assert!(j.commit().is_ok());
    }

    #[test]
    pub fn discard_returns_err_when_closed() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.close().unwrap();
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            j.commit().unwrap_err().description()
        );
    }

    #[test]
    pub fn discard_returns_ok_when_uncommitted_data() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        assert!(j.discard().is_ok());
    }

    #[test]
    pub fn discard_returns_err_when_no_data() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        assert_eq!(
            journal::ERR_WRITE_NOT_IN_PROGRESS,
            j.discard().unwrap_err().description()
        );
    }

    #[test]
    pub fn write_returns_ok_after_discard() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.discard().unwrap();
        assert!(j.write(&[0x0, 0x1, 0x2]).is_ok());
    }


    // is_writing() tests
    #[test]
    pub fn is_not_writing_when_new() {
        let j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        assert!(!j.is_writing());
    }

    #[test]
    pub fn is_not_writing_when_newly_opened() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        assert!(!j.is_writing());
    }

    #[test]
    pub fn is_writing_after_write() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        assert!(j.is_writing());
    }

    #[test]
    pub fn is_still_writing_when_closed() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.close().unwrap();
        assert!(j.is_writing());
    }

    #[test]
    pub fn is_still_writing_when_reopened() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.close().unwrap();
        j.open().unwrap();
        assert!(j.is_writing());
    }

    #[test]
    pub fn is_not_writing_after_commit() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        assert!(!j.is_writing());
    }

    #[test]
    pub fn is_not_writing_after_discard() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.discard().unwrap();
        assert!(!j.is_writing());
    }

    // has_start() tests
    #[test]
    pub fn has_start_returns_err_when_closed() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            j.has_start().unwrap_err().description()
        );
    }

    #[test]
    pub fn has_start_returns_err_when_past_txn_boundary() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            j.has_start().unwrap_err().description()
        );
    }

    #[test]
    pub fn has_start_returns_ok_when_open() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        assert!(j.has_start().is_ok());
    }

    #[test]
    pub fn has_start_returns_true_when_record_exists() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        assert!(j.has_start().unwrap());
    }

    // has_end() tests
    #[test]
    pub fn has_end_returns_err_when_closed() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            j.has_end().unwrap_err().description()
        );
    }

    #[test]
    pub fn has_end_returns_err_when_past_txn_boundary() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            j.has_end().unwrap_err().description()
        );
    }

    #[test]
    pub fn has_end_returns_true_when_record_is_committed() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        assert!(j.has_end().is_ok());
    }

    // read() tests
    #[test]
    pub fn read_returns_err_when_closed() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            j.read().unwrap_err().description()
        );
    }

    #[test]
    pub fn read_returns_err_when_no_data() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            j.read().unwrap_err().description()
        );
    }

    #[test]
    pub fn read_returns_err_when_uncommitted_data() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            j.read().unwrap_err().description()
        );
    }

    #[test]
    pub fn read_returns_ok_when_committed_data() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        assert!(j.read().is_ok());
    }

    #[test]
    pub fn read_returns_first_record() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        assert_eq!(vec!(0x0, 0x1, 0x2), j.read().unwrap());
    }

    #[test]
    pub fn read_returns_record_multiple_times() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        assert_eq!(vec!(0x0, 0x1, 0x2), j.read().unwrap());
        assert_eq!(vec!(0x0, 0x1, 0x2), j.read().unwrap());
    }

    // jump_to() tests
    #[test]
    pub fn jump_to_returns_err_when_closed() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        assert_eq!(
            binary_storage::ERR_OPERATION_INVALID_WHEN_CLOSED,
            j.jump_to(6).unwrap_err().description()
        );
    }

    #[test]
    pub fn jump_to_returns_err_when_past_txn_boundary() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            j.jump_to(6).unwrap_err().description()
        );
    }

    #[test]
    pub fn jump_to_returns_ok_when_at_record_start() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        j.write(&[0x3, 0x4, 0x5]).unwrap();
        j.commit().unwrap();
        assert!(j.jump_to(9).is_ok());
    }

    #[test]
    pub fn jump_to_returns_err_when_not_at_record_start() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        j.write(&[0x3, 0x4, 0x5]).unwrap();
        j.commit().unwrap();
        assert_eq!(
            journal::ERR_NO_COMMITTED_RECORD,
            j.jump_to(8).unwrap_err().description()
        );
    }

    #[test]
    pub fn jump_to_returns_err_when_at_uncommitted_record_start() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        j.write(&[0x3, 0x4, 0x5]).unwrap();
        assert_eq!(
            binary_storage::ERR_READ_AFTER_TXN_BOUNDARY,
            j.jump_to(9).unwrap_err().description()
        );
    }

    #[test]
    pub fn jump_to_still_jumps_when_err() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        j.write(&[0x3, 0x4, 0x5]).unwrap();
        j.jump_to(9).unwrap_err();
        assert_eq!(9, j.read_offset());
    }

    #[test]
    pub fn jump_to_jumps_when_complete_record() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        j.write(&[0x3, 0x4, 0x5]).unwrap();
        j.commit().unwrap();
        j.jump_to(9).unwrap();
        assert_eq!(9, j.read_offset());
    }

    #[test]
    pub fn jump_to_allows_record_read_at_jump_location() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        assert_eq!(vec!(0x0, 0x1, 0x2), j.read().unwrap());
        j.write(&[0x3, 0x4, 0x5]).unwrap();
        j.commit().unwrap();
        j.jump_to(9).unwrap();
        assert_eq!(vec!(0x3, 0x4, 0x5), j.read().unwrap());
    }

    // reset() tests
    #[test]
    pub fn reset_does_not_change_read_offset_when_already_0() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        assert_eq!(0, j.read_offset());
        j.reset();
        assert_eq!(0, j.read_offset());
    }

    #[test]
    pub fn reset_changes_read_offset_to_0() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        assert_eq!(0, j.read_offset());
        j.write(&[0x3, 0x4, 0x5]).unwrap();
        j.commit().unwrap();
        j.jump_to(9).unwrap();
        assert_eq!(9, j.read_offset());
        j.reset();
        assert_eq!(0, j.read_offset());
    }

    #[test]
    pub fn reset_allows_reading_from_first_record() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        assert_eq!(vec!(0x0, 0x1, 0x2), j.read().unwrap());
        j.write(&[0x3, 0x4, 0x5]).unwrap();
        j.commit().unwrap();
        j.jump_to(9).unwrap();
        assert_eq!(vec!(0x3, 0x4, 0x5), j.read().unwrap());
        j.reset();
        assert_eq!(vec!(0x0, 0x1, 0x2), j.read().unwrap());
    }

    // next() tests
    #[test]
    pub fn next_returns_none_when_closed() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        assert!(j.next().is_none());
    }

    #[test]
    pub fn next_returns_none_when_no_records() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        assert!(j.next().is_none());
    }

    #[test]
    pub fn next_returns_records_in_order() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        assert_eq!(vec!(0x0, 0x1, 0x2), j.next().unwrap());
        j.write(&[0x3, 0x4, 0x5]).unwrap();
        j.commit().unwrap();
        assert_eq!(vec!(0x3, 0x4, 0x5), j.next().unwrap());
    }

    #[test]
    pub fn next_returns_none_when_no_more_records_available() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        assert_eq!(vec!(0x0, 0x1, 0x2), j.next().unwrap());
        assert!(j.next().is_none());
    }

    #[test]
    pub fn next_returns_records_as_they_become_available() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        assert_eq!(vec!(0x0, 0x1, 0x2), j.next().unwrap());
        assert!(j.next().is_none());
        j.write(&[0x4, 0x5, 0x6]).unwrap();
        j.commit().unwrap();
        assert_eq!(vec!(0x4, 0x5, 0x6), j.next().unwrap());
        assert!(j.next().is_none());
    }

    // read_offset() tests
    #[test]
    pub fn read_offset_starts_at_0() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        assert_eq!(0, j.read_offset());
    }

    #[test]
    pub fn read_offset_moves_on_next() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        assert_eq!(0, j.read_offset());
        assert_eq!(vec!(0x0, 0x1, 0x2), j.next().unwrap());
        assert_eq!(9, j.read_offset());
    }

    #[test]
    pub fn read_offset_retains_position_after_reopening() {
        let mut j = EventJournal::new(MemoryBinaryStorage::new(256, 256, false).unwrap());
        j.open().unwrap();
        j.write(&[0x0, 0x1, 0x2]).unwrap();
        j.commit().unwrap();
        assert_eq!(vec!(0x0, 0x1, 0x2), j.next().unwrap());
        assert_eq!(9, j.read_offset());
        j.close().unwrap();
        j.open().unwrap();
        assert_eq!(9, j.read_offset());
    }

    // write_offset() tests
    // TODO: write these

    // capacity() tests
    // TODO: write these

    // txn_boundary() tests
    // TODO: write these


}
