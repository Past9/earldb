#![feature(alloc, heap_api)]

extern crate alloc;
extern crate core;

use alloc::heap;
use std::{mem, ptr, slice};


pub trait Journal {

    fn open(&mut self);
    fn close(&mut self);
    fn is_open(&self) -> bool;

    fn reset(&mut self);

    fn write(&mut self, data: &[u8]);
    fn commit(&mut self);
    fn discard(&mut self);

    fn next(&mut self);
    fn size(&self) -> Option<u32>;
    fn read(&self) -> Option<Vec<u8>>;
    fn is_complete(&self) -> bool;

    fn cap_bytes(&self) -> usize;

}
