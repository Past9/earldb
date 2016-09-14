use error::{ Error, AssertionError };
use storage::binary_storage::BinaryStorage;

pub static ERR_NODE_TYPE_INVALID: & 'static str = "Node type not recognized";
pub static ERR_NO_NODE_DATA: & 'static str = "No data for node";

const NODE_TYPE_OFFSET: u64 = 0;
const PARENT_PTR_OFFSET: u64 = 1;
const NUM_RECORDS_OFFSET: u64 = 9;
const RECORDS_OFFSET: u64 = 17;

pub enum NodeType {
    Inner,
    Leaf
}

pub struct Node {
    offset: u64,
    size_bytes: u64,
    node_type: NodeType,
    num_records: u64,
    parent_ptr: u64,
    keys: Vec<Vec<u8>>,
    values: Vec<Vec<u8>>,
    key_len: usize,
    val_len: usize,
}
impl Node {

    pub fn from_storage<T: BinaryStorage + Sized>(
        storage: &mut T,
        offset: u64,
        size_bytes: u64,
        block_size: u64,
        key_len: usize, 
        val_len: usize
    ) -> Result<Node, Error> {

        let node_type = match try!(storage.r_u8(offset + NODE_TYPE_OFFSET)) {
            1 => NodeType::Inner,
            2 => NodeType::Leaf,
            _ => return Err(Error::Assertion(AssertionError::new(ERR_NODE_TYPE_INVALID)))
        };

        let parent_ptr = try!(storage.r_u64(offset + PARENT_PTR_OFFSET));
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
            size_bytes: size_bytes,
            node_type: node_type,
            parent_ptr: parent_ptr,
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

        try!(storage.w_u64(self.offset + PARENT_PTR_OFFSET, self.parent_ptr));
        try!(storage.w_u64(self.offset + NUM_RECORDS_OFFSET, self.num_records));

        let mut rec_data = Vec::new();
        for i in 0..self.num_records {
            rec_data.extend_from_slice(self.keys[i as usize].as_slice());
            rec_data.extend_from_slice(self.values[i as usize].as_slice());
        }

        try!(storage.w_bytes(self.offset + RECORDS_OFFSET, rec_data.as_slice()));

        Ok(())

    }

}

