use error::{ Error, AssertionError };
use storage::binary_storage::BinaryStorage;

pub static ERR_NODE_CORRUPTED: & 'static str = "Node type not recognized";
pub static ERR_NO_NODE_DATA: & 'static str = "No data for node";

const NODE_TYPE_OFFSET: u64 = 0;
const HAS_PARENT_BLOCK_OFFSET: u64 = 1;
const PARENT_BLOCK_NUM_OFFSET: u64 = 2;
const HAS_PREV_BLOCK_OFFSET: u64 = 10;
const PREV_BLOCK_NUM_OFFSET: u64 = 11;
const HAS_NEXT_BLOCK_OFFSET: u64 = 19;
const NEXT_BLOCK_NUM_OFFSET: u64 = 20;
const NUM_RECORDS_OFFSET: u64 = 28;
const RECORDS_OFFSET: u64 = 36;

pub enum NodeType {
    Inner,
    Leaf
}

pub struct Node {
    offset: u64,
    block_size: u64,
    node_type: NodeType,
    num_records: u64,
    has_parent_block: bool,
    has_prev_block: bool,
    has_next_block: bool,
    parent_block_num: u64,
    prev_block_num: u64,
    next_block_num: u64,
    keys: Vec<Vec<u8>>,
    values: Vec<Vec<u8>>,
    key_len: usize,
    val_len: usize,
}
impl Node {

    pub fn from_storage<T: BinaryStorage + Sized>(
        storage: &mut T,
        offset: u64,
        block_size: u64,
        key_len: usize, 
        val_len: usize
    ) -> Result<Node, Error> {

        let node_type = match try!(storage.r_u8(offset + NODE_TYPE_OFFSET)) {
            1 => NodeType::Inner,
            2 => NodeType::Leaf,
            _ => return Err(Error::Assertion(AssertionError::new(ERR_NODE_CORRUPTED)))
        };

        let mut parent_block_num = 0;
        let has_parent_block = match try!(storage.r_u8(offset + HAS_PARENT_BLOCK_OFFSET)) {
            1 => {
                parent_block_num = try!(storage.r_u64(offset + PARENT_BLOCK_NUM_OFFSET));
                true
            },
            2 => false,
            _ => return Err(Error::Assertion(AssertionError::new(ERR_NODE_CORRUPTED)))
        };

        let mut prev_block_num = 0;
        let has_prev_block = match try!(storage.r_u8(offset + HAS_PREV_BLOCK_OFFSET)) {
            1 => {
                prev_block_num = try!(storage.r_u64(offset + PREV_BLOCK_NUM_OFFSET));
                true
            },
            2 => false,
            _ => return Err(Error::Assertion(AssertionError::new(ERR_NODE_CORRUPTED)))
        };

        let mut next_block_num = 0;
        let has_next_block = match try!(storage.r_u8(offset + HAS_NEXT_BLOCK_OFFSET)) {
            1 => {
                next_block_num = try!(storage.r_u64(offset + NEXT_BLOCK_NUM_OFFSET));
                true
            },
            2 => false,
            _ => return Err(Error::Assertion(AssertionError::new(ERR_NODE_CORRUPTED)))
        };

        let num_records = try!(storage.r_u64(offset + NUM_RECORDS_OFFSET));

        let mut keys = Vec::new();
        let mut values = Vec::new();

        let rec_len = (key_len + val_len) as u64;
        for i in 0..num_records {
            let k_offset = offset + RECORDS_OFFSET + rec_len * i;
            let v_offset = k_offset + key_len as u64;
            keys.push(try!(storage.r_bytes(offset + k_offset, key_len))); 
            values.push(try!(storage.r_bytes(offset + v_offset, val_len))); 
        }

        Ok(Node {
            offset: offset,
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
            val_len: val_len
        })

    }


    pub fn to_storage<T: BinaryStorage + Sized>(
        &self, 
        storage: &mut T
    ) -> Result<(), Error> {

        try!(storage.w_u8(self.offset + NODE_TYPE_OFFSET, match self.node_type {
            NodeType::Inner => 1,
            NodeType::Leaf => 2,
        }));

        try!(storage.w_u64(self.offset + PARENT_BLOCK_NUM_OFFSET, self.parent_block_num));
        try!(storage.w_u64(self.offset + PREV_BLOCK_NUM_OFFSET, self.prev_block_num));
        try!(storage.w_u64(self.offset + NEXT_BLOCK_NUM_OFFSET, self.next_block_num));
        try!(storage.w_u64(self.offset + NUM_RECORDS_OFFSET, self.num_records));

        let mut rec_data = Vec::new();
        for i in 0..self.num_records {
            rec_data.extend_from_slice(self.keys[i as usize].as_slice());
            rec_data.extend_from_slice(self.values[i as usize].as_slice());
        }

        try!(storage.w_bytes(self.offset + RECORDS_OFFSET, rec_data.as_slice()));

        Ok(())

    }

    pub fn has_parent_block(&self) -> bool { self.has_parent_block }
    pub fn has_prev_block(&self) -> bool { self.has_prev_block }
    pub fn has_next_block(&self) -> bool { self.has_next_block }

    pub fn is_root(&self) -> bool {
        !self.has_parent_block
    }

    pub fn get_parent_block_num(&self) -> Option<u64> { 
        match self.has_parent_block {
            true => Some(self.parent_block_num),
            false => None
        }
    }

    pub fn get_prev_block_num(&self) -> Option<u64> { 
        match self.has_prev_block {
            true => Some(self.prev_block_num),
            false => None
        }
    }

    pub fn get_next_block_num(&self) -> Option<u64> { 
        match self.has_next_block {
            true => Some(self.next_block_num),
            false => None
        }
    }


}

