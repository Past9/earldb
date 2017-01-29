use error::Error;

use storage::binary_storage::BinaryStorage;
use storage::bp_tree::node::Node;
use storage::bp_tree::inner_node::InnerNode;
use storage::bp_tree::leaf_node::LeafNode;


pub struct BPStorage<T: BinaryStorage + Sized> {
    storage: T,
    node_size: u64,
    key_len: u8,
    val_len: u8
}
impl<T: BinaryStorage + Sized> BPStorage<T> {

    pub fn new(mut storage: T, node_size: u64, key_len: u8, val_len: u8) -> BPStorage<T> {
        BPStorage {
            storage: storage,
            node_size: node_size,
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

    pub fn read_node(&mut self, node_ptr: u64) -> Result<Node, Error> {
        let data = try!(self.storage.r_bytes(
            node_ptr, 
            self.node_size as usize
        ));
        Node::from_bytes(data.as_slice(), node_ptr, self.key_len, self.val_len)
    }

    pub fn save_leaf(&mut self, node: LeafNode) -> Result<(), Error> {
        let data = try!(node.to_bytes());
        self.storage.w_bytes(
            node.node_ptr(), 
            data.as_slice()
        )
    }

    pub fn save_inner(&mut self, node: InnerNode) -> Result<(), Error> {
        let data = try!(node.to_bytes());
        self.storage.w_bytes(
            node.node_ptr(), 
            data.as_slice()
        )
    }

}
