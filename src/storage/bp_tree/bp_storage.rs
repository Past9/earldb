use error::Error;

use storage::binary_storage::BinaryStorage;
use storage::bp_tree::node::Node;
use storage::bp_tree::inner_node::InnerNode;
use storage::bp_tree::leaf_node::LeafNode;


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

    pub fn read_node(&mut self, block: u32) -> Result<Node, Error> {
        let data = try!(self.storage.r_bytes(
            block as u64 * self.block_size as u64, 
            self.block_size as usize
        ));
        Node::from_bytes(data.as_slice(), block, self.block_size, self.key_len, self.val_len)
    }

    pub fn save_leaf(&mut self, node: LeafNode) -> Result<(), Error> {
        let data = try!(node.to_bytes());
        self.storage.w_bytes(
            node.block() as u64 * self.block_size as u64, 
            data.as_slice()
        )
    }

    pub fn save_inner(&mut self, node: InnerNode) -> Result<(), Error> {
        let data = try!(node.to_bytes());
        self.storage.w_bytes(
            node.block() as u64 * self.block_size as u64, 
            data.as_slice()
        )
    }

}
