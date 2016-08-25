#![feature(alloc, heap_api)]

extern crate alloc;
extern crate core;

extern crate uuid;
extern crate byteorder;

mod storage;
mod error;

#[cfg(test)]
mod test;
