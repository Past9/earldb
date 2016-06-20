extern crate alloc;
extern crate core;

use error::Error;

pub trait Journal {

    fn open(&mut self) -> Result<(), Error> ;
    fn close(&mut self) -> Result<(), Error> ;
    fn is_open(&self) -> bool;

    fn reset(&mut self);

    fn write(&mut self, data: &[u8]) -> Result<(), Error>;
    fn commit(&mut self) -> Result<(), Error>;
    fn discard(&mut self) -> Result<(), Error>;

    fn is_writing(&self) -> bool;

    fn capacity(&self) -> Result<u64, Error>;

}
