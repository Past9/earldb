
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

    fn get_capacity(&self) -> usize;

    fn expand(&mut self, min_capacity: usize) -> bool;

}


#[cfg(test)]
pub mod tests {

    use std::{mem, str};

    use storage::binary_storage::BinaryStorage;


    // open(), close(), and is_open() tests 
    pub fn is_closed_when_new<T: BinaryStorage>(s: T) {
        assert!(!s.is_open());
    }

    pub fn is_open_after_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.is_open());
    }

    pub fn is_closed_after_open_and_close<T: BinaryStorage>(mut s: T) {
        s.open();
        s.close();
        assert!(!s.is_open());
    }

    // w_i8() tests
    pub fn w_i8_returns_false_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.w_i8(0, i8::max_value()));
    }

    pub fn w_i8_returns_true_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.w_i8(0, i8::max_value()));
    }

    pub fn w_i8_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        s.w_i8(0, i8::max_value());
        s.open();
        assert_eq!(0, s.r_i8(0).unwrap());
    }

    pub fn w_i8_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(4);
        assert!(!s.w_i8(3, i8::max_value()));
        assert!(s.w_i8(4, i8::max_value()));
        s.set_txn_boundary(16);
        assert_eq!(0, s.r_i8(3).unwrap());
        assert_eq!(i8::max_value(), s.r_i8(4).unwrap());
    }

    pub fn w_i8_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_i8(256, i8::max_value()));
        assert_eq!(512, s.get_capacity());
        assert_eq!(i8::max_value(), s.r_i8(256).unwrap());
    }

    // w_i16() tests
    pub fn w_i16_returns_false_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.w_i16(0, i16::max_value()));
    }

    pub fn w_i16_returns_true_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.w_i16(0, i16::max_value()));
    }

    pub fn w_i16_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        s.w_i16(0, i16::max_value());
        s.open();
        assert_eq!(0, s.r_i16(0).unwrap());
    }

    pub fn w_i16_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(4);
        assert!(!s.w_i16(3, i16::max_value()));
        assert!(s.w_i16(4, i16::max_value()));
        s.set_txn_boundary(16);
        assert_eq!(0, s.r_i16(2).unwrap());
        assert_eq!(i16::max_value(), s.r_i16(4).unwrap());
    }

    pub fn w_i16_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_i16(256, i16::max_value()));
        assert_eq!(512, s.get_capacity());
        assert_eq!(i16::max_value(), s.r_i16(256).unwrap());
    }

    // w_i32() tests
    pub fn w_i32_returns_false_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.w_i32(0, i32::max_value()));
    }

    pub fn w_i32_returns_true_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.w_i32(0, i32::max_value()));
    }

    pub fn w_i32_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        s.w_i32(0, i32::max_value());
        s.open();
        assert_eq!(0, s.r_i32(0).unwrap());
    }

    pub fn w_i32_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(8);
        assert!(!s.w_i32(7, i32::max_value()));
        assert!(s.w_i32(8, i32::max_value()));
        s.set_txn_boundary(16);
        assert_eq!(0, s.r_i32(4).unwrap());
        assert_eq!(i32::max_value(), s.r_i32(8).unwrap());
    }

    pub fn w_i32_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_i32(256, i32::max_value()));
        assert_eq!(512, s.get_capacity());
        assert_eq!(i32::max_value(), s.r_i32(256).unwrap());
    }

    // w_i64() tests
    pub fn w_i64_returns_false_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.w_i64(0, i64::max_value()));
    }

    pub fn w_i64_returns_true_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.w_i64(0, i64::max_value()));
    }

    pub fn w_i64_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        s.w_i64(0, i64::max_value());
        s.open();
        assert_eq!(0, s.r_i64(0).unwrap());
    }

    pub fn w_i64_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(8);
        assert!(!s.w_i64(7, i64::max_value()));
        assert!(s.w_i64(8, i64::max_value()));
        s.set_txn_boundary(16);
        assert_eq!(0, s.r_i64(0).unwrap());
        assert_eq!(i64::max_value(), s.r_i64(8).unwrap());
    }

    pub fn w_i64_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_i64(256, i64::max_value()));
        assert_eq!(512, s.get_capacity());
        assert_eq!(i64::max_value(), s.r_i64(256).unwrap());
    }

    // w_u8() tests
    pub fn w_u8_returns_false_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.w_u8(0, u8::max_value()));
    }

    pub fn w_u8_returns_true_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.w_u8(0, u8::max_value()));
    }

    pub fn w_u8_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        s.w_u8(0, u8::max_value());
        s.open();
        assert_eq!(0, s.r_u8(0).unwrap());
    }

    pub fn w_u8_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(4);
        assert!(!s.w_u8(3, u8::max_value()));
        assert!(s.w_u8(4, u8::max_value()));
        s.set_txn_boundary(16);
        assert_eq!(0, s.r_u8(3).unwrap());
        assert_eq!(u8::max_value(), s.r_u8(4).unwrap());
    }

    pub fn w_u8_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_u8(256, u8::max_value()));
        assert_eq!(512, s.get_capacity());
        assert_eq!(u8::max_value(), s.r_u8(256).unwrap());
    }

    // w_u16() tests
    pub fn w_u16_returns_false_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.w_u16(0, u16::max_value()));
    }

    pub fn w_u16_returns_true_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.w_u16(0, u16::max_value()));
    }

    pub fn w_u16_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        s.w_u16(0, u16::max_value());
        s.open();
        assert_eq!(0, s.r_u16(0).unwrap());
    }

    pub fn w_u16_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(4);
        assert!(!s.w_u16(3, u16::max_value()));
        assert!(s.w_u16(4, u16::max_value()));
        s.set_txn_boundary(16);
        assert_eq!(0, s.r_u16(2).unwrap());
        assert_eq!(u16::max_value(), s.r_u16(4).unwrap());
    }

    pub fn w_u16_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_u16(256, u16::max_value()));
        assert_eq!(512, s.get_capacity());
        assert_eq!(u16::max_value(), s.r_u16(256).unwrap());
    }

    // w_u32() tests
    pub fn w_u32_returns_false_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.w_u32(0, u32::max_value()));
    }

    pub fn w_u32_returns_true_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.w_u32(0, u32::max_value()));
    }

    pub fn w_u32_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        s.w_u32(0, u32::max_value());
        s.open();
        assert_eq!(0, s.r_u32(0).unwrap());
    }

    pub fn w_u32_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(8);
        assert!(!s.w_u32(7, u32::max_value()));
        assert!(s.w_u32(8, u32::max_value()));
        s.set_txn_boundary(16);
        assert_eq!(0, s.r_u32(4).unwrap());
        assert_eq!(u32::max_value(), s.r_u32(8).unwrap());
    }

    pub fn w_u32_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_u32(256, u32::max_value()));
        assert_eq!(512, s.get_capacity());
        assert_eq!(u32::max_value(), s.r_u32(256).unwrap());
    }

    // w_u64() tests
    pub fn w_u64_returns_false_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.w_u64(0, u64::max_value()));
    }

    pub fn w_u64_returns_true_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.w_u64(0, u64::max_value()));
    }

    pub fn w_u64_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        s.w_u64(0, u64::max_value());
        s.open();
        assert_eq!(0, s.r_u64(0).unwrap());
    }

    pub fn w_u64_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(8);
        assert!(!s.w_u64(7, u64::max_value()));
        assert!(s.w_u64(8, u64::max_value()));
        s.set_txn_boundary(16);
        assert_eq!(0, s.r_u64(0).unwrap());
        assert_eq!(u64::max_value(), s.r_u64(8).unwrap());
    }

    pub fn w_u64_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_u64(256, u64::max_value()));
        assert_eq!(512, s.get_capacity());
        assert_eq!(u64::max_value(), s.r_u64(256).unwrap());
    }

    // w_f32() tests
    pub fn w_f32_returns_false_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.w_f32(0, 12345.6789));
    }

    pub fn w_f32_returns_true_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.w_f32(0, 12345.6789));
    }

    pub fn w_f32_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        s.w_f32(0, 12345.6789);
        s.open();
        assert_eq!(0.0, s.r_f32(0).unwrap());
    }

    pub fn w_f32_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(8);
        assert!(!s.w_f32(7, 12345.6789));
        assert!(s.w_f32(8, 12345.6789));
        s.set_txn_boundary(16);
        assert_eq!(0.0, s.r_f32(4).unwrap());
        assert_eq!(12345.6789, s.r_f32(8).unwrap());
    }

    pub fn w_f32_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_f32(256, 12345.6789));
        assert_eq!(512, s.get_capacity());
        assert_eq!(12345.6789, s.r_f32(256).unwrap());
    }

    // w_f64() tests
    pub fn w_f64_returns_false_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.w_f64(0, 12345.6789));
    }

    pub fn w_f64_returns_true_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.w_f64(0, 12345.6789));
    }

    pub fn w_f64_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        s.w_f64(0, 12345.6789);
        s.open();
        assert_eq!(0.0, s.r_f64(0).unwrap());
    }

    pub fn w_f64_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(8);
        assert!(!s.w_f64(7, 12345.6789));
        assert!(s.w_f64(8, 12345.6789));
        s.set_txn_boundary(16);
        assert_eq!(0.0, s.r_f64(0).unwrap());
        assert_eq!(12345.6789, s.r_f64(8).unwrap());
    }

    pub fn w_f64_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_f64(256, 12345.6789));
        assert_eq!(512, s.get_capacity());
        assert_eq!(12345.6789, s.r_f64(256).unwrap());
    }

    // w_bool() tests
    pub fn w_bool_returns_false_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.w_bool(0, false));
        assert!(!s.w_bool(0, true));
    }

    pub fn w_bool_returns_true_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.w_bool(0, false));
        assert!(s.w_bool(0, true));
    }

    pub fn w_bool_does_not_write_when_closed<T: BinaryStorage>(mut s1: T, mut s2: T) {
        s1.w_bool(0, false);
        s1.open();
        assert_eq!(false, s1.r_bool(0).unwrap());

        s2.w_bool(0, true);
        s2.open();
        assert_eq!(false, s2.r_bool(0).unwrap());
    }

    pub fn w_bool_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s1: T, mut s2: T) {
        s1.open();
        s1.set_txn_boundary(4);
        assert!(!s1.w_bool(3, false));
        assert!(s1.w_bool(4, false));
        s1.set_txn_boundary(8);
        assert_eq!(false, s1.r_bool(3).unwrap());
        assert_eq!(false, s1.r_bool(4).unwrap());

        s2.open();
        s2.set_txn_boundary(4);
        assert!(!s2.w_bool(3, true));
        assert!(s2.w_bool(4, true));
        s2.set_txn_boundary(8);
        assert_eq!(false, s2.r_bool(3).unwrap());
        assert_eq!(true, s2.r_bool(4).unwrap());
    }

    pub fn w_bool_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_bool(256, true));
        assert_eq!(512, s.get_capacity());
        assert_eq!(true, s.r_bool(256).unwrap());
    }

    // w_bytes() tests
    pub fn w_bytes_returns_false_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.w_bytes(0, &[0x0, 0x1, 0x2, 0x3, 0x4]));
    }

    pub fn w_bytes_returns_true_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.w_bytes(0, &[0x0, 0x1, 0x2, 0x3, 0x4]));
    }

    pub fn w_bytes_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        s.w_bytes(0, &[0x0, 0x1, 0x2, 0x3, 0x4]);
        s.open();
        assert_eq!(&[0x0, 0x0, 0x0, 0x0, 0x0], s.r_bytes(0, 5).unwrap());
    }

    pub fn w_bytes_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(8);
        assert!(!s.w_bytes(7, &[0x0, 0x1, 0x2, 0x3, 0x4]));
        assert!(s.w_bytes(8, &[0x0, 0x1, 0x2, 0x3, 0x4]));
        s.set_txn_boundary(16);
        assert_eq!(&[0x0, 0x0, 0x0, 0x0, 0x0], s.r_bytes(3, 5).unwrap());
        assert_eq!(&[0x0, 0x1, 0x2, 0x3, 0x4], s.r_bytes(8, 5).unwrap());
    }

    pub fn w_bytes_over_capacity_expands_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_bytes(255, &[0x0, 0x1]));
        assert_eq!(512, s.get_capacity());
        assert_eq!(&[0x0, 0x1], s.r_bytes(255, 2).unwrap());
    }

    pub fn w_bytes_over_capacity_expands_storage_multiple_times<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(256, s.get_capacity());
        assert!(s.w_bytes(255, &[0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6]));
        assert_eq!(264, s.get_capacity());
        assert_eq!(&[0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6], s.r_bytes(255, 7).unwrap());
    }

    // w_str() tests
    pub fn w_str_returns_false_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.w_str(0, "foobar"));
        assert!(!s.w_str(0, "I \u{2661} Rust"));
    }

    pub fn w_str_returns_true_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.w_str(0, "foobar"));
        assert!(s.w_str(0, "I \u{2661} Rust"));
    }

    pub fn w_str_does_not_write_when_closed<T: BinaryStorage>(mut s1: T, mut s2: T) {
        s1.w_str(0, "foobar");
        s1.open();
        assert_eq!(str::from_utf8(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x0]).unwrap(), s1.r_str(0, 6).unwrap());

        s2.w_str(0, "I \u{2661} Rust");
        s2.open();
        assert_eq!(
            str::from_utf8(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0]).unwrap(), 
            s2.r_str(0, 10).unwrap()
        );
    }

    pub fn w_str_does_not_write_before_txn_boundary<T: BinaryStorage>(mut s1: T, mut s2: T) {
        s1.open();
        s1.set_txn_boundary(8);
        assert!(!s1.w_str(7, "foobar"));
        assert!(s1.w_str(8, "foobar"));
        s1.set_txn_boundary(16);
        assert_eq!(str::from_utf8(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x0]).unwrap(), s1.r_str(2, 6).unwrap());
        assert_eq!("foobar", s1.r_str(8, 6).unwrap());

        s2.open();
        s2.set_txn_boundary(16);
        assert!(!s2.w_str(15, "I \u{2661} Rust"));
        assert!(s2.w_str(16, "I \u{2661} Rust"));
        s2.set_txn_boundary(32);
        assert_eq!(
            str::from_utf8(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0]).unwrap(), 
            s2.r_str(6, 10).unwrap()
        );
        assert_eq!("I \u{2661} Rust", s2.r_str(16, 10).unwrap());
    }

    pub fn w_str_over_capacity_expands_storage<T: BinaryStorage>(mut s1: T, mut s2: T) {
        s1.open();
        assert_eq!(256, s1.get_capacity());
        assert!(s1.w_str(255, "foobar"));
        assert_eq!(512, s1.get_capacity());
        assert_eq!("foobar", s1.r_str(255, 6).unwrap());

        s2.open();
        assert_eq!(256, s2.get_capacity());
        assert!(s2.w_str(255, "I \u{2661} Rust"));
        assert_eq!(512, s2.get_capacity());
        assert_eq!("I \u{2661} Rust", s2.r_str(255, 10).unwrap());
    }

    pub fn w_str_over_capacity_expands_storage_multiple_times<T: BinaryStorage>(mut s1: T, mut s2: T) {
        s1.open();
        assert_eq!(256, s1.get_capacity());
        assert!(s1.w_str(255, "foobar"));
        assert_eq!(264, s1.get_capacity());
        assert_eq!("foobar", s1.r_str(255, 6).unwrap());

        s2.open();
        assert_eq!(256, s2.get_capacity());
        assert!(s2.w_str(255, "I \u{2661} Rust"));
        assert_eq!(268, s2.get_capacity());
        assert_eq!("I \u{2661} Rust", s2.r_str(255, 10).unwrap());
    }

    // r_i8() tests
    pub fn r_i8_returns_none_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(s.r_i8(0).is_none());
    }

    pub fn r_i8_returns_some_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_i8(0).is_some());
    }

    pub fn r_i8_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(0, s.r_i8(0).unwrap());
    }

    pub fn r_i8_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open();
        s.w_i8(0, i8::max_value());
        assert_eq!(i8::max_value(), s.r_i8(0).unwrap());
        s.w_i8(32, i8::max_value());
        assert_eq!(i8::max_value(), s.r_i8(32).unwrap());
    }

    pub fn r_i8_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(4);
        assert!(s.r_i8(3).is_some());
        assert!(s.r_i8(4).is_none());
    }

    pub fn r_i8_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_i8(255).is_some());
        assert!(s.r_i8(256).is_none());
    }

    // r_i16() tests
    pub fn r_i16_returns_none_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(s.r_i16(0).is_none());
    }

    pub fn r_i16_returns_some_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_i16(0).is_some());
    }

    pub fn r_i16_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(0, s.r_i16(0).unwrap());
    }

    pub fn r_i16_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open();
        s.w_i16(0, i16::max_value());
        assert_eq!(i16::max_value(), s.r_i16(0).unwrap());
        s.w_i16(32, i16::max_value());
        assert_eq!(i16::max_value(), s.r_i16(32).unwrap());
    }

    pub fn r_i16_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(4);
        assert!(s.r_i16(2).is_some());
        assert!(s.r_i16(3).is_none());
        assert!(s.r_i16(4).is_none());
    }

    pub fn r_i16_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_i16(254).is_some());
        assert!(s.r_i16(255).is_none());
        assert!(s.r_i16(256).is_none());
    }

    // r_i32() tests
    pub fn r_i32_returns_none_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(s.r_i32(0).is_none());
    }

    pub fn r_i32_returns_some_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_i32(0).is_some());
    }

    pub fn r_i32_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(0, s.r_i32(0).unwrap());
    }

    pub fn r_i32_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open();
        s.w_i32(0, i32::max_value());
        assert_eq!(i32::max_value(), s.r_i32(0).unwrap());
        s.w_i32(32, i32::max_value());
        assert_eq!(i32::max_value(), s.r_i32(32).unwrap());
    }

    pub fn r_i32_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(8);
        assert!(s.r_i32(4).is_some());
        assert!(s.r_i32(6).is_none());
        assert!(s.r_i32(8).is_none());
    }

    pub fn r_i32_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_i32(252).is_some());
        assert!(s.r_i32(254).is_none());
        assert!(s.r_i32(256).is_none());
    }

    // r_i64() tests
    pub fn r_i64_returns_none_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(s.r_i64(0).is_none());
    }

    pub fn r_i64_returns_some_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_i64(0).is_some());
    }

    pub fn r_i64_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(0, s.r_i64(0).unwrap());
    }

    pub fn r_i64_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open();
        s.w_i64(0, i64::max_value());
        assert_eq!(i64::max_value(), s.r_i64(0).unwrap());
        s.w_i64(32, i64::max_value());
        assert_eq!(i64::max_value(), s.r_i64(32).unwrap());
    }

    pub fn r_i64_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(16);
        assert!(s.r_i64(8).is_some());
        assert!(s.r_i64(12).is_none());
        assert!(s.r_i64(16).is_none());
    }

    pub fn r_i64_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_i64(248).is_some());
        assert!(s.r_i64(252).is_none());
        assert!(s.r_i64(256).is_none());
    }

    // r_u8() tests
    pub fn r_u8_returns_none_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(s.r_u8(0).is_none());
    }

    pub fn r_u8_returns_some_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_u8(0).is_some());
    }

    pub fn r_u8_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(0, s.r_u8(0).unwrap());
    }

    pub fn r_u8_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open();
        s.w_u8(0, u8::max_value());
        assert_eq!(u8::max_value(), s.r_u8(0).unwrap());
        s.w_u8(32, u8::max_value());
        assert_eq!(u8::max_value(), s.r_u8(32).unwrap());
    }

    pub fn r_u8_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(4);
        assert!(s.r_u8(3).is_some());
        assert!(s.r_u8(4).is_none());
    }

    pub fn r_u8_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_u8(255).is_some());
        assert!(s.r_u8(256).is_none());
    }

    // r_u16() tests
    pub fn r_u16_returns_none_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(s.r_u16(0).is_none());
    }

    pub fn r_u16_returns_some_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_u16(0).is_some());
    }

    pub fn r_u16_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(0, s.r_u16(0).unwrap());
    }

    pub fn r_u16_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open();
        s.w_u16(0, u16::max_value());
        assert_eq!(u16::max_value(), s.r_u16(0).unwrap());
        s.w_u16(32, u16::max_value());
        assert_eq!(u16::max_value(), s.r_u16(32).unwrap());
    }

    pub fn r_u16_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(4);
        assert!(s.r_u16(2).is_some());
        assert!(s.r_u16(3).is_none());
        assert!(s.r_u16(4).is_none());
    }

    pub fn r_u16_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_u16(254).is_some());
        assert!(s.r_u16(255).is_none());
        assert!(s.r_u16(256).is_none());
    }

    // r_u32() tests
    pub fn r_u32_returns_none_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(s.r_u32(0).is_none());
    }

    pub fn r_u32_returns_some_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_u32(0).is_some());
    }

    pub fn r_u32_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(0, s.r_u32(0).unwrap());
    }

    pub fn r_u32_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open();
        s.w_u32(0, u32::max_value());
        assert_eq!(u32::max_value(), s.r_u32(0).unwrap());
        s.w_u32(32, u32::max_value());
        assert_eq!(u32::max_value(), s.r_u32(32).unwrap());
    }

    pub fn r_u32_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(8);
        assert!(s.r_u32(4).is_some());
        assert!(s.r_u32(6).is_none());
        assert!(s.r_u32(8).is_none());
    }

    pub fn r_u32_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_u32(252).is_some());
        assert!(s.r_u32(254).is_none());
        assert!(s.r_u32(256).is_none());
    }

    // r_i64() tests
    pub fn r_u64_returns_none_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(s.r_u64(0).is_none());
    }

    pub fn r_u64_returns_some_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_u64(0).is_some());
    }

    pub fn r_u64_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(0, s.r_u64(0).unwrap());
    }

    pub fn r_u64_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open();
        s.w_u64(0, u64::max_value());
        assert_eq!(u64::max_value(), s.r_u64(0).unwrap());
        s.w_u64(32, u64::max_value());
        assert_eq!(u64::max_value(), s.r_u64(32).unwrap());
    }

    pub fn r_u64_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(16);
        assert!(s.r_u64(8).is_some());
        assert!(s.r_u64(12).is_none());
        assert!(s.r_u64(16).is_none());
    }

    pub fn r_u64_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_u64(248).is_some());
        assert!(s.r_u64(252).is_none());
        assert!(s.r_u64(256).is_none());
    }

    // r_f32() tests
    pub fn r_f32_returns_none_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(s.r_f32(0).is_none());
    }

    pub fn r_f32_returns_some_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_f32(0).is_some());
    }

    pub fn r_f32_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(0.0, s.r_f32(0).unwrap());
    }

    pub fn r_f32_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open();
        s.w_f32(0, 12345.6789);
        assert_eq!(12345.6789, s.r_f32(0).unwrap());
        s.w_f32(32, 12345.6789);
        assert_eq!(12345.6789, s.r_f32(32).unwrap());
    }

    pub fn r_f32_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(8);
        assert!(s.r_f32(4).is_some());
        assert!(s.r_f32(6).is_none());
        assert!(s.r_f32(8).is_none());
    }

    pub fn r_f32_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_f32(252).is_some());
        assert!(s.r_f32(254).is_none());
        assert!(s.r_f32(256).is_none());
    }

    // r_f64() tests
    pub fn r_f64_returns_none_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(s.r_f64(0).is_none());
    }

    pub fn r_f64_returns_some_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_f64(0).is_some());
    }

    pub fn r_f64_reads_zero_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(0.0, s.r_f64(0).unwrap());
    }

    pub fn r_f64_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open();
        s.w_f64(0, 12345.6789);
        assert_eq!(12345.6789, s.r_f64(0).unwrap());
        s.w_f64(32, 12345.6789);
        assert_eq!(12345.6789, s.r_f64(32).unwrap());
    }

    pub fn r_f64_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(16);
        assert!(s.r_f64(8).is_some());
        assert!(s.r_f64(12).is_none());
        assert!(s.r_f64(16).is_none());
    }

    pub fn r_f64_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_f64(248).is_some());
        assert!(s.r_f64(252).is_none());
        assert!(s.r_f64(256).is_none());
    }

    // r_bool() tests
    pub fn r_bool_returns_none_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(s.r_bool(0).is_none());
    }

    pub fn r_bool_returns_some_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_bool(0).is_some());
    }

    pub fn r_bool_reads_false_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(false, s.r_bool(0).unwrap());
    }

    pub fn r_bool_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open();
        s.w_bool(0, false);
        assert_eq!(false, s.r_bool(0).unwrap());
        s.w_bool(32, true);
        assert_eq!(true, s.r_bool(32).unwrap());
    }

    pub fn r_bool_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(8);
        assert!(s.r_bool(7).is_some());
        assert!(s.r_bool(8).is_none());
    }

    pub fn r_bool_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_bool(255).is_some());
        assert!(s.r_bool(256).is_none());
    }

    // r_bytes() tests
    pub fn r_bytes_returns_none_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(s.r_bytes(0, 5).is_none());
    }

    pub fn r_bytes_returns_some_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_bytes(0, 5).is_some());
    }

    pub fn r_bytes_reads_zeros_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(&[0x0, 0x0, 0x0, 0x0, 0x0], s.r_bytes(0, 5).unwrap());
    }

    pub fn r_bytes_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open();
        s.w_bytes(0, &[0x0, 0x1, 0x2, 0x3, 0x4]);
        assert_eq!(&[0x0, 0x1, 0x2, 0x3, 0x4], s.r_bytes(0, 5).unwrap());
        s.w_bytes(32, &[0x5, 0x6, 0x7, 0x8, 0x9]);
        assert_eq!(&[0x5, 0x6, 0x7, 0x8, 0x9], s.r_bytes(32, 5).unwrap());
    }

    pub fn r_bytes_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(8);
        assert!(s.r_bytes(6, 2).is_some());
        assert!(s.r_bytes(7, 2).is_none());
        assert!(s.r_bytes(8, 2).is_none());
    }

    pub fn r_bytes_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_bytes(254, 2).is_some());
        assert!(s.r_bytes(255, 2).is_none());
        assert!(s.r_bytes(256, 2).is_none());
    }

    // r_str() tests
    pub fn r_str_returns_none_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(s.r_str(0, 5).is_none());
    }

    pub fn r_str_returns_some_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_str(0, 5).is_some());
    }

    pub fn r_str_reads_nulls_from_unwritten_storage<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(str::from_utf8(&[0x0, 0x0, 0x0, 0x0, 0x0]).unwrap(), s.r_str(0, 5).unwrap());
    }

    pub fn r_str_reads_written_data<T: BinaryStorage>(mut s: T) {
        s.open();
        s.w_str(0, "foobar");
        assert_eq!("foobar", s.r_str(0, 6).unwrap());
        s.w_str(32, "I \u{2661} Rust");
        assert_eq!("I \u{2661} Rust", s.r_str(32, 10).unwrap());
    }

    pub fn r_str_does_not_read_past_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(8);
        assert!(s.r_str(6, 2).is_some());
        assert!(s.r_str(7, 2).is_none());
        assert!(s.r_str(8, 2).is_none());
    }

    pub fn r_str_does_not_read_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.r_str(254, 2).is_some());
        assert!(s.r_str(255, 2).is_none());
        assert!(s.r_str(256, 2).is_none());
    }

    // fill() tests
    pub fn fill_returns_false_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.fill(None, None, 0x1));
    }

    pub fn fill_does_not_write_when_closed<T: BinaryStorage>(mut s: T) {
        s.fill(None, None, 0x1);
        s.open();
        assert!(s.assert_filled(None, None, 0x0));
    }

    pub fn fill_returns_true_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.fill(None, None, 0x1));
    }

    pub fn fill_repeats_byte_in_storage_range<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.fill(Some(10), Some(20), 0x1));
        assert!(s.assert_filled(Some(0), Some(10), 0x0));
        assert!(s.assert_filled(Some(10), Some(20), 0x1));
        assert!(s.assert_filled(Some(20), None, 0x0));
    }

    pub fn fill_starts_from_beginning_when_start_offset_is_none<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.fill(None, Some(20), 0x1));
        assert!(s.assert_filled(Some(0), Some(20), 0x1));
        assert!(s.assert_filled(Some(20), None, 0x0));
    }

    pub fn fill_goes_to_end_when_end_offset_is_none<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.fill(Some(10), None, 0x1));
        assert!(s.assert_filled(None, Some(10), 0x0));
        assert!(s.assert_filled(Some(10), None, 0x1));
    }

    pub fn fill_returns_false_when_end_offset_is_before_start_offset<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(!s.fill(Some(20), Some(10), 0x1));
    }

    pub fn fill_does_not_write_when_end_offset_is_before_start_offset<T: BinaryStorage>(mut s: T) {
        s.open();
        s.fill(Some(20), Some(10), 0x1);
        assert!(s.assert_filled(None, None, 0x0));
    }

    pub fn fill_returns_false_when_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.get_use_txn_boundary());
        s.set_txn_boundary(10);
        assert!(!s.fill(Some(9), None, 0x1));
    }

    pub fn fill_does_not_write_when_before_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.get_use_txn_boundary());
        s.set_txn_boundary(10);
        s.fill(Some(9), None, 0x1);
        assert!(s.assert_filled(None, None, 0x0));
    }

    pub fn fill_returns_true_when_after_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.get_use_txn_boundary());
        s.set_txn_boundary(10);
        assert!(s.fill(Some(10), None, 0x1));
    }

    pub fn fill_writes_when_after_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.get_use_txn_boundary());
        s.set_txn_boundary(10);
        s.fill(Some(10), None, 0x1);
        assert!(s.assert_filled(None, Some(10), 0x0));
        assert!(s.assert_filled(Some(10), None, 0x1));
    }

    pub fn fill_returns_false_when_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(!s.fill(Some(9), Some(257), 0x1));
    }

    pub fn fill_does_not_write_when_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        s.fill(Some(9), Some(257), 0x1);
        assert!(s.assert_filled(None, None, 0x0));
    }

    pub fn fill_does_not_expand_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        s.fill(Some(9), Some(257), 0x1);
        assert_eq!(256, s.get_capacity());
    }

    // assert_filled() tests
    pub fn assert_filled_retuns_false_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.assert_filled(None, None, 0x0));
    }

    pub fn assert_filled_returns_false_when_start_offset_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.assert_filled(Some(255), None, 0x0));
        assert!(!s.assert_filled(Some(256), None, 0x0));
    }

    pub fn assert_filled_returns_false_when_end_offset_at_or_before_start_offset<T: BinaryStorage>(
        mut s: T
    ) {
        s.open();
        assert!(s.assert_filled(Some(10), Some(11), 0x0));
        assert!(!s.assert_filled(Some(10), Some(10), 0x0));
        assert!(!s.assert_filled(Some(10), Some(9), 0x0));
    }

    pub fn assert_filled_returns_false_when_end_offset_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.assert_filled(Some(10), Some(256), 0x0));
        assert!(!s.assert_filled(Some(10), Some(257), 0x0));
    }

    pub fn assert_filled_checks_whether_all_bytes_in_range_match_value<T: BinaryStorage>(mut s: T) {
        s.open();
        s.fill(Some(10), Some(20), 0x1);
        assert!(s.assert_filled(None, Some(10), 0x0));
        assert!(!s.assert_filled(None, Some(11), 0x0));
        assert!(s.assert_filled(Some(10), Some(20), 0x1));
        assert!(!s.assert_filled(Some(9), Some(20), 0x1));
        assert!(!s.assert_filled(Some(10), Some(21), 0x1));
        assert!(s.assert_filled(Some(20), None, 0x0));
        assert!(!s.assert_filled(Some(19), None, 0x0));
    }

    pub fn assert_filled_starts_from_start_offset<T: BinaryStorage>(mut s: T) {
        s.open();
        s.fill(Some(0), Some(10), 0x1);
        assert!(s.assert_filled(Some(10), None, 0x0));
        assert!(!s.assert_filled(Some(9), None, 0x0));
    }

    pub fn assert_filled_starts_from_beginning_when_start_offset_is_none<T: BinaryStorage>(mut s: T) {
        s.open();
        s.fill(Some(1), None, 0x1);
        assert!(s.assert_filled(None, Some(1), 0x0));
        assert!(!s.assert_filled(Some(1), Some(2), 0x0));
    }

    pub fn assert_filled_goes_to_end_offset<T: BinaryStorage>(mut s: T) {
        s.open();
        s.fill(Some(250), None, 0x1);
        assert!(s.assert_filled(None, Some(250), 0x0));
        assert!(!s.assert_filled(None, Some(251), 0x0));
    }

    pub fn assert_filled_goes_to_end_when_end_offset_is_none<T: BinaryStorage>(mut s: T) {
        s.open();
        s.fill(Some(255), None, 0x1);
        assert!(s.assert_filled(None, Some(255), 0x0));
        assert!(!s.assert_filled(None, None, 0x0));
    }

    // get_use_txn_boundary(), set_use_txn_boundary(), get_txn_boundary(), and set_txn_boundary() tests
    pub fn get_use_txn_boundary_returns_initialized_value<T: BinaryStorage>(mut s1: T, mut s2: T) {
        assert!(!s1.get_use_txn_boundary());
        assert!(s2.get_use_txn_boundary());
    }

    pub fn set_use_txn_boundary_changes_value<T: BinaryStorage>(mut s: T) {
        s.set_use_txn_boundary(true);
        assert!(s.get_use_txn_boundary());
        s.set_use_txn_boundary(false);
        assert!(!s.get_use_txn_boundary());
    }

    pub fn set_use_txn_boundary_resets_boundary_to_zero_when_false<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(10);
        assert_eq!(10, s.get_txn_boundary());
        s.set_use_txn_boundary(false);
        assert_eq!(0, s.get_txn_boundary());
        s.set_use_txn_boundary(true);
        assert_eq!(0, s.get_txn_boundary());
    }

    pub fn get_txn_boundary_starts_at_0_whether_used_or_not<T: BinaryStorage>(mut s1: T, mut s2: T) {
        assert_eq!(0, s1.get_txn_boundary());
        assert_eq!(0, s2.get_txn_boundary());
    }

    pub fn set_txn_boundary_returns_false_when_not_using_txn_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(!s.set_txn_boundary(10));
    }

    pub fn set_txn_boundary_does_not_change_boundary_when_not_using_txn_boundary<T: BinaryStorage>(
        mut s: T
    ) {
        s.open();
        s.set_txn_boundary(10);
        assert_eq!(0, s.get_txn_boundary());
    }

    pub fn set_txn_boundary_returns_false_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.set_txn_boundary(10));
    }

    pub fn set_txn_boundary_does_not_change_boundary_when_closed<T: BinaryStorage>(mut s: T) {
        s.set_txn_boundary(10);
        s.open();
        assert_eq!(0, s.get_txn_boundary());
    }

    pub fn set_txn_boundary_returns_false_when_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(!s.set_txn_boundary(257));
        assert!(s.set_txn_boundary(256));
    }

    pub fn set_txn_boundary_does_not_change_boundary_when_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(257);
        assert_eq!(0, s.get_txn_boundary());
    }

    pub fn set_txn_boundary_does_not_expand_capacity_when_past_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(256, s.get_capacity());
        s.set_txn_boundary(257);
        assert_eq!(256, s.get_capacity());
    }

    pub fn set_txn_boundary_changes_boundary<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_txn_boundary(50);
        assert_eq!(50, s.get_txn_boundary());
        s.set_txn_boundary(25);
        assert_eq!(25, s.get_txn_boundary());
        s.set_txn_boundary(200);
        assert_eq!(200, s.get_txn_boundary());
    }

    // get_expand_size() and set_expand_size() tests
    pub fn get_expand_size_returns_initial_expand_size<T: BinaryStorage>(mut s: T) {
        assert_eq!(512, s.get_expand_size());
    }

    pub fn set_expand_size_returns_false_when_expand_size_is_zero<T: BinaryStorage>(mut s: T) {
        assert!(!s.set_expand_size(0));
    }

    pub fn set_expand_size_does_not_change_expand_size_when_expand_size_is_zero<T: BinaryStorage>(mut s: T) {
        s.set_expand_size(0);
        assert_eq!(512, s.get_expand_size());
    }

    pub fn set_expand_size_returns_false_when_expand_size_is_not_power_of_2<T: BinaryStorage>(mut s: T) {
        assert!(!s.set_expand_size(513));
    }

    pub fn set_expand_size_does_not_change_expand_size_when_expand_size_is_not_power_of_2<T: BinaryStorage>(
        mut s: T
    ) {
        s.set_expand_size(513);
        assert_eq!(512, s.get_expand_size());
    }

    pub fn set_expand_size_returns_true_when_checks_pass<T: BinaryStorage>(mut s: T) {
        assert!(s.set_expand_size(1024));
    }

    pub fn set_expand_size_changes_expand_size_when_checks_pass<T: BinaryStorage>(mut s: T) {
        s.set_expand_size(1024);
        assert_eq!(1024, s.get_expand_size());
    }

    pub fn capacity_increases_to_increments_of_last_set_expand_size<T: BinaryStorage>(mut s: T) {
        s.open();
        s.w_u8(256, 0x1);
        assert_eq!(512, s.get_capacity());
        s.set_expand_size(8);
        s.w_u8(512, 0x1);
        assert_eq!(520, s.get_capacity());
    }

    // get_capacity() tests
    pub fn get_capacity_returns_0_when_closed<T: BinaryStorage>(mut s: T) {
        assert_eq!(0, s.get_capacity());
        s.open();
        s.close();
        assert_eq!(0, s.get_capacity());
    }

    pub fn get_capacity_returns_initial_capacity_when_open<T: BinaryStorage>(mut s: T) {
        s.open();
        assert_eq!(256, s.get_capacity());
    }

    pub fn get_capacity_returns_new_capacity_after_expansion<T: BinaryStorage>(mut s: T) {
        s.open();
        s.w_u8(256, 0x1);
        assert_eq!(512, s.get_capacity());
    }

    // expand() tests
    pub fn expand_returns_false_when_closed<T: BinaryStorage>(mut s: T) {
        assert!(!s.expand(10000));
    }

    pub fn expand_does_not_change_capacity_when_closed<T: BinaryStorage>(mut s: T) {
        s.expand(10000);
        s.open();
        assert_eq!(256, s.get_capacity());
    }

    pub fn expand_returns_true_when_already_has_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_expand_size(16);
        assert!(s.expand(50));
    }

    pub fn expand_does_not_change_capacity_when_already_has_capacity<T: BinaryStorage>(mut s: T) {
        s.open();
        s.set_expand_size(16);
        s.expand(50);
        assert_eq!(256, s.get_capacity());
    }

    pub fn expand_returns_false_when_allocation_arithmetic_overflows<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(!s.expand(usize::max_value()));
    }

    pub fn expand_does_not_change_capacity_when_allocation_arithmetic_overflows<T: BinaryStorage>(mut s: T) {
        s.open();
        s.expand(usize::max_value());
        assert_eq!(256, s.get_capacity());
    }

    pub fn expand_returns_false_when_allocation_fails<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(!s.expand((usize::max_value() - 1024) as usize));
    }

    pub fn expand_does_not_change_capacity_when_allocation_fails<T: BinaryStorage>(mut s: T) {
        s.open();
        s.expand((usize::max_value() - 1024) as usize);
        assert_eq!(256, s.get_capacity());
    }

    pub fn expand_returns_true_when_successful<T: BinaryStorage>(mut s: T) {
        s.open();
        assert!(s.expand(300));
    }

    pub fn expand_changes_capacity_by_expand_size_when_successful<T: BinaryStorage>(mut s: T) {
        s.open();
        s.expand(300);
        assert_eq!(512, s.get_capacity());
    }

    pub fn expand_changes_capacity_by_multiples_of_expand_size_when_successful<T: BinaryStorage>(mut s: T) {
        s.open();
        s.expand(3000);
        assert_eq!(3072, s.get_capacity());
    }


}
