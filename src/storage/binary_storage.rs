

pub trait BinaryStorage {

    fn open(&mut self) -> bool;
    fn close(&mut self) -> bool;

    fn is_open(&self) -> bool;

    fn w_i8(&mut self, offset: usize, data: i8) -> bool;
    fn w_i16(&mut self, offset: usize, data: i16) -> bool;
    fn w_i32(&mut self, offset: usize, data: i32) -> bool;
    fn w_i64(&mut self, offset: usize, data: i64) -> bool;

    fn w_u8(&mut self, offset: usize, data: u8) -> bool;
    fn w_u16(&mut self, offset: usize, data: u16) -> bool;
    fn w_u32(&mut self, offset: usize, data: u32) -> bool;
    fn w_u64(&mut self, offset: usize, data: u64) -> bool;

    fn w_f32(&mut self, offset: usize, data: f32) -> bool;
    fn w_f64(&mut self, offset: usize, data: f64) -> bool;

    fn w_bool(&mut self, offset: usize, data: bool) -> bool;

    fn w_bytes(&mut self, offset: usize, data: &[u8]) -> bool;
    fn w_str(&mut self, offset: usize, data: &str) -> bool;


    fn r_i8(&self, offset: usize) -> Option<i8>;
    fn r_i16(&self, offset: usize) -> Option<i16>;
    fn r_i32(&self, offset: usize) -> Option<i32>;
    fn r_i64(&self, offset: usize) -> Option<i64>;

    fn r_u8(&self, offset: usize) -> Option<u8>;
    fn r_u16(&self, offset: usize) -> Option<u16>;
    fn r_u32(&self, offset: usize) -> Option<u32>;
    fn r_u64(&self, offset: usize) -> Option<u64>;

    fn r_f32(&self, offset: usize) -> Option<f32>;
    fn r_f64(&self, offset: usize) -> Option<f64>;

    fn r_bool(&self, offset: usize) -> Option<bool>;

    fn r_bytes(&self, offset: usize, len: usize) -> Option<&[u8]>;
    fn r_str(&self, offset: usize, len: usize) -> Option<&str>;

    fn fill(&mut self, start: Option<usize>, end: Option<usize>, val: u8) -> bool;
    fn assert_filled(&self, start: Option<usize>, end: Option<usize>, val: u8) -> bool;

    fn get_use_txn_boundary(&self) -> bool;
    fn set_use_txn_boundary(&mut self, val: bool);

    fn get_txn_boundary(&self) -> usize;
    fn set_txn_boundary(&mut self, offset: usize) -> bool;

    fn get_expand_size(&self) -> usize;
    fn set_expand_size(&mut self, expand_size: usize) -> bool ;

    fn get_align(&self) -> usize;
    fn set_align(&mut self, align: usize) -> bool ;

    fn get_capacity(&self) -> usize;

    fn get_max_page_size(&self) -> usize;

    fn expand(&mut self, min_capacity: usize) -> bool;


}


