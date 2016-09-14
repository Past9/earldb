use error::Error;

use storage::binary_storage::BinaryStorage;
use storage::bp_tree::bp_storage::BPStorage;


pub struct BPTree<T: BinaryStorage + Sized> {
    storage: BPStorage<T>,
    block_size: u32,
    key_len: u32,
    val_len: u32
}
impl<T: BinaryStorage + Sized> BPTree<T> {

    pub fn new(mut storage: T, block_size: u32, key_len: u32, val_len: u32) -> BPTree<T> {
        storage.set_use_txn_boundary(false);
        let bp_storage = BPStorage::new(storage, block_size, key_len, val_len);

        BPTree {
            storage: bp_storage,
            block_size: block_size,
            key_len: key_len,
            val_len: val_len,
        }
    }


    pub fn open(&mut self) -> Result<(), Error> {
        self.storage.open()
        // TODO: Ensure block_size, key_len, and val_len are same as saved index data
    }

    pub fn close(&mut self) -> Result<(), Error> {
        self.storage.close()
    }

    pub fn search(&mut self, k: &[u8]) -> Result<Node, Error> {
        let root = try!(self.storage.read_node(1))
        self.tree_search(k, root);
    }

    fn tree_search(&mut self, k: &[u8], node: Node) -> Result<Node, Error> {
        match node.get_node_type() {
            NodeType::Leaf => Ok(node),
            NodeType::Inner => {

            }
        }
    }

}
