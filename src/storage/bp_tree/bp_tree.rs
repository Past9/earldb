
/*
use storage::binary_storage::BinaryStorage;

struct LeafNode {
    storage: T,
    start: u64,
    b_len: u64,
    count: u64,

}

pub struct BPTree<T: BinaryStorage + Sized> {
    storage: T,
    init_block_size: usize,
    block_size: usize,
    key_len: u8,
    val_len: u8
}
impl<T: BinaryStorage + Sized> BPTree<T> {

    pub fn new(mut storage: T, init_block_size: usize, key_len: u8, val_len: u8) -> BPTree<T> {
        storage.set_use_txn_boundary(false);
        BPTree {
            storage: storage,
            init_block_size: init_block_size,
            block_size: init_block_size,
            key_len: key_len,
            val_len: val_len
        }
    }

}
*/
