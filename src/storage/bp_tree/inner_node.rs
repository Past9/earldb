use std::io::Cursor;

use byteorder::{ LittleEndian, ReadBytesExt, WriteBytesExt };

use error::{ Error, AssertionError };
use storage::bp_tree::node;

pub static ERR_INVALID_BLOCK_NUM: & 'static str = "Invalid block number";

const NODE_TYPE_OFFSET: usize = 0;
const PARENT_OFFSET: usize = 1;
const LEN_OFFSET: usize = 5;
const RECORDS_OFFSET: usize = 9;

pub struct InnerNode {
    block: u32,
    block_size: u32,
    len: u32,
    parent: u32,
    keys: Vec<Vec<u8>>,
    pointers: Vec<u32>,
    key_len: u32
}
impl InnerNode {

    pub fn from_bytes(
        data: &[u8],
        block: u32,
        block_size: u32,
        key_len: u32
    ) -> Result<InnerNode, Error> {

        let parent_buf = &data[PARENT_OFFSET..(PARENT_OFFSET + 4)];
        let mut parent_rdr = Cursor::new(parent_buf);
        let parent = try!(parent_rdr.read_u32::<LittleEndian>());

        let len_buf = &data[LEN_OFFSET..(LEN_OFFSET + 4)];
        let mut len_rdr = Cursor::new(len_buf);
        let len = try!(len_rdr.read_u32::<LittleEndian>());

        let mut keys = Vec::new();
        let mut pointers = Vec::new();

        let rec_len = (key_len + 4) as usize;
        for i in 0..len {
            let p_offset = RECORDS_OFFSET + rec_len * i as usize;
            let k_offset = p_offset + 4;

            let p_buf = &data[p_offset..(p_offset + 4)];
            let mut p_rdr = Cursor::new(p_buf);
            pointers.push(try!(p_rdr.read_u32::<LittleEndian>()));

            keys.push(data[k_offset..(k_offset + key_len as usize)].to_vec());
        }

        let final_p_offset = RECORDS_OFFSET + rec_len * (len as usize + 1);
        let p_buf = &data[final_p_offset..(final_p_offset + 4)];
        let mut p_rdr = Cursor::new(p_buf);
        pointers.push(try!(p_rdr.read_u32::<LittleEndian>()));

        Ok(InnerNode {
            block: block,
            block_size: block_size,
            len: len,
            parent: parent,
            keys: keys,
            pointers: pointers,
            key_len: key_len
        })

    }

    pub fn block(&self) -> u32 { self.block }

    pub fn has_parent(&self) -> bool { self.parent != 0 }

    pub fn parent(&self) -> Option<u32> {
        match self.parent {
            0 => None,
            _ => Some(self.parent)
        }
    }

    pub fn link_parent(&mut self, block: u32) -> Result<(), Error> {
        try!(AssertionError::assert_not(block == 0, ERR_INVALID_BLOCK_NUM));
        self.parent = block;
        Ok(())
    }

    pub fn unlink_parent(&mut self) {
        self.parent = 0;
    }

    pub fn len (&self) -> u32 {
        self.len
    }

}
impl IntoIterator for InnerNode {

    type Item = (Vec<u8>, u32, u32);
    type IntoIter = InnerNodeIterator;

    fn into_iter(self) -> Self::IntoIter {
        InnerNodeIterator { node: self, current: 0 }
    }

}


pub struct InnerNodeIterator {
    node: InnerNode,
    current: u32
}
impl Iterator for InnerNodeIterator {

    type Item = (Vec<u8>, u32, u32);

    fn next(&mut self) -> Option<(Vec<u8>, u32, u32)> {
        if self.current < self.node.len {
            let i = self.current as usize;
            self.current += 1;
            Some((
                self.node.keys[i].clone(),
                self.node.pointers[i].clone(),
                self.node.pointers[i + 1].clone(),
            ))
        } else {
            None
        }
    }

}
