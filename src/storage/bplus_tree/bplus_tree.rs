use core::clone::Clone;
use byteorder::{ LittleEndian, ReadBytesExt, WriteBytesExt };

use error::{ Error, AssertionError };
use storage::binary_storage::BinaryStorage;

pub static ERR_USE_LEAF_WHERE_NONE: & 'static str = 
  "Tried to read leaf node from file location where none exists";
pub static ERR_USE_INNER_WHERE_NONE: & 'static str = 
  "Tried to read inner node from file location where none exists";
pub static ERR_KEY_WRONG_SIZE: & 'static str = 
  "Key is the wrong nubmer of bytes";
pub static ERR_READ_PAST_INNER_NODE: & 'static str = 
  "Tried to read more records from inner node than exist in the node";
pub static ERR_INNER_NODE_EMPTY: & 'static str = 
  "Encountered an inner node with no records";
pub static ERR_SEARCH_NO_LEAF_FOR_KEY: & 'static str = 
  "Search could not find a leaf node for the key";
pub static ERR_INVALID_NODE_TYPE: & 'static str = 
  "Node is not marked as either an inner node or a leaf node";

const INNER_NODE_REC_OFFSET: u32 = 13;
const LEAF_NODE_REC_OFFSET: u32 = 29;

struct LeafRecord {
  pub leaf_idx: u32,
  pub key: Vec<u8>,
  pub val: Vec<u8>
}

struct InnerRecord {
  pub min_key: Option<Vec<u8>>,
  pub ptr: u64,
  pub max_key: Option<Vec<u8>>
}

#[derive(Clone)]
struct InnerState {
  pub ptr: u64,
  pub parent_ptr: u64,
  pub num_recs: u32,
  pub cur_rec_idx: u32
}

#[derive(Clone)]
struct LeafState {
  pub ptr: u64,
  pub parent_ptr: u64,
  pub prev_ptr: u64,
  pub next_ptr: u64,
  pub num_recs: u32,
  pub cur_rec_idx: u32,
}

enum State {
  Nothing(),
  Inner(InnerState),
  Leaf(LeafState)
}


pub struct BPlusTree<T: BinaryStorage + Sized> {
  storage: T,
  key_len: u8,
  val_len: u8,
  node_size: u32,
  state: State,
  num_nodes: u64
}
impl<T: BinaryStorage + Sized> BPlusTree<T> {

  pub fn new(
    storage: T,
    key_len: u8,
    val_len: u8,
    node_size: u32
  ) -> BPlusTree<T> {
    BPlusTree {
      storage: storage,
      key_len: key_len,
      val_len: val_len,
      node_size: node_size,
      state: State::Nothing(),
      num_nodes: 0
    }
  }

  pub fn open(&mut self) -> Result<(), Error> {
    self.storage.open()
    // TODO: Ensure object properties match saved file data
  }

  pub fn close(&mut self) -> Result<(), Error> {
    self.storage.close()
  }

  pub fn insert(&mut self, key: &[u8], val: &[u8]) -> Result<(), Error> {
    try!(self.search_node(key));

    match try!(self.leaf_is_full()) {
      true => try!(self.split_leaf(key, val)),
      false => try!(self.insert_in_leaf(key, val))
    };

    Ok(())
  }

  fn alloc_leaf(&mut self, prev_ptr: u64, parent_ptr: u64) -> Result<u64, Error> {

    let ptr = self.num_nodes * self.node_size as u64;

    try!(self.storage.w_u8(ptr, 0x02)); // Leaf node marker
    try!(self.storage.w_u64(ptr + 1, parent_ptr)); // Pointer to parent node
    try!(self.storage.w_u64(ptr + 9, prev_ptr)); // Pointer to previous leaf node
    try!(self.storage.w_u64(ptr + 17, 0)); // Pointer to next leaf node
    try!(self.storage.w_u32(ptr + 25, 0)); // Number of records in this node 

    self.num_nodes += 1;

    Ok(ptr)
  }

  fn split_leaf(&mut self, key: &[u8], val: &[u8]) -> Result<(), Error> {
    let l = match self.state {
      State::Leaf(ref l) => l.clone(),
      _ => { return Err(Error::Assertion(AssertionError::new(ERR_USE_LEAF_WHERE_NONE))); }
    };

    let split_idx = l.num_recs / 2;
    let new_leaf_ptr = try!(self.alloc_leaf(l.ptr, l.parent_ptr));

    let split_offset = Self::leaf_rec_offset(split_idx, self.key_len, self.val_len) as u64;
    let len_to_copy = self.node_size as u64 - split_offset;

    let bytes_to_copy = try!(self.storage.r_bytes(l.ptr + split_offset, len_to_copy as usize)); 
    try!(self.storage.w_bytes(new_leaf_ptr + LEAF_NODE_REC_OFFSET as u64, bytes_to_copy.as_slice()));
    try!(self.storage.fill(Some(l.ptr + split_offset), Some(len_to_copy), 0x0));

    try!(self.enter_node(l.parent_ptr)); 
    self.insert_in_inner(key, new_leaf_ptr);

    Ok(())
  }

  fn insert_in_inner(&mut self, key: &[u8], ptr: u64) -> Result<(), Error> {
    let i = match self.state {
      State::Inner(ref i) => i.clone(),
      _ => { return Err(Error::Assertion(AssertionError::new(ERR_USE_INNER_WHERE_NONE))); }
    };

    match try!(self.inner_is_full()) {
      true => try!(self.split_inner(key, ptr)),
      false => {

        let mut idx = 0;
        let mut stop = false;

        while !stop {

          match try!(self.next_inner_rec()) {
            Some(r) => {
              match (r.min_key, r.max_key) {
                (None, Some(max)) => {
                  if key < max.as_slice() {
                    try!(self.insert_in_inner_at_idx(idx, key, ptr));
                  }
                },
                (Some(min), Some(max)) => {
                  if key == min.as_slice() {
                    try!(self.overwrite_in_inner_at_idx(idx, key, ptr));
                  } else if min.as_slice() < key && key < max.as_slice() {
                    try!(self.insert_in_inner_at_idx(idx, key, ptr));
                  }
                },
                (Some(min), None) => {
                  if key == min.as_slice() {
                    try!(self.overwrite_in_inner_at_idx(idx, key, ptr));
                  } else if min.as_slice() < key {
                    try!(self.insert_in_inner_at_idx(idx, key, ptr));
                  }
                },
                (None, None) => {
                  if idx == 0 {
                    try!(self.insert_in_inner_at_idx(idx, key, ptr));
                  }
                  stop = true;
                }
              }
            },
            None => try!(self.insert_in_inner_at_idx(idx, key, ptr))
          };

          idx += 1;
        }

      }
    };

    Ok(())
  }

  fn insert_in_inner_at_idx(&mut self, idx: u32, key: &[u8], ptr: u64) -> Result<(), Error> {
    let rec_size = Self::inner_rec_size(self.key_len) as u64;
    let rec_offset = Self::inner_rec_offset(idx, self.key_len) as u64;
    let len_to_move = self.node_size as u64 - rec_offset - rec_size;

    let bytes_to_move = try!(self.storage.r_bytes(rec_offset, len_to_move as usize));
    try!(self.storage.w_bytes(rec_offset + rec_size, bytes_to_move.as_slice()));
    try!(self.overwrite_in_inner_at_idx(idx, key, ptr));
     Ok(())
  }

  fn overwrite_in_inner_at_idx(&mut self, idx: u32, key: &[u8], ptr: u64) -> Result<(), Error> {
    let rec_offset = Self::inner_rec_offset(idx, self.key_len) as u64;
    try!(self.storage.w_bytes(rec_offset, key));
    try!(self.storage.w_u64(rec_offset + self.key_len as u64, ptr));
    Ok(())
  }

  fn alloc_inner(&mut self, parent_ptr: u64) -> Result<u64, Error> {

    let ptr = self.num_nodes * self.node_size as u64;

    try!(self.storage.w_u8(ptr, 0x01)); // Inner node marker
    try!(self.storage.w_u64(ptr + 1, parent_ptr)); // Pointer to parent node
    try!(self.storage.w_u32(ptr + 9, 0)); // Number of records in this node 

    self.num_nodes += 1;

    Ok(ptr)
  }

  fn split_inner(&mut self, key: &[u8], ptr: u64) -> Result<(), Error> {
    let i = match self.state {
      State::Inner(ref i) => i.clone(),
      _ => { return Err(Error::Assertion(AssertionError::new(ERR_USE_INNER_WHERE_NONE))); }
    };

    let split_idx = i.num_recs / 2;
    let new_inner_ptr = try!(self.alloc_inner(i.parent_ptr));
    
    let split_offset = Self::inner_rec_offset(split_idx, self.key_len) as u64;
    let len_to_copy = self.node_size as u64 - split_offset;

    let bytes_to_copy = try!(self.storage.r_bytes(i.ptr + split_offset, len_to_copy as usize));
    try!(self.storage.w_bytes(new_inner_ptr + INNER_NODE_REC_OFFSET as u64, bytes_to_copy.as_slice()));
    try!(self.storage.fill(Some(i.ptr + split_offset), Some(len_to_copy), 0x0));

    try!(self.enter_node(i.parent_ptr));
    self.insert_in_inner(key, new_inner_ptr);

    Ok(())
  }

  fn insert_in_leaf(&mut self, key: &[u8], val: &[u8]) -> Result<(), Error> {

    let mut i = 0;
    let mut min_key: Option<Vec<u8>> = None;
    let mut max_key: Option<Vec<u8>> = None;
    let mut stop = false;

    while !stop {

      max_key = match try!(self.next_leaf_rec()) {
        Some(r) => Some(r.key),
        None => None
      };

      match (min_key.clone(), max_key.clone()) {
        (None, Some(max)) => {
          if key < max.as_slice() {
            try!(self.insert_in_leaf_at_idx(i, key, val));
          }
        },
        (Some(min), Some(max)) => {
          if key == min.as_slice() {
            try!(self.overwrite_in_leaf_at_idx(i, key, val));
          } else if min.as_slice() < key && key < max.as_slice() {
            try!(self.insert_in_leaf_at_idx(i, key, val));
          }
        },
        (Some(min), None) => {
          if key == min.as_slice() {
            try!(self.overwrite_in_leaf_at_idx(i, key, val));
          } else if min.as_slice() < key {
            try!(self.insert_in_leaf_at_idx(i, key, val));
          }
        },
        (None, None) => {
          if i == 0 {
            try!(self.insert_in_leaf_at_idx(i, key, val));
          } 
          stop = true;
        }
      }

      min_key = match max_key {
        Some(k) => Some(k.clone()),
        None => None
      };
      i += 1;
    }
    Ok(())
  }

  fn insert_in_leaf_at_idx(&mut self, idx: u32, key: &[u8], val: &[u8]) -> Result<(), Error> {
    let rec_size = Self::leaf_rec_size(self.key_len, self.val_len) as u64;
    let rec_offset = Self::leaf_rec_offset(idx, self.key_len, self.val_len) as u64;
    let len_to_move = self.node_size as u64 - rec_offset - rec_size;

    let bytes_to_move = try!(self.storage.r_bytes(rec_offset, len_to_move as usize));
    try!(self.storage.w_bytes(rec_offset + rec_size, bytes_to_move.as_slice()));
    try!(self.overwrite_in_leaf_at_idx(idx, key, val));
    Ok(())
  }

  fn overwrite_in_leaf_at_idx(&mut self, idx: u32, key: &[u8], val: &[u8]) -> Result<(), Error> {
    let rec_offset = Self::leaf_rec_offset(idx, self.key_len, self.val_len) as u64;
    try!(self.storage.w_bytes(rec_offset, key)); 
    try!(self.storage.w_bytes(rec_offset + self.key_len as u64, val)); 
    Ok(())
  }

  pub fn search(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
    // TODO: Implement binary search on leaf node records
    try!(self.search_node(key));
    while let Some(r) = try!(self.next_leaf_rec()) {
      println!("WHILE");
      if key == r.key.as_slice() { return Ok(Some(r.val)); }
    };
    Ok(None)
  }

  fn search_node(&mut self, key: &[u8]) -> Result<(), Error> {
    try!(AssertionError::assert(key.len() == self.key_len as usize, ERR_KEY_WRONG_SIZE)); 
    self.state = State::Nothing();

    while match self.state { 
      State::Leaf(_) => false, 
      _ => true 
    } {
      match self.state {
        State::Nothing() => try!(self.enter_node(0)),
        State::Leaf(ref l) => (),
        State::Inner(_) => {
          // TODO: Implement binary search on inner node
          while let Some(r) = try!(self.next_inner_rec()) {
            match (r.min_key, r.max_key) {
              (None, Some(max)) => if key < max.as_slice() {
                return self.enter_node(r.ptr);
              },
              (Some(min), Some(max)) => if min.as_slice() <= key && key < max.as_slice() {
                return self.enter_node(r.ptr);
              },
              (Some(min), None) => if min.as_slice() <= key {
                return self.enter_node(r.ptr);
              },
              (None, None) => { 
                return Err(Error::Assertion(AssertionError::new(ERR_INNER_NODE_EMPTY))); 
              }
            }
          }
        },
        _ => ()
      }
    }

    match self.state {
      State::Leaf(_) => Ok(()),
      _ => Err(Error::Assertion(AssertionError::new(ERR_SEARCH_NO_LEAF_FOR_KEY))) 
    }

  }

  fn enter_node(&mut self, ptr: u64) -> Result<(), Error> {
    match try!(self.storage.r_u8(ptr)) {
      0x02 => {
        self.state = State::Leaf(LeafState {
          ptr: ptr,
          parent_ptr: try!(self.storage.r_u64(ptr + 1)),
          prev_ptr: try!(self.storage.r_u64(ptr + 9)),
          next_ptr: try!(self.storage.r_u64(ptr + 17)),
          num_recs: try!(self.storage.r_u32(ptr + 25)),
          cur_rec_idx: 0
        });
        Ok(())
      },  
      0x01 => {
        self.state = State::Inner(InnerState {
          ptr: ptr,
          parent_ptr: 0,
          num_recs: 0,
          cur_rec_idx: 0
        });
        Ok(())
      },
      _ => Err(Error::Assertion(AssertionError::new(ERR_INVALID_NODE_TYPE)))
    }
  }

  fn inner_rec_offset(rec_idx: u32, key_len: u8) -> u32 {
    INNER_NODE_REC_OFFSET + (8 + key_len as u32) * rec_idx as u32 - key_len as u32
  }

  fn leaf_rec_offset(rec_idx: u32, key_len: u8, val_len: u8) -> u32 {
    LEAF_NODE_REC_OFFSET + (key_len as u32 + val_len as u32) * rec_idx as u32
  }

  fn inner_max_records(node_size: u32, key_len: u8) -> u32 {
    (node_size - INNER_NODE_REC_OFFSET - 8) / Self::inner_rec_size(key_len)
  }

  fn leaf_max_records(node_size: u32, key_len: u8, val_len: u8) -> u32 {
    (node_size - LEAF_NODE_REC_OFFSET) / Self::leaf_rec_size(key_len, val_len)
  }

  fn inner_rec_size(key_len: u8) -> u32 {
    key_len as u32 + 8
  }

  fn leaf_rec_size(key_len: u8, val_len: u8) -> u32 {
    key_len as u32 + val_len as u32
  }

  fn inner_is_full(&mut self) -> Result<bool, Error> {
    match self.state {
      State::Inner(ref i) => Ok(
        Self::inner_max_records(self.node_size, self.key_len) > i.num_recs
      ),
      _ => Err(Error::Assertion(AssertionError::new(ERR_USE_INNER_WHERE_NONE)))
    }
  }

  fn leaf_is_full(&mut self) -> Result<bool, Error> {
    match self.state {
      State::Leaf(ref l) => Ok(
        Self::leaf_max_records(self.node_size, self.key_len, self.val_len) > l.num_recs
      ),
      _ => Err(Error::Assertion(AssertionError::new(ERR_USE_LEAF_WHERE_NONE)))
    }
  }

  fn next_leaf_rec(&mut self) -> Result<Option<LeafRecord>, Error> {
    match self.state {
      State::Leaf(ref mut l) => {
        match l.cur_rec_idx < l.num_recs {
          false => Ok(None),
          true => {
            let rec_offset = Self::leaf_rec_offset(l.cur_rec_idx, self.key_len, self.val_len) as u64;

            let leaf_idx = l.cur_rec_idx;
            let key = try!(self.storage.r_bytes(rec_offset, self.key_len as usize));
            let val = try!(self.storage.r_bytes(rec_offset + self.key_len as u64, self.key_len as usize));

            l.cur_rec_idx += 1;

            Ok(Some(LeafRecord {
              leaf_idx: leaf_idx,
              key: key,
              val: val
            }))
          }
        }
      },
      _ => Err(Error::Assertion(AssertionError::new(ERR_USE_LEAF_WHERE_NONE)))
    }
  }

  fn next_inner_rec(&mut self) -> Result<Option<InnerRecord>, Error> {
    match self.state {
      State::Inner(ref mut i) => {
        match i.cur_rec_idx < i.num_recs {
          false => Ok(None),
          true => {
            let rec_offset = Self::inner_rec_offset(i.cur_rec_idx, self.key_len) as u64;

            let mut min_key: Option<Vec<u8>> = None;
            if i.cur_rec_idx > 0 {
              min_key = Some(try!(self.storage.r_bytes(
                rec_offset - self.key_len as u64,
                self.key_len as usize
              )));
            }

            let ptr = try!(self.storage.r_u64(rec_offset));

            let mut max_key: Option<Vec<u8>> = None;
            if i.cur_rec_idx < i.num_recs {
              max_key = Some(try!(self.storage.r_bytes(
                rec_offset + 8,
                self.key_len as usize
              )));
            }

            i.cur_rec_idx += 1;

            Ok(Some(InnerRecord {
              min_key: min_key,
              ptr: ptr,
              max_key: max_key,
            }))
          }
        }
      }
      _ => Err(Error::Assertion(AssertionError::new(ERR_USE_INNER_WHERE_NONE)))
    }
  }



}
