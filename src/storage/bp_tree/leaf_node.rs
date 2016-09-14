use std::io::Cursor;

use byteorder::{ LittleEndian, ReadBytesExt, WriteBytesExt };

use error::{ Error, AssertionError };
use storage::bp_tree::node;

pub static ERR_INVALID_BLOCK_NUM: & 'static str = "Invalid block number";

const NODE_TYPE_OFFSET: usize = 0;
const PARENT_OFFSET: usize = 1;
const PREV_OFFSET: usize = 5;
const NEXT_OFFSET: usize = 9;
const LEN_OFFSET: usize = 13;
const RECORDS_OFFSET: usize = 17;

pub struct LeafNode {
    block: u32,
    block_size: u32,
    len: u32,
    parent: u32,
    prev: u32,
    next: u32,
    keys: Vec<Vec<u8>>,
    values: Vec<Vec<u8>>,
    key_len: u32,
    val_len: u32
}
impl LeafNode {

    pub fn from_bytes(
        data: &[u8],
        block: u32,
        block_size: u32,
        key_len: u32,
        val_len: u32
    ) -> Result<LeafNode, Error> {

        let parent_buf = &data[PARENT_OFFSET..(PARENT_OFFSET + 4)];
        let mut parent_rdr = Cursor::new(parent_buf);
        let parent = try!(parent_rdr.read_u32::<LittleEndian>());

        let prev_buf = &data[PREV_OFFSET..(PREV_OFFSET + 4)];
        let mut prev_rdr = Cursor::new(prev_buf);
        let prev = try!(prev_rdr.read_u32::<LittleEndian>());

        let next_buf = &data[NEXT_OFFSET..(NEXT_OFFSET + 4)];
        let mut next_rdr = Cursor::new(next_buf);
        let next = try!(next_rdr.read_u32::<LittleEndian>());

        let len_buf = &data[LEN_OFFSET..(LEN_OFFSET + 4)];
        let mut len_rdr = Cursor::new(len_buf);
        let len = try!(len_rdr.read_u32::<LittleEndian>());

        let mut keys = Vec::new();
        let mut values = Vec::new();

        let rec_len = (key_len + val_len) as usize;
        for i in 0..len {
            let k_offset = RECORDS_OFFSET + rec_len * i as usize;
            let v_offset = k_offset + key_len as usize;
            keys.push(data[k_offset..(k_offset + key_len as usize)].to_vec()); 
            values.push(data[v_offset..(v_offset + val_len as usize)].to_vec()); 
        }

        Ok(LeafNode {
            block: block,
            block_size: block_size,
            len: len,
            parent: parent,
            prev: prev,
            next: next,
            keys: keys,
            values: values,
            key_len: key_len,
            val_len: val_len
        })

    }


    pub fn block(&self) -> u32 { self.block }

    pub fn has_parent(&self) -> bool { self.parent != 0 }
    pub fn has_prev(&self) -> bool { self.prev != 0 }
    pub fn has_next(&self) -> bool { self.next != 0 }

    pub fn parent(&self) -> Option<u32> {
        match self.parent {
            0 => None,
            _ => Some(self.parent)
        }
    }

    pub fn prev(&self) -> Option<u32> {
        match self.prev {
            0 => None,
            _ => Some(self.prev)
        }
    }

    pub fn next(&self) -> Option<u32> {
        match self.next {
            0 => None,
            _ => Some(self.next)
        }
    }

    pub fn link_parent(&mut self, block: u32) -> Result<(), Error> {
        try!(AssertionError::assert_not(block == 0, ERR_INVALID_BLOCK_NUM));
        self.parent = block;
        Ok(())
    }

    pub fn link_prev(&mut self, block: u32) -> Result<(), Error> {
        try!(AssertionError::assert_not(block == 0, ERR_INVALID_BLOCK_NUM));
        self.prev = block;
        Ok(())
    }

    pub fn link_next(&mut self, block: u32) -> Result<(), Error> {
        try!(AssertionError::assert_not(block == 0, ERR_INVALID_BLOCK_NUM));
        self.next = block;
        Ok(())
    }

    pub fn unlink_parent(&mut self) {
        self.parent = 0;
    }

    pub fn unlink_prev(&mut self) {
        self.prev = 0;
    }

    pub fn unlink_next(&mut self) {
        self.next = 0;
    }

    pub fn len(&self) -> u32 {
        self.len
    }

}
impl IntoIterator for LeafNode {

    type Item = (Vec<u8>, Vec<u8>);
    type IntoIter = LeafNodeIterator;

    fn into_iter(self) -> Self::IntoIter {
        LeafNodeIterator { node: self, current: 0 }
    }

}


pub struct LeafNodeIterator {
    node: LeafNode,
    current: u32
}
impl Iterator for LeafNodeIterator {

    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<(Vec<u8>, Vec<u8>)> {
        if self.current < self.node.len {
            let i = self.current as usize;
            self.current += 1;
            Some((
                self.node.keys[i].clone(), 
                self.node.values[i].clone()
            ))
        } else {
            None
        }
    }

}
