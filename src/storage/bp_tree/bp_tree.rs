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

    pub fn search(&mut self, min: &[u8], max: &[u8]) -> Result<Vec<Vec<u8>>, Error> {
        unimplemented!();
    }

}
