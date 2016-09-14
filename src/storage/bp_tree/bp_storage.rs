use error::Error;

use storage::binary_storage::BinaryStorage;
use storage::bp_tree::node::{ Node, NodeType };


pub struct BPStorage<T: BinaryStorage + Sized> {
    storage: T,
    block_size: u32,
    key_len: u32,
    val_len: u32
}
impl<T: BinaryStorage + Sized> BPStorage<T> {

    pub fn new(mut storage: T, block_size: u32, key_len: u32, val_len: u32) -> BPStorage<T> {
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

    pub fn read_node(&mut self, block_num: u32) -> Result<Node, Error> {
        let data = try!(self.storage.r_bytes(
            block_num as u64 * self.block_size as u64, 
            self.block_size as usize
        ));
        Node::from_bytes(data.as_slice(), block_num, self.block_size, self.key_len, self.val_len)
    }

    pub fn save_node(&mut self, node: Node) -> Result<(), Error> {
        let data = try!(node.to_bytes());
        self.storage.w_bytes(
            node.get_block_num() as u64 * self.block_size as u64, 
            data.as_slice()
        )
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
