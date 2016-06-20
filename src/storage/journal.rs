extern crate alloc;
extern crate core;

use error::Error;

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
    fn jump_to(&mut self, offset: u64, back_on_fail: bool) -> Result<bool, Error>;
    fn next(&mut self) -> Result<Option<Vec<u8>>, Error>;

    fn read_offset(&self) -> u64;
    fn write_offset(&self) -> u64;

    fn capacity(&self) -> Result<u64, Error>;

}
