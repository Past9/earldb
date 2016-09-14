use std::io::Cursor;
use std::ops::Index;

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

    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        unimplemented!();
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

    pub fn len(&self) -> u32 {
        self.len
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
    current: u32
}
impl Iterator for InnerNodeIterator {

    type Item = InnerNodeRecord;

    fn next(&mut self) -> Option<InnerNodeRecord> {
        if self.current < self.node.len {
            let i = self.current as usize;
            self.current += 1;

            let is_first = i == 0;
            let is_last = i < self.node.len as usize - 1;

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

            /*
            Some(InnerNodeRecord {
                key: self.node.keys[i].clone(),
                lt_pointer: self.node.pointers[i].clone(),
                gte_pointer: self.node.pointers[i + 1].clone(),
            })
            */
        } else {
            None
        }
    }

}


pub struct InnerNodeRecord {
    pub min_key: Option<Vec<u8>>,
    pub max_key: Option<Vec<u8>>,
    pub pointer: u32,
}

