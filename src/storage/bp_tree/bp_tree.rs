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
  node_size: u64,
  key_len: u8,
  val_len: u8,
  key_cmp: F // returns true if GTE
}
impl<T: BinaryStorage + Sized, F: Fn(&[u8], &[u8]) -> bool> BPTree<T, F> {

  pub fn new(
    mut storage: T, 
    node_size: u64, 
    key_len: u8, 
    val_len: u8, 
    key_cmp: F
  ) -> BPTree<T, F> {
    let bp_storage = BPStorage::new(storage, node_size, key_len, val_len);

    BPTree {
      storage: bp_storage,
      node_size: node_size,
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

  pub fn search(&self, k: &[u8]) -> Result<LeafNode, Error> {
    let root = try!(self.storage.read_node(0));
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

  fn tree_search(&self, k: &[u8], node: Node) -> Result<LeafNode, Error> {
    let inner = match node {
      Node::Leaf(l) => { return Ok(l); },
      Node::Inner(i) => i
    };

    try!(AssertionError::assert(inner.len() > 0, ERR_EMPTY_INNER_NODE));

    for record in inner.into_iter() {
      match (record.min_key, record.max_key) {
        // First record
        (None, Some(max)) => {
          if k < max.as_slice() { 
            return self.tree_search(k, try!(self.storage.read_node(record.pointer))); 
          }
        },
        // Any middle record
        (Some(min), Some(max)) => {
          if min.as_slice() <= k && k < max.as_slice() { 
            return self.tree_search(k, try!(self.storage.read_node(record.pointer))); 
          }
        },
        // Last record
        (Some(min), None) => {
          if min.as_slice() <= k { 
            return self.tree_search(k, try!(self.storage.read_node(record.pointer))); 
          }
        },
        // Impossible situation
        (None, None) => {
          return Err(Error::Assertion(AssertionError::new(ERR_NODE_CORRUPTED)));
        }
      }
    };

    return Err(Error::Assertion(AssertionError::new(ERR_NODE_CORRUPTED)));
  }

}
