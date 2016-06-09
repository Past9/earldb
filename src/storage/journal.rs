/*
#![feature(alloc, heap_api)]

extern crate alloc;
extern crate core;

use alloc::heap;
use std::{mem, ptr, slice};


pub trait Journal {

    fn open(&mut self);
    fn close(&mut self);
    fn is_open(&self) -> bool;

    fn reset(&mut self) -> bool;

    fn write(&mut self, data: &[u8]) -> bool;
    fn commit(&mut self) -> bool;
    fn discard(&mut self) -> bool;

    //fn next(&mut self) -> bool;
    fn size(&self) -> u32;
    //fn read(&self) -> Option<Vec<u8>>;
    fn is_writing(&self) -> bool;

    fn capacity(&self) -> usize;

}
*/
