use byteorder::{ LittleEndian, ReadBytesExt, WriteBytesExt };

use error::{ Error, AssertionError };
use storage::binary_storage::BinaryStorage;

pub static ERR_READ_LEAF_WHERE_NONE: & 'static str = 
  "Tried to read leaf node from file location where none exists";
pub static ERR_READ_INNER_WHERE_NONE: & 'static str = 
  "Tried to read inner node from file location where none exists";
pub static ERR_KEY_WRONG_SIZE: & 'static str = 
  "Key is the wrong nubmer of bytes";
pub static ERR_READ_PAST_INNER_NODE: & 'static str = 
  "Tried to read more records from inner node than exist in the node";
pub static ERR_INNER_NODE_EMPTY: & 'static str = 
  "Encountered an inner node with no records";
pub static ERR_SEARCH_NO_LEAF: & 'static str = 
  "Search could not find a leaf node";

const INNER_NODE_REC_OFFSET: u64 = 0;
const LEAF_NODE_REC_OFFSET: u64 = 0;

struct LeafRecord {
  pub key: Vec<u8>,
  pub val: Vec<u8>
}

struct InnerRecord {
  pub min_key: Option<Vec<u8>>,
  pub ptr: u64,
  pub max_key: Option<Vec<u8>>
}

struct InnerState {
  pub ptr: u64,
  pub num_recs: u32,
  pub cur_rec_idx: u32
}

struct LeafState {
  pub ptr: u64,
  pub num_recs: u32,
  pub cur_rec_idx: u32
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
  state: State
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
      state: State::Nothing()
    }
  }

  pub fn open(&mut self) -> Result<(), Error> {
    self.storage.open()
    // TODO: Ensure block_size, key_len, and val_len are same as saved index data
  }

  pub fn close(&mut self) -> Result<(), Error> {
    self.storage.close()
  }

  fn search(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
    // TODO: Implement binary search on leaf node records
    try!(self.search_node(key));
    while let Some(r) = try!(self.next_leaf_rec()) {
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
        _ => (),
        State::Nothing() => try!(self.enter_node(0)),
        State::Leaf(ref l) => (),
        State::Inner(_) => {
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
        }
          /*
        while if let Some(r) = try!(self.next_inner_rec()) {
          match (r.min_key, r.max_key) {
            (None, Some(max)) => if k < max.as_slice() {
              return self.enter_node(r.ptr);
            },
            (Some(min), Some(max)) => if min.as_slice() <= k && k < max.as_slice() {
              return self.enter_node(r.ptr);
            },
            (Some(min), None) => if min.as_slice <= k {
              return self.enter_node(r.ptr);
            },
            (None, None) => { 
              return Err(Error::Assertion(AssertionError::new(ERR_INNER_NODE_EMPTY))); 
            }
          }
        }
        */
      }
    }

    Err(Error::Assertion(AssertionError::new(ERR_SEARCH_NO_LEAF))) 

  }

  fn enter_node(&mut self, ptr: u64) -> Result<(), Error> {
    match try!(self.storage.r_bool(ptr)) {
      true => {
        self.state = State::Leaf(LeafState {
          ptr: ptr,
          num_recs: 0,
          cur_rec_idx: 0
        });
      },  
      false => {
        self.state = State::Inner(InnerState {
          ptr: ptr,
          num_recs: 0,
          cur_rec_idx: 0
        });
      }
    };
    Ok(())
  }

  fn inner_rec_offset(rec_idx: u32, key_len: u8) -> u64 {
    INNER_NODE_REC_OFFSET + (8 + key_len as u64) * rec_idx as u64 - key_len as u64
  }

  fn leaf_rec_offset(rec_idx: u32, key_len: u8, val_len: u8) -> u64 {
    LEAF_NODE_REC_OFFSET + (key_len as u64 + val_len as u64) * rec_idx as u64
  }

  fn next_leaf_rec(&mut self) -> Result<Option<LeafRecord>, Error> {
    match self.state {
      State::Leaf(ref mut l) => {
        match l.cur_rec_idx < l.num_recs {
          false => Ok(None),
          true => {
            let rec_offset = Self::leaf_rec_offset(l.cur_rec_idx, self.key_len, self.val_len);

            let key = try!(self.storage.r_bytes(rec_offset, self.key_len as usize));
            let val = try!(self.storage.r_bytes(rec_offset + self.key_len as u64, self.key_len as usize));

            l.cur_rec_idx += 1;

            Ok(Some(LeafRecord {
              key: key,
              val: val
            }))
          }
        }
      },
      _ => Err(Error::Assertion(AssertionError::new(ERR_READ_LEAF_WHERE_NONE)))
    }
  }

  fn next_inner_rec(&mut self) -> Result<Option<InnerRecord>, Error> {
    match self.state {
      State::Inner(ref mut i) => {
        match i.cur_rec_idx < i.num_recs {
          false => Ok(None),
          true => {
            let rec_offset = Self::inner_rec_offset(i.cur_rec_idx, self.key_len);

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
      _ => Err(Error::Assertion(AssertionError::new(ERR_READ_INNER_WHERE_NONE)))
    }
  }



}
