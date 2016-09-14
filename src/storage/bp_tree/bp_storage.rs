use error::Error;

use storage::binary_storage::BinaryStorage;
use storage::bp_tree::node::{ Node, NodeType };


pub struct BPStorage<T: BinaryStorage + Sized> {
    storage: T,
    block_size: u64,
    key_len: u64,
    val_len: u64
}
impl<T: BinaryStorage + Sized> BPStorage<T> {

    pub fn new(mut storage: T, block_size: u64, key_len: u64, val_len: u64) -> BPStorage<T> {
        storage.set_use_txn_boundary(false);
        BPStorage {
            storage: storage,
            block_size: block_size,
            key_len: key_len,
            val_len: val_len
        }
    }

    pub fn open(&mut self) -> Result<(), Error> {
        self.storage.open()
    }

    pub fn close(&mut self) -> Result<(), Error> {
        self.storage.close()
    }

    pub fn read_node(&mut self, block_num: u64) -> Result<Node, Error> {
        unimplemented!();
    }

    pub fn save_node(&mut self) -> Result<Node, Error> {
        unimplemented!();
    }


}
