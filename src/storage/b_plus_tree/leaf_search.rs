use std::collections::VecDeque;
use storage::binary_storage::BinaryStorage;

const RECORDS_OFFSET: usize = 0;

pub struct LeafSearch<T: BinaryStorage + Sized> {
  key_len: u8,
  val_len: u8,
  ptr: u64,
  prev: u64,
  next: u64,
  parents: VecDeque<u64>,
  storage: T
}
impl<T: BinaryStorage + Sized> LeafSearch<T> {

  pub fn new(
    key_len: u8,
    val_len: u8,
    ptr: u64,
    prev: u64,
    next: u64
    parents: VecDeque<u64>,
    storage: T,
  ) -> LeafSearch<T> {
    LeafSearch {
      key_len: key_len,
      val_len: val_len,
      ptr: ptr,
      prev: prev,
      next: next,
      parents: parents,
      storage: storage
    }
  }

  pub fn get_at_index(&self, index: u32) {
    
  }

  fn get_ptr_at_index(&self, index: u32) -> u64 {
    RECORDS_OFFSET as u64 + index as u64 *  
  }


}

pub struct LeafRecord {
  key: Vec<u8>,
  val: Vec<u8>
}
