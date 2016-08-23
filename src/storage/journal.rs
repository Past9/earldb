extern crate alloc;
extern crate core;

use error::Error;

pub static ERR_WRITE_IN_PROGRESS: & 'static str =
    "Cannot perform this operation while an uncommitted write is in progress";
pub static ERR_WRITE_NOT_IN_PROGRESS: & 'static str =
    "Cannot perform this operation when no write is in progress";
pub static ERR_NOTHING_TO_WRITE: & 'static str =
    "Cannot write 0 bytes";
pub static ERR_NO_COMMITTED_RECORD: & 'static str =
    "Location is not the start of a committed record";

pub trait Journal {

    fn open(&mut self) -> Result<(), Error> ;
    fn close(&mut self) -> Result<(), Error> ;
    fn is_open(&self) -> bool;


    fn write(&mut self, data: &[u8]) -> Result<(), Error>;
    fn commit(&mut self) -> Result<(), Error>;
    fn discard(&mut self) -> Result<(), Error>;

    fn is_writing(&self) -> bool;

    fn reset(&mut self);
    fn has_start(&mut self) -> Result<bool, Error>;
    fn has_end(&mut self) -> Result<bool, Error>;
    fn read(&mut self) -> Result<Vec<u8>, Error>;
    fn jump_to(&mut self, offset: u64) -> Result<(), Error>;
    fn next(&mut self) -> Option<Vec<u8>>;

    fn read_offset(&self) -> u64;
    fn write_offset(&self) -> u64;

    fn capacity(&self) -> Result<u64, Error>;

    fn txn_boundary(&self) -> Result<u64, Error>;


}
