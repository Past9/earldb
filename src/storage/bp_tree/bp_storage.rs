/*
use error::Error;

use storage::binary_storage::BinaryStorage;
use storage::bp_tree::node::{ Node, NodeType };


pub struct BPStorage<T: BinaryStorage + Sized> {
    storage: T,
    block_size: u64,
    key_len: usize,
    val_len: usize
}
impl<T: BinaryStorage + Sized> BPStorage<T> {

    pub fn new(mut storage: T, block_size: u64, key_len: usize, val_len: usize) -> BPStorage<T> {
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
        Node::from_storage(
            &mut self.storage,
            block_num * self.block_size,
            self.block_size,
            self.key_len,
            self.val_len
        )
    }

    pub fn save_node(&mut self, node: Node) -> Result<(), Error> {
        node.to_storage(&mut self.storage)
    }

    pub fn read_parent(&mut self, node: &Node) -> Option<Result<Node, Error>> {
        match node.get_parent_block_num() {
            Some(n) => Some(self.read_node(n)),
            None => None
        }
    }

    pub fn read_prev(&mut self, node: &Node) -> Option<Result<Node, Error>> {
        match node.get_prev_block_num() {
            Some(n) => Some(self.read_node(n)),
            None => None
        }
    }

    pub fn read_next(&mut self, node: &Node) -> Option<Result<Node, Error>> {
        match node.get_next_block_num() {
            Some(n) => Some(self.read_node(n)),
            None => None
        }
    }


}
*/
