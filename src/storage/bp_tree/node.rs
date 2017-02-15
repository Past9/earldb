use std::io::Cursor;

use byteorder::{ LittleEndian, ReadBytesExt, WriteBytesExt };

use error::{ Error, AssertionError };
use storage::bp_tree::inner_node::InnerNode;
use storage::bp_tree::leaf_node::LeafNode;

pub static ERR_INVALID_NODE_TYPE: & 'static str = "Node type not recognized";
pub static ERR_NODE_DATA_WRONG_LENGTH: & 'static str = "Invalid node block size";
pub static ERR_BLOCK_SIZE_TOO_SMALL: & 'static str = "Data too small to read inner block";
pub static ERR_INVALID_BLOCK_NUM: & 'static str = "Invalid block number";

const NODE_TYPE_OFFSET: usize = 0;

pub enum Node {
  Inner(InnerNode),
  Leaf(LeafNode)
}
impl Node {

  pub fn from_bytes(
    data: &[u8],
    node_ptr: u64,
    key_len: u8,
    val_len: u8
  ) -> Result<Node, Error> {
    match data[0] {
      1 => {
        let node = try!(InnerNode::from_bytes(data, node_ptr, key_len));
        Ok(Node::Inner(node))
      },
      2 => {
        let node = try!(LeafNode::from_bytes(data, node_ptr, key_len, val_len));
        Ok(Node::Leaf(node))
      },
      _ => return Err(Error::Assertion(AssertionError::new(ERR_INVALID_NODE_TYPE)))
    }
  }

}


/*
pub enum NodeType {
  Inner,
  Leaf
}

pub struct Node {
  block_num: u32,
  block_size: u32,
  node_type: NodeType,
  num_records: u32,
  has_parent_block: bool,
  has_prev_block: bool,
  has_next_block: bool,
  parent_block_num: u32,
  prev_block_num: u32,
  next_block_num: u32,
  keys: Vec<Vec<u8>>,
  values: Vec<Vec<u8>>,
  key_len: u32,
  val_len: u32,
  current_rec: u32
}
impl Node {

  pub fn from_bytes(
    data: &[u8],
    block_num: u32,
    block_size: u32,
    key_len: u32, 
    val_len: u32
  ) -> Result<Node, Error> {

    try!(AssertionError::assert(data.len() == block_size as usize, ERR_NODE_DATA_WRONG_LENGTH));

    let node_type = match data[0] {
      1 => NodeType::Inner,
      2 => NodeType::Leaf,
      _ => return Err(Error::Assertion(AssertionError::new(ERR_NODE_CORRUPTED)))
    };

    let mut parent_block_num = 0;
    let has_parent_block = match data[HAS_PARENT_BLOCK_OFFSET] {
      1 => {
        let buf = &data[PARENT_BLOCK_NUM_OFFSET..(PARENT_BLOCK_NUM_OFFSET + 4)];
        let mut rdr = Cursor::new(buf);
        parent_block_num = try!(rdr.read_u32::<LittleEndian>());
        true
      },
      2 => false,
      _ => return Err(Error::Assertion(AssertionError::new(ERR_NODE_CORRUPTED)))
    };

    let mut prev_block_num = 0;
    let has_prev_block = match data[HAS_PREV_BLOCK_OFFSET] {
      1 => {
        let buf = &data[PREV_BLOCK_NUM_OFFSET..(PREV_BLOCK_NUM_OFFSET + 4)];
        let mut rdr = Cursor::new(buf);
        prev_block_num = try!(rdr.read_u32::<LittleEndian>());
        true
      },
      2 => false,
      _ => return Err(Error::Assertion(AssertionError::new(ERR_NODE_CORRUPTED)))
    };

    let mut next_block_num = 0;
    let has_next_block = match data[HAS_NEXT_BLOCK_OFFSET] {
      1 => {
        let buf = &data[NEXT_BLOCK_NUM_OFFSET..(NEXT_BLOCK_NUM_OFFSET + 4)];
        let mut rdr = Cursor::new(buf);
        next_block_num = try!(rdr.read_u32::<LittleEndian>());
        true
      },
      2 => false,
      _ => return Err(Error::Assertion(AssertionError::new(ERR_NODE_CORRUPTED)))
    };

    let buf = &data[NUM_RECORDS_OFFSET..(NUM_RECORDS_OFFSET + 4)];
    let mut rdr = Cursor::new(buf);
    let num_records = try!(rdr.read_u32::<LittleEndian>());

    let mut keys = Vec::new();
    let mut values = Vec::new();

    let rec_len = (key_len + val_len) as usize;
    for i in 0..num_records {
      let k_offset = RECORDS_OFFSET + rec_len * i as usize;
      let v_offset = k_offset + key_len as usize;
      keys.push(data[k_offset..key_len as usize].to_vec()); 
      values.push(data[v_offset..val_len as usize].to_vec()); 
    }

    Ok(Node {
      block_num: block_num,
      block_size: block_size,
      node_type: node_type,
      has_parent_block: has_parent_block,
      has_prev_block: has_prev_block,
      has_next_block: has_next_block,
      parent_block_num: parent_block_num,
      prev_block_num: prev_block_num,
      next_block_num: next_block_num,
      num_records: num_records, 
      keys: keys,
      values: values,
      key_len: key_len,
      val_len: val_len,
      current_rec: 0
    })

  }


  pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {

    let offset = self.block_num * self.block_size;

    let mut data: Vec<u8> = vec![0; self.block_size as usize];

    data.push(match self.node_type {
      NodeType::Inner => 1,
      NodeType::Leaf => 2
    });

    data.push(match self.has_parent_block {
      true => 1,
      false => 0
    });

    let mut parent_block_num_buf = vec!();
    try!(parent_block_num_buf.write_u32::<LittleEndian>(self.parent_block_num));
    data.extend_from_slice(parent_block_num_buf.as_slice());

    data.push(match self.has_prev_block {
      true => 1,
      false => 0
    });

    let mut prev_block_num_buf = vec!();
    try!(prev_block_num_buf.write_u32::<LittleEndian>(self.prev_block_num));
    data.extend_from_slice(prev_block_num_buf.as_slice());

    data.push(match self.has_next_block {
      true => 1,
      false => 0
    });

    let mut next_block_num_buf = vec!();
    try!(next_block_num_buf.write_u32::<LittleEndian>(self.next_block_num));
    data.extend_from_slice(next_block_num_buf.as_slice());

    let mut num_records_buf = vec!();
    try!(num_records_buf.write_u32::<LittleEndian>(self.num_records));
    data.extend_from_slice(num_records_buf.as_slice());

    for i in 0..self.num_records {
      data.extend_from_slice(self.keys[i as usize].as_slice());
      data.extend_from_slice(self.values[i as usize].as_slice());
    }

    if data.len() < self.block_size as usize {
      let padding = vec![0; (self.block_size as usize) - data.len()];
      data.extend_from_slice(padding.as_slice());
    }

    Ok(data)

  }

  pub fn get_block_num(&self) -> u32 { self.block_num }

  pub fn has_parent_block(&self) -> bool { self.has_parent_block }
  pub fn has_prev_block(&self) -> bool { self.has_prev_block }
  pub fn has_next_block(&self) -> bool { self.has_next_block }

  pub fn is_root(&self) -> bool { !self.has_parent_block && self.block_num == 1 }

  pub fn get_parent_block_num(&self) -> Option<u32> { 
    match self.has_parent_block {
      true => Some(self.parent_block_num),
      false => None
    }
  }

  pub fn get_prev_block_num(&self) -> Option<u32> { 
    match self.has_prev_block {
      true => Some(self.prev_block_num),
      false => None
    }
  }

  pub fn get_next_block_num(&self) -> Option<u32> { 
    match self.has_next_block {
      true => Some(self.next_block_num),
      false => None
    }
  }

  pub fn set_prev_block_num(&mut self, prev_block_num: u32) {
    self.has_prev_block = true;
    self.prev_block_num = prev_block_num;
  }

  pub fn set_next_block_num(&mut self, next_block_num: u32) {
    self.has_next_block = true;
    self.next_block_num = next_block_num;
  }

  pub fn set_parent_block_num(&mut self, parent_block_num: u32) {
    self.has_parent_block = true;
    self.parent_block_num = parent_block_num;
  }

  pub fn remove_prev_block(&mut self) {
    self.has_prev_block = false;
    self.prev_block_num = 0;
  }

  pub fn remove_next_block(&mut self) {
    self.has_next_block = false;
    self.next_block_num = 0;
  }

  pub fn remove_parent_block(&mut self) {
    self.has_parent_block = false;
    self.parent_block_num = 0;
  }

  pub fn get_node_type(&self) -> NodeType { self.node_type }

  pub fn reset(&mut self) { self.current_rec = 0; }

}
impl Iterator for Node {

  type Item = (Vec<u8>, Vec<u8>);

  fn next(&mut self) -> Option<(Vec<u8>, Vec<u8>)> {

    if self.current_rec < self.num_records {
      let i = self.current_rec as usize;
      self.current_rec += 1;
      Some((self.keys[i].clone(), self.values[i].clone()))
    } else {
      None
    }

  }

}
*/
