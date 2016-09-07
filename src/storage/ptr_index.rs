use std::io::{ Cursor };
use std::ops::Index;

use byteorder::{ LittleEndian, ReadBytesExt, WriteBytesExt };

use error::{ Error, AssertionError };
use storage::journal::{ PRE_DATA_LEN, POST_DATA_LEN };
use storage::journal::Journal;
use storage::binary_storage::BinaryStorage;

pub static ERR_NOT_INDEXED: &'static str = "Record is not present in index";

pub struct PtrIndex<T: BinaryStorage + Sized> {
    journal: Journal<T>,
    record_count: u64
}
impl<T: BinaryStorage + Sized> PtrIndex<T> {

    pub fn new(mut journal: Journal<T>) -> PtrIndex<T> {
        PtrIndex {
            journal: journal,
            record_count: 0
        }
    }

    pub fn open(&mut self) -> Result<(), Error> {
        self.journal.open()
    }

    pub fn close(&mut self) -> Result<(), Error> {
        self.journal.close()
    }

    pub fn verify(&mut self) -> Result<(), Error> {
        self.journal.verify()
    }

    pub fn append(&mut self, value: u64) -> Result<(), Error> {
        let mut buf = vec![];
        try!(buf.write_u64::<LittleEndian>(value));
        self.journal
            .write(buf.as_slice())
            .and(self.journal.commit())
    }

    pub fn get(&mut self, n: u64) -> Result<u64, Error> {
        try!(AssertionError::assert(self.record_count > n, ERR_NOT_INDEXED));

        try!(self.journal.jump_to(
            (PRE_DATA_LEN + 16 + POST_DATA_LEN) * n
        ));

        match self.journal.read() {
            Ok(data) => {
                let mut rdr = Cursor::new(data);
                match rdr.read_u64::<LittleEndian>() {
                    Ok(v) => Ok(v),
                    Err(e) => Err(Error::Io(e))
                }
            },
            Err(e) => Err(e)
        }
    }

    pub fn record_count(&self) -> u64 {
        self.record_count
    }

}
impl<T: BinaryStorage + Sized> Iterator for PtrIndex<T> {

    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        match self.journal.next() {
            Some(data) => {
                let mut rdr = Cursor::new(data);
                match rdr.read_u64::<LittleEndian>() {
                    Ok(v) => Some(v),
                    Err(_) => None
                }
            },
            None => None
        }
    }

}
