use std::cell::RefCell;
use std::io::{ Read, Cursor };

use byteorder::{ LittleEndian, ReadBytesExt, WriteBytesExt };

use error::{ Error, AssertionError };
use storage::binary_storage::BinaryStorage;
use storage::bp_tree::node;
use storage::bp_tree::bp_storage::{ NodeData, BPStorage };

// Node type symbol is at the beginning of the node
const NODE_TYPE_OFFSET: usize = 0;
// u8 size
const NODE_TYPE_LEN: usize = 1; 
// NODE_TYPE_OFFSET + NODE_TYPE_LEN
const PARENT_PTR_OFFSET: usize = 1; 
// u64 size
const PARENT_PTR_LEN: usize = 8; 
// PARENT_PTR_OFFSET + PARENT_PTR_LEN
const PREV_PTR_OFFSET: usize = 9; 
// u64 size
const PREV_PTR_LEN: usize = 8; 
// PREV_PTR_OFFSET + PREV_PTR_LEN
const NEXT_PTR_OFFSET: usize = 17; 
// u64 size
const NEXT_PTR_LEN: usize = 8; 
// # all record bytes, NEXT_PTR_OFFSET + NEXT_PTR_LEN
const RECORDS_LEN_OFFSET: usize = 25; 
// u32 size
const RECORDS_LEN_SIZE: usize = 4; 
// Start of records, RECORDS_LEN_OFFSET + RECORDS_LEN_SIZE
const RECORD_START_OFFSET: usize = 29; 

pub struct LeafNode {
  node_ptr: u64,
  parent_ptr: u64,
  node_size: u32,
  keys: Vec<Vec<u8>>,
  vals: Vec<Vec<u8>>,
  key_len: u8,
  val_len: u8,
  prev_ptr: u64,
  next_ptr: u64,
  capacity: u64
}
impl LeafNode {

  pub fn new_empty(
    parent_ptr: u64,
    node_size: u32,
    key_len: u8,
    val_len: u8,
    prev_ptr: u64,
    next_ptr: u64,
    capacity: u64
  ) -> LeafNode {
    LeafNode {
      node_ptr: 0,
      parent_ptr: parent_ptr,
      node_size: node_size,
      keys: Vec::new(),
      vals: Vec::new(),
      key_len: key_len,
      val_len: val_len,
      prev_ptr: prev_ptr,
      next_ptr: next_ptr,
      capacity: capacity
    }
  }

  pub fn from_bytes(
    data: &[u8],
    node_ptr: u64,
    key_len: u8,
    val_len: u8,
  ) -> Result<LeafNode, Error> {

    let node_size = data.len() as u32;
    try!(AssertionError::assert(
      node_size >= RECORD_START_OFFSET as u32, 
      node::ERR_BLOCK_SIZE_TOO_SMALL
    ));

    let mut reader = Cursor::new(data);
    reader.set_position(1);

    let parent_ptr = try!(reader.read_u64::<LittleEndian>());
    let records_len = try!(reader.read_u32::<LittleEndian>());
    let prev_ptr = try!(reader.read_u64::<LittleEndian>());
    let next_ptr = try!(reader.read_u64::<LittleEndian>());

    let mut cur_pos: u64 = RECORD_START_OFFSET as u64;

    let mut keys = Vec::new();
    let mut vals = Vec::new();

    while 
      reader.position() < RECORD_START_OFFSET as u64 + 
      records_len as u64 - 
      val_len as u64 - 
      key_len as u64 
    {
      let mut key_reader = Cursor::new(data);
      key_reader.set_position(cur_pos);
      let mut key_buf = vec![];
      try!(key_reader.take(key_len as u64).read_to_end(&mut key_buf));
      keys.push(key_buf);
      cur_pos = cur_pos + key_len as u64;

      let mut val_reader = Cursor::new(data);
      val_reader.set_position(cur_pos);
      let mut val_buf = vec![];
      try!(val_reader.take(val_len as u64).read_to_end(&mut val_buf));
      vals.push(val_buf);
      cur_pos = cur_pos + val_len as u64;
    }

    Ok(LeafNode {
      node_ptr: node_ptr,
      parent_ptr: parent_ptr,
      node_size: node_size,
      keys: keys,
      vals: vals,
      key_len: key_len,
      val_len: val_len,
      prev_ptr: prev_ptr,
      next_ptr: next_ptr,
      capacity: (node_size as u64 - RECORD_START_OFFSET as u64) / (key_len as u64 + val_len as u64)
    })
  }

  pub fn insert(&mut self, changes: &mut Vec<NodeData>, k: &[u8], v: &[u8]) -> Result<(), Error> {
    match self.is_full() {
      true => {
        unimplemented!();
      },
      false => {
        try!(self.insert_record(changes, k, v));
      }
    }
    // TODO: save node
    //self.storage.save_leaf(self);
    Ok(())
  }

  pub fn split(&mut self, changes: &mut Vec<NodeData>) -> Result<(), Error> {
    let split_idx = self.keys.len() / 2;
    let split_keys = self.keys.split_off(split_idx);
    let split_vals = self.vals.split_off(split_idx);
    /*
    let new_leaf = Leaf::new_empty(
      self.parent_ptr, 
      self.node_size,
      self.key_len,
      self.val_len,
      self.node_ptr,
      0,
      */
    Ok(())
  }

  fn insert_record(&mut self, changes: &mut Vec<NodeData>, k: &[u8], v: &[u8]) -> Result<(), Error> {
    for i in 0..self.len() {
      let key = self.keys[i].clone();
      let key_slice = key.as_slice();

      if key_slice == k {
        self.keys[i] = k.to_vec();
        self.vals[i] = v.to_vec();
      }

      if i == self.len() { break; }

      let next_key = self.keys[i + 1].clone();
      let next_key_slice = next_key.as_slice();

      if key_slice < k && k < next_key_slice {
        self.keys.insert(i, k.to_vec()); 
        self.vals.insert(i, v.to_vec()); 
      }
    }
    changes.push(NodeData {
      ptr: self.parent_ptr,
      req_alloc: false,
      data: try!(self.to_bytes())
    });
    Ok(())
  }

  pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
    let mut bytes: Vec<u8> = Vec::new();
    bytes.push(0x2);
    try!(bytes.write_u64::<LittleEndian>(self.parent_ptr));
    try!(bytes.write_u64::<LittleEndian>(self.prev_ptr));
    try!(bytes.write_u64::<LittleEndian>(self.next_ptr));
    try!(bytes.write_u32::<LittleEndian>(self.node_size));
    for i in 0..self.len() {
      bytes.extend(self.keys[i].clone());
      bytes.extend(self.vals[i].clone());
    }
    Ok(bytes)
  }

  pub fn node_ptr(&self) -> u64 { self.node_ptr }

  pub fn has_parent(&self) -> bool { self.parent_ptr != 0 }
  pub fn has_prev(&self) -> bool { self.prev_ptr != 0 }
  pub fn has_next(&self) -> bool { self.next_ptr != 0 }

  pub fn parent_ptr(&self) -> Option<u64> {
    match self.parent_ptr {
      0 => None,
      _ => Some(self.parent_ptr)
    }
  }

  pub fn prev_ptr(&self) -> Option<u64> {
    match self.prev_ptr {
      0 => None,
      _ => Some(self.prev_ptr)
    }
  }

  pub fn next_ptr(&self) -> Option<u64> {
    match self.next_ptr {
      0 => None,
      _ => Some(self.next_ptr)
    }
  }

  pub fn link_parent(&mut self, parent_ptr: u64) -> Result<(), Error> {
    try!(AssertionError::assert_not(parent_ptr == 0, node::ERR_INVALID_BLOCK_NUM));
    self.parent_ptr = parent_ptr;
    Ok(())
  }

  pub fn link_prev(&mut self, prev_ptr: u64) -> Result<(), Error> {
    try!(AssertionError::assert_not(prev_ptr == 0, node::ERR_INVALID_BLOCK_NUM));
    self.prev_ptr = prev_ptr;
    Ok(())
  }

  pub fn link_next(&mut self, next_ptr: u64) -> Result<(), Error> {
    try!(AssertionError::assert_not(next_ptr == 0, node::ERR_INVALID_BLOCK_NUM));
    self.next_ptr = next_ptr;
    Ok(())
  }

  pub fn unlink_parent(&mut self) {
    self.parent_ptr = 0;
  }

  pub fn unlink_prev(&mut self) {
    self.prev_ptr = 0;
  }

  pub fn unlink_next(&mut self) {
    self.next_ptr = 0;
  }

  pub fn len(&self) -> usize {
    self.keys.len()
  }

  pub fn capacity(&self) -> u64 {
    self.capacity
  }

  pub fn is_full(&self) -> bool {
    let rec_len = self.key_len as u64 + self.val_len as u64;
    self.capacity - (self.len() as u64 * rec_len) > rec_len
  }

}
impl IntoIterator for LeafNode {

  type Item = LeafNodeRecord;
  type IntoIter = LeafNodeIterator;

  fn into_iter(self) -> Self::IntoIter {
    LeafNodeIterator { node: self, current: 0 }
  }

}


pub struct LeafNodeIterator {
  node: LeafNode,
  current: usize
}
impl Iterator for LeafNodeIterator {

  type Item = LeafNodeRecord;

  fn next(&mut self) -> Option<LeafNodeRecord> {
    if self.current < self.node.len() {
      let i = self.current as usize;
      self.current += 1;
      Some(LeafNodeRecord {
        key: self.node.keys[i].clone(), 
        val: self.node.vals[i].clone()
      })
    } else {
      None
    }
  }

}

pub struct LeafNodeRecord {
  pub key: Vec<u8>,
  pub val: Vec<u8>
}
