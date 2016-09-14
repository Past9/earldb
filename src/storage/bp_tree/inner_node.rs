

pub struct InnerNode {
    block: u32,
    block_size: u32,
    num_keys: u32,
    is_root: bool,
    keys: Vec<Vec<u8>>,
    child_blocks: Vec<u32>,
    key_len: u32
}
