use std::mem::size_of;
use std::io::{ Read, Cursor };
use std::ops::Index;

use byteorder::{ LittleEndian, ReadBytesExt, WriteBytesExt };

use error::{ Error, AssertionError };
use storage::bp_tree::node;

// Node type symbol is at beginning of node
const NODE_TYPE_OFFSET: usize = 0;
// u8 size
const NODE_TYPE_LEN: usize = 1; 
// NODE_TYPE_OFFSET + NODE_TYPE_LEN
const PARENT_PTR_OFFSET: usize = 1; 
// u64 size
const PARENT_PTR_LEN: usize = 8; 
// # all record bytes, PARENT_PTR_OFFSET + PARENT_PTR_LEN
const RECORDS_LEN_OFFSET: usize = 9; 
// u32 size
const RECORDS_LEN_SIZE: usize = 4; 
// Start of records, RECORDS_LEN_OFFSET + RECORDS_LEN_SIZE
const RECORD_START_OFFSET: usize = 9; 
// u64 size
const PTR_LEN: usize = 8; 

pub struct InnerNode {
  node_ptr: u64, // Pointer to beginning of node data
  parent_ptr: u64, // Pointer to parent node 
  node_size: u32, // Size in bytes of the entire node, used or not
  keys: Vec<Vec<u8>>, // List of keys
  pointers: Vec<u64>, // List of pointers to left and right of keys 
  key_len: u8 // Length in bytes of a single key
}
impl InnerNode {

  pub fn from_bytes(
    data: &[u8],
    node_ptr: u64,
    key_len: u8
  ) -> Result<InnerNode, Error> {

    let node_size = data.len() as u32;
    try!(AssertionError::assert(
      node_size >= RECORD_START_OFFSET as u32, 
      node::ERR_BLOCK_SIZE_TOO_SMALL
    ));

    let mut reader = Cursor::new(data);
    reader.set_position(1);

    let parent_ptr = try!(reader.read_u64::<LittleEndian>());
    let records_len = try!(reader.read_u32::<LittleEndian>());

    let mut next_is_ptr = true;
    let mut cur_pos: u64 = RECORD_START_OFFSET as u64;

    let mut keys = Vec::new();
    let mut pointers = Vec::new();

    // Loop as long as we still have room to read the next key or pointer
    while match next_is_ptr {
      true => cur_pos < 
        RECORD_START_OFFSET as u64 + 
        records_len as u64 - 
        PTR_LEN as u64,
      false => cur_pos < 
        RECORD_START_OFFSET as u64 + 
        records_len as u64 - 
        key_len as u64
    } {
      let mut rec_reader = Cursor::new(data);
      rec_reader.set_position(cur_pos);
      match next_is_ptr {
        true => pointers.push(try!(rec_reader.read_u64::<LittleEndian>())),
        false => {
          let mut buf = vec![];
          try!(rec_reader.take(key_len as u64).read_to_end(&mut buf));
          keys.push(buf);
        }
      };
      next_is_ptr = !next_is_ptr;
      cur_pos = reader.position();
    }

    Ok(InnerNode {
      node_ptr: node_ptr,
      parent_ptr: parent_ptr,
      node_size: node_size,
      keys: keys,
      pointers: pointers,
      key_len: key_len
    })
  }

  pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
    unimplemented!();
  }

  pub fn node_ptr(&self) -> u64 { self.node_ptr }

  pub fn has_parent(&self) -> bool { self.parent_ptr != 0 }

  pub fn parent_ptr(&self) -> Option<u64> {
    match self.parent_ptr {
      0 => None,
      _ => Some(self.parent_ptr)
    }
  }

  pub fn link_parent(&mut self, parent_ptr: u64) -> Result<(), Error> {
    try!(AssertionError::assert_not(parent_ptr == 0, node::ERR_INVALID_BLOCK_NUM));
    self.parent_ptr = parent_ptr;
    Ok(())
  }

  pub fn unlink_parent(&mut self) {
    self.parent_ptr = 0;
  }


  pub fn len(&self) -> usize {
    self.keys.len()
  }

}
impl IntoIterator for InnerNode {

  type Item = InnerNodeRecord;
  type IntoIter = InnerNodeIterator;

  fn into_iter(self) -> Self::IntoIter {
    InnerNodeIterator { node: self, current: 0 }
  }

}


pub struct InnerNodeIterator {
  node: InnerNode,
  current: usize
}
impl Iterator for InnerNodeIterator {

  type Item = InnerNodeRecord;

  fn next(&mut self) -> Option<InnerNodeRecord> {
    if self.current < self.node.len() {
      let i = self.current;
      self.current += 1;

      let is_first = i == 0;
      let is_last = i < self.node.len() as usize - 1;

      Some(InnerNodeRecord {
        min_key: match is_first {
          true => None,
          false => Some(self.node.keys[i].clone())
        },
        max_key: match is_last {
          true => None,
          false => Some(self.node.keys[i + 1].clone()),
        },
        pointer: self.node.pointers[i] 
      })
    } else {
        None
    }
  }

}


pub struct InnerNodeRecord {
  pub min_key: Option<Vec<u8>>,
  pub max_key: Option<Vec<u8>>,
  pub pointer: u64,
}

