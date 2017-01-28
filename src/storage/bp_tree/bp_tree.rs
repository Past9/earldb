use error::{ Error, AssertionError };

use storage::binary_storage::BinaryStorage;
use storage::bp_tree::bp_storage::BPStorage;
use storage::bp_tree::node::Node;
use storage::bp_tree::inner_node::{ InnerNode, InnerNodeRecord };
use storage::bp_tree::leaf_node::LeafNode;

pub static ERR_EMPTY_INNER_NODE: & 'static str = "Empty inner node";
pub static ERR_NODE_CORRUPTED: & 'static str = "Node data corrupted";

pub struct BPTree<T: BinaryStorage + Sized, F: Fn(&[u8], &[u8]) -> bool> {
    storage: BPStorage<T>,
    block_size: u32,
    key_len: u32,
    val_len: u32,
    key_cmp: F // returns true if GTE
}
impl<T: BinaryStorage + Sized, F: Fn(&[u8], &[u8]) -> bool> BPTree<T, F> {

    pub fn new(
        mut storage: T, 
        block_size: u32, 
        key_len: u32, 
        val_len: u32, 
        key_cmp: F
    ) -> BPTree<T, F> {
        let bp_storage = BPStorage::new(storage, block_size, key_len, val_len);

        BPTree {
            storage: bp_storage,
            block_size: block_size,
            key_len: key_len,
            val_len: val_len,
            key_cmp: key_cmp
        }
    }


    pub fn open(&mut self) -> Result<(), Error> {
        self.storage.open()
        // TODO: Ensure block_size, key_len, and val_len are same as saved index data
    }

    pub fn close(&mut self) -> Result<(), Error> {
        self.storage.close()
    }

    pub fn search(&mut self, k: &[u8]) -> Result<LeafNode, Error> {
        let root = try!(self.storage.read_node(1));
        self.tree_search(k, root)
    }

    fn is_in_range(&self, k: &[u8], r: &InnerNodeRecord) -> Result<bool, Error> {
        match r.min_key {
            None => match r.max_key {
                None => Err(Error::Assertion(AssertionError::new(ERR_NODE_CORRUPTED))),
                Some(ref max) => Ok(!(self.key_cmp)(k, max.as_slice()))
            },
            Some(ref min) => match r.max_key {
                None => Ok((self.key_cmp)(k, min.as_slice())),
                Some(ref max) => Ok(!(self.key_cmp)(k, max.as_slice()))
            }
        }
    }

    fn tree_search(&mut self, k: &[u8], node: Node) -> Result<LeafNode, Error> {
        let inner = match node {
            Node::Leaf(n) => { return Ok(n) },
            Node::Inner(n) => n 
        };

        try!(AssertionError::assert(inner.len() > 0, ERR_EMPTY_INNER_NODE));

        for r in inner {
            if try!(self.is_in_range(k, &r)) {
                let child = try!(self.storage.read_node(r.pointer));
                return self.tree_search(k, child);
            }
        }

        return Err(Error::Assertion(AssertionError::new(ERR_NODE_CORRUPTED)));
    }

}
