use storage::binary_storage::BinaryStorage;
use error::{ Error };


pub struct FileBinaryStorage {

}
impl FileBinaryStorage {

    fn write<T>(&mut self, offset: usize, data: T) -> Result<(), Error> {
        unimplemented!();
    }

    fn read<T: Copy>(&self, offset: usize) -> Result<T, Error> {
        unimplemented!();
    }
}
impl BinaryStorage for FileBinaryStorage {

    fn open(&mut self) -> Result<(), Error> {
        unimplemented!();
    }

    fn close(&mut self) -> Result<(), Error> {
        unimplemented!();
    }

    fn w_i8(&mut self, offset: usize, data: i8) -> Result<(), Error> { self.write(offset, data) }
    fn w_i16(&mut self, offset: usize, data: i16) -> Result<(), Error> { self.write(offset, data) }
    fn w_i32(&mut self, offset: usize, data: i32) -> Result<(), Error> { self.write(offset, data) }
    fn w_i64(&mut self, offset: usize, data: i64) -> Result<(), Error> { self.write(offset, data) }

    fn w_u8(&mut self, offset: usize, data: u8) -> Result<(), Error> { self.write(offset, data) }
    fn w_u16(&mut self, offset: usize, data: u16) -> Result<(), Error> { self.write(offset, data) }
    fn w_u32(&mut self, offset: usize, data: u32) -> Result<(), Error> { self.write(offset, data) }
    fn w_u64(&mut self, offset: usize, data: u64) -> Result<(), Error> { self.write(offset, data) }

    fn w_f32(&mut self, offset: usize, data: f32) -> Result<(), Error> { self.write(offset, data) }
    fn w_f64(&mut self, offset: usize, data: f64) -> Result<(), Error> { self.write(offset, data) }

    fn w_bool(&mut self, offset: usize, data: bool) -> Result<(), Error> { self.write(offset, data) }

    fn w_bytes(&mut self, offset: usize, data: &[u8]) -> Result<(), Error> {
        unimplemented!();
    }

    fn w_str(&mut self, offset: usize, data: &str) -> Result<(), Error> { self.w_bytes(offset, data.as_bytes()) }


    fn r_i8(&self, offset: usize) -> Result<i8, Error> { self.read(offset) }
    fn r_i16(&self, offset: usize) -> Result<i16, Error> { self.read(offset) }
    fn r_i32(&self, offset: usize) -> Result<i32, Error> { self.read(offset) }
    fn r_i64(&self, offset: usize) -> Result<i64, Error> { self.read(offset) }

    fn r_u8(&self, offset: usize) -> Result<u8, Error> { self.read(offset) }
    fn r_u16(&self, offset: usize) -> Result<u16, Error> { self.read(offset) }
    fn r_u32(&self, offset: usize) -> Result<u32, Error> { self.read(offset) }
    fn r_u64(&self, offset: usize) -> Result<u64, Error> { self.read(offset) }

    fn r_f32(&self, offset: usize) -> Result<f32, Error> { self.read(offset) }
    fn r_f64(&self, offset: usize) -> Result<f64, Error> { self.read(offset) }

    fn r_bool(&self, offset: usize) -> Result<bool, Error> { self.read(offset) }

    fn r_bytes(&self, offset: usize, len: usize) -> Result<Vec<u8>, Error> {
        unimplemented!();
    }

    fn r_str(&self, offset: usize, len: usize) -> Result<String, Error> {
        unimplemented!();
    }


    fn fill(&mut self, start: Option<usize>, end: Option<usize>, val: u8) -> Result<(), Error> {
        unimplemented!();
    }

    fn is_filled(&self, start: Option<usize>, end: Option<usize>, val: u8) -> Result<bool, Error> {
        unimplemented!();
    }


    fn get_use_txn_boundary(&self) -> bool {
        unimplemented!();
    }

    fn set_use_txn_boundary(&mut self, val: bool) {
        unimplemented!();
    }


    fn get_txn_boundary(&self) -> Result<usize, Error> {
        unimplemented!();
    }

    fn set_txn_boundary(&mut self, offset: usize) -> Result<(), Error> {
        unimplemented!();
    }


    fn get_expand_size(&self) -> usize {
        unimplemented!();
    }

    fn set_expand_size(&mut self, expand_size: usize) -> Result<(), Error> {
        unimplemented!();
    }


    fn expand(&mut self, min_capacity: usize) -> Result<(), Error> {
        unimplemented!();
    }

    fn get_capacity(&self) -> Result<usize, Error> {
        unimplemented!();
    }

    fn is_open(&self) -> bool {
        unimplemented!();
    }

}


#[cfg(test)]
mod file_binary_storage_tests {

    use storage::binary_storage::tests;
    use storage::binary_storage::BinaryStorage;
    use storage::file_binary_storage::FileBinaryStorage;

    fn get_storage() -> FileBinaryStorage {
        unimplemented!();
    }


    // open(), close(), and is_open() tests 
    #[test]
    pub fn open_returns_err_when_already_open() {
        tests::open_returns_err_when_already_open(
            get_storage()
        );
    }

    #[test]
    pub fn close_returns_err_when_already_closed() {
        tests::close_returns_err_when_already_closed(
            get_storage()
        );
    }

    #[test]
    pub fn open_returns_ok_when_previously_closed() {
        tests::open_returns_ok_when_previously_closed(
            get_storage()
        );
    }

    #[test]
    pub fn close_returns_ok_when_previously_open() {
        tests::close_returns_ok_when_previously_open(
            get_storage()
        );
    }

    #[test]
    fn is_closed_when_new() {
        tests::is_closed_when_new(
            get_storage()
        );
    }

    #[test]
    fn is_open_after_open() {
        tests::is_open_after_open(
            get_storage()
        );
    }

    #[test]
    fn is_closed_after_open_and_close() {
        tests::is_closed_after_open_and_close(
            get_storage()
        );
    }

    // new() tests
    // TODO: Write these

    // w_i8() tests
    #[test]
    fn w_i8_returns_err_when_closed() {
        tests::w_i8_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_i8_returns_ok_when_open() {
        tests::w_i8_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_i8_does_not_write_when_closed() {
        tests::w_i8_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_i8_does_not_write_before_txn_boundary() {
        tests::w_i8_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_i8_over_capacity_expands_storage() {
        tests::w_i8_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_i16() tests
    #[test]
    fn w_i16_returns_err_when_closed() {
        tests::w_i16_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_i16_returns_ok_when_open() {
        tests::w_i16_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_i16_does_not_write_when_closed() {
        tests::w_i16_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_i16_does_not_write_before_txn_boundary() {
        tests::w_i16_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_i16_over_capacity_expands_storage() {
        tests::w_i16_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_i32() tests
    #[test]
    fn w_i32_returns_err_when_closed() {
        tests::w_i32_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_i32_returns_ok_when_open() {
        tests::w_i32_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_i32_does_not_write_when_closed() {
        tests::w_i32_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_i32_does_not_write_before_txn_boundary() {
        tests::w_i32_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_i32_over_capacity_expands_storage() {
        tests::w_i32_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_i64() tests
    #[test]
    fn w_i64_returns_err_when_closed() {
        tests::w_i64_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_i64_returns_ok_when_open() {
        tests::w_i64_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_i64_does_not_write_when_closed() {
        tests::w_i64_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_i64_does_not_write_before_txn_boundary() {
        tests::w_i64_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_i64_over_capacity_expands_storage() {
        tests::w_i64_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_u8() tests
    #[test]
    fn w_u8_returns_err_when_closed() {
        tests::w_u8_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_u8_returns_ok_when_open() {
        tests::w_u8_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_u8_does_not_write_when_closed() {
        tests::w_u8_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_u8_does_not_write_before_txn_boundary() {
        tests::w_u8_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_u8_over_capacity_expands_storage() {
        tests::w_u8_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_u16() tests
    #[test]
    fn w_u16_returns_err_when_closed() {
        tests::w_u16_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_u16_returns_ok_when_open() {
        tests::w_u16_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_u16_does_not_write_when_closed() {
        tests::w_u16_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_u16_does_not_write_before_txn_boundary() {
        tests::w_u16_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_u16_over_capacity_expands_storage() {
        tests::w_u16_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_u32() tests
    #[test]
    fn w_u32_returns_err_when_closed() {
        tests::w_u32_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_u32_returns_ok_when_open() {
        tests::w_u32_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_u32_does_not_write_when_closed() {
        tests::w_u32_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_u32_does_not_write_before_txn_boundary() {
        tests::w_u32_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_u32_over_capacity_expands_storage() {
        tests::w_u32_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_u64() tests
    #[test]
    fn w_u64_returns_err_when_closed() {
        tests::w_u64_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_u64_returns_ok_when_open() {
        tests::w_u64_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_u64_does_not_write_when_closed() {
        tests::w_u64_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_u64_does_not_write_before_txn_boundary() {
        tests::w_u64_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_u64_over_capacity_expands_storage() {
        tests::w_u64_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_f32() tests
    #[test]
    fn w_f32_returns_err_when_closed() {
        tests::w_f32_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_f32_returns_ok_when_open() {
        tests::w_f32_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_f32_does_not_write_when_closed() {
        tests::w_f32_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_f32_does_not_write_before_txn_boundary() {
        tests::w_f32_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_f32_over_capacity_expands_storage() {
        tests::w_f32_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_f64() tests
    #[test]
    fn w_f64_returns_err_when_closed() {
        tests::w_f64_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_f64_returns_ok_when_open() {
        tests::w_f64_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_f64_does_not_write_when_closed() {
        tests::w_f64_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_f64_does_not_write_before_txn_boundary() {
        tests::w_f64_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_f64_over_capacity_expands_storage() {
        tests::w_f64_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_bool() tests
    #[test]
    fn w_bool_returns_err_when_closed() {
        tests::w_bool_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_bool_returns_ok_when_open() {
        tests::w_bool_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_bool_does_not_write_when_closed() {
        tests::w_bool_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_bool_does_not_write_before_txn_boundary() {
        tests::w_bool_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_bool_over_capacity_expands_storage() {
        tests::w_bool_over_capacity_expands_storage(
            get_storage()
        );
    }

    // w_bytes() tests
    #[test]
    fn w_bytes_returns_err_when_closed() {
        tests::w_bytes_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_bytes_returns_ok_when_open() {
        tests::w_bytes_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_bytes_does_not_write_when_closed() {
        tests::w_bytes_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_bytes_does_not_write_before_txn_boundary() {
        tests::w_bytes_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_bytes_over_capacity_expands_storage() {
        tests::w_bytes_over_capacity_expands_storage(
            get_storage()
        );
    }

    #[test]
    fn w_bytes_over_capacity_expands_storage_multiple_times() {
        tests::w_bytes_over_capacity_expands_storage_multiple_times(
            get_storage()
        );
    }

    // w_str() tests
    #[test]
    fn w_str_returns_err_when_closed() {
        tests::w_str_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_str_returns_ok_when_open() {
        tests::w_str_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn w_str_does_not_write_when_closed() {
        tests::w_str_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn w_str_does_not_write_before_txn_boundary() {
        tests::w_str_does_not_write_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn w_str_over_capacity_expands_storage() {
        tests::w_str_over_capacity_expands_storage(
            get_storage()
        );
    }

    #[test]
    fn w_str_over_capacity_expands_storage_multiple_times() {
        tests::w_str_over_capacity_expands_storage_multiple_times(
            get_storage()
        );
    }

    // r_i8() tests
    #[test]
    fn r_i8_returns_err_when_closed() {
        tests::r_i8_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_i8_returns_ok_when_open() {
        tests::r_i8_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_i8_reads_zero_from_unwritten_storage() {
        tests::r_i8_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_i8_reads_written_data() {
        tests::r_i8_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_i8_does_not_read_past_txn_boundary() {
        tests::r_i8_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_i8_does_not_read_past_capacity() {
        tests::r_i8_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_i8_result_is_not_mutated_on_subsequent_write() {
        tests::r_i8_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_i16() tests
    #[test]
    fn r_i16_returns_err_when_closed() {
        tests::r_i16_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_i16_returns_ok_when_open() {
        tests::r_i16_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_i16_reads_zero_from_unwritten_storage() {
        tests::r_i16_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_i16_reads_written_data() {
        tests::r_i16_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_i16_does_not_read_past_txn_boundary() {
        tests::r_i16_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_i16_does_not_read_past_capacity() {
        tests::r_i16_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_i16_result_is_not_mutated_on_subsequent_write() {
        tests::r_i16_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_i32() tests
    #[test]
    fn r_i32_returns_err_when_closed() {
        tests::r_i32_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_i32_returns_ok_when_open() {
        tests::r_i32_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_i32_reads_zero_from_unwritten_storage() {
        tests::r_i32_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_i32_reads_written_data() {
        tests::r_i32_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_i32_does_not_read_past_txn_boundary() {
        tests::r_i32_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_i32_does_not_read_past_capacity() {
        tests::r_i32_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_i32_result_is_not_mutated_on_subsequent_write() {
        tests::r_i32_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_i64() tests
    #[test]
    fn r_i64_returns_err_when_closed() {
        tests::r_i64_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_i64_returns_ok_when_open() {
        tests::r_i64_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_i64_reads_zero_from_unwritten_storage() {
        tests::r_i64_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_i64_reads_written_data() {
        tests::r_i64_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_i64_does_not_read_past_txn_boundary() {
        tests::r_i64_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_i64_does_not_read_past_capacity() {
        tests::r_i64_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_i64_result_is_not_mutated_on_subsequent_write() {
        tests::r_i64_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_u8() tests
    #[test]
    fn r_u8_returns_err_when_closed() {
        tests::r_u8_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_u8_returns_ok_when_open() {
        tests::r_u8_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_u8_reads_zero_from_unwritten_storage() {
        tests::r_u8_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_u8_reads_written_data() {
        tests::r_u8_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_u8_does_not_read_past_txn_boundary() {
        tests::r_u8_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_u8_does_not_read_past_capacity() {
        tests::r_u8_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_u8_result_is_not_mutated_on_subsequent_write() {
        tests::r_u8_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_u16() tests
    #[test]
    fn r_u16_returns_err_when_closed() {
        tests::r_u16_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_u16_returns_ok_when_open() {
        tests::r_u16_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_u16_reads_zero_from_unwritten_storage() {
        tests::r_u16_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_u16_reads_written_data() {
        tests::r_u16_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_u16_does_not_read_past_txn_boundary() {
        tests::r_u16_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_u16_does_not_read_past_capacity() {
        tests::r_u16_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_u16_result_is_not_mutated_on_subsequent_write() {
        tests::r_u16_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_u32() tests
    #[test]
    fn r_u32_returns_err_when_closed() {
        tests::r_u32_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_u32_returns_ok_when_open() {
        tests::r_u32_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_u32_reads_zero_from_unwritten_storage() {
        tests::r_u32_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_u32_reads_written_data() {
        tests::r_u32_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_u32_does_not_read_past_txn_boundary() {
        tests::r_u32_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_u32_does_not_read_past_capacity() {
        tests::r_u32_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_u32_result_is_not_mutated_on_subsequent_write() {
        tests::r_u32_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_u64() tests
    #[test]
    fn r_u64_returns_err_when_closed() {
        tests::r_u64_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_u64_returns_ok_when_open() {
        tests::r_u64_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_u64_reads_zero_from_unwritten_storage() {
        tests::r_u64_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_u64_reads_written_data() {
        tests::r_u64_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_u64_does_not_read_past_txn_boundary() {
        tests::r_u64_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_u64_does_not_read_past_capacity() {
        tests::r_u64_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_u64_result_is_not_mutated_on_subsequent_write() {
        tests::r_u64_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_f32() tests
    #[test]
    fn r_f32_returns_err_when_closed() {
        tests::r_f32_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_f32_returns_ok_when_open() {
        tests::r_f32_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_f32_reads_zero_from_unwritten_storage() {
        tests::r_f32_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_f32_reads_written_data() {
        tests::r_f32_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_f32_does_not_read_past_txn_boundary() {
        tests::r_f32_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_f32_does_not_read_past_capacity() {
        tests::r_f32_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_f32_result_is_not_mutated_on_subsequent_write() {
        tests::r_f32_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_f64() tests
    #[test]
    fn r_f64_returns_err_when_closed() {
        tests::r_f64_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_f64_returns_ok_when_open() {
        tests::r_f64_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_f64_reads_zero_from_unwritten_storage() {
        tests::r_f64_reads_zero_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_f64_reads_written_data() {
        tests::r_f64_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_f64_does_not_read_past_txn_boundary() {
        tests::r_f64_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_f64_does_not_read_past_capacity() {
        tests::r_f64_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_f64_result_is_not_mutated_on_subsequent_write() {
        tests::r_f64_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_bool() tests
    #[test]
    fn r_bool_returns_err_when_closed() {
        tests::r_bool_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_bool_returns_ok_when_open() {
        tests::r_bool_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_bool_reads_false_from_unwritten_storage() {
        tests::r_bool_reads_false_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_bool_reads_written_data() {
        tests::r_bool_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_bool_does_not_read_past_txn_boundary() {
        tests::r_bool_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_bool_does_not_read_past_capacity() {
        tests::r_bool_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_bool_result_is_not_mutated_on_subsequent_write() {
        tests::r_bool_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_bytes() tests
    #[test]
    fn r_bytes_returns_err_when_closed() {
        tests::r_bytes_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_bytes_returns_ok_when_open() {
        tests::r_bytes_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_bytes_reads_zeros_from_unwritten_storage() {
        tests::r_bytes_reads_zeros_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_bytes_reads_written_data() {
        tests::r_bytes_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_bytes_does_not_read_past_txn_boundary() {
        tests::r_bytes_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_bytes_does_not_read_past_capacity() {
        tests::r_bytes_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_bytes_result_is_not_mutated_on_subsequent_write() {
        tests::r_bytes_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // r_str() tests
    #[test]
    fn r_str_returns_err_when_closed() {
        tests::r_str_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn r_str_returns_ok_when_open() {
        tests::r_str_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn r_str_reads_nulls_from_unwritten_storage() {
        tests::r_str_reads_nulls_from_unwritten_storage(
            get_storage()
        );
    }

    #[test]
    fn r_str_reads_written_data() {
        tests::r_str_reads_written_data(
            get_storage()
        );
    }

    #[test]
    fn r_str_does_not_read_past_txn_boundary() {
        tests::r_str_does_not_read_past_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn r_str_does_not_read_past_capacity() {
        tests::r_str_does_not_read_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn r_str_result_is_not_mutated_on_subsequent_write() {
        tests::r_str_result_is_not_mutated_on_subsequent_write(
            get_storage()
        );
    }

    // fill() tests
    #[test]
    fn fill_returns_err_when_closed() {
        tests::fill_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn fill_does_not_write_when_closed() {
        tests::fill_does_not_write_when_closed(
            get_storage()
        );
    }

    #[test]
    fn fill_returns_ok_when_open() {
        tests::fill_returns_ok_when_open(
            get_storage()
        );
    }

    #[test]
    fn fill_repeats_byte_in_storage_range() {
        tests::fill_repeats_byte_in_storage_range(
            get_storage()
        );
    }

    #[test]
    fn fill_starts_from_beginning_when_start_offset_is_none() {
        tests::fill_starts_from_beginning_when_start_offset_is_none(
            get_storage()
        );
    }

    #[test]
    fn fill_goes_to_end_when_end_offset_is_none() {
        tests::fill_goes_to_end_when_end_offset_is_none(
            get_storage()
        );
    }

    #[test]
    fn fill_returns_err_when_end_offset_is_before_start_offset() {
        tests::fill_returns_err_when_end_offset_is_before_start_offset(
            get_storage()
        );
    }

    #[test]
    fn fill_does_not_write_when_end_offset_is_before_start_offset() {
        tests::fill_does_not_write_when_end_offset_is_before_start_offset(
            get_storage()
        );
    }

    #[test]
    fn fill_returns_err_when_before_txn_boundary() {
        tests::fill_returns_err_when_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn fill_does_not_write_when_before_txn_boundary() {
        tests::fill_does_not_write_when_before_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn fill_returns_ok_when_after_txn_boundary() {
        tests::fill_returns_ok_when_after_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn fill_writes_when_after_txn_boundary() {
        tests::fill_writes_when_after_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn fill_returns_err_when_past_capacity() {
        tests::fill_returns_err_when_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn fill_does_not_write_when_past_capacity() {
        tests::fill_does_not_write_when_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn fill_does_not_expand_capacity() {
        tests::fill_does_not_expand_capacity(
            get_storage()
        );
    }

    // assert_filled() tests
    #[test]
    fn is_filled_retuns_err_when_closed() {
        tests::is_filled_retuns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn is_filled_returns_err_when_start_offset_past_capacity() {
        tests::is_filled_returns_err_when_start_offset_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn is_filled_returns_err_when_end_offset_at_or_before_start_offset() {
        tests::is_filled_returns_err_when_end_offset_at_or_before_start_offset(
            get_storage()
        );
    }

    #[test]
    fn is_filled_returns_err_when_end_offset_past_capacity() {
        tests::is_filled_returns_err_when_end_offset_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn is_filled_checks_whether_all_bytes_in_range_match_value() {
        tests::is_filled_checks_whether_all_bytes_in_range_match_value(
            get_storage()
        );
    }

    #[test]
    fn is_filled_starts_from_start_offset() {
        tests::is_filled_starts_from_start_offset(
            get_storage()
        );
    }

    #[test]
    fn is_filled_starts_from_beginning_when_start_offset_is_none() {
        tests::is_filled_starts_from_beginning_when_start_offset_is_none(
            get_storage()
        );
    }

    #[test]
    fn is_filled_goes_to_end_offset() {
        tests::is_filled_goes_to_end_offset(
            get_storage()
        );
    }

    #[test]
    fn is_filled_goes_to_end_when_end_offset_is_none() {
        tests::is_filled_goes_to_end_when_end_offset_is_none(
            get_storage()
        );
    }

    // get_use_txn_boundary(), set_use_txn_boundary(), get_txn_boundary(), and set_txn_boundary() tests
    #[test]
    fn set_use_txn_boundary_changes_value() {
        tests::set_use_txn_boundary_changes_value(
            get_storage()
        );
    }

    #[test]
    fn set_use_txn_boundary_resets_boundary_to_zero_when_txn_boundary_turned_off() {
        tests::set_use_txn_boundary_resets_boundary_to_zero_when_txn_boundary_turned_off(
            get_storage()
        );
    }

    #[test]
    fn get_txn_boundary_returns_err_when_closed() {
        tests::get_txn_boundary_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn get_txn_boundary_returns_err_when_not_using_txn_boundary() {
        tests::get_txn_boundary_returns_err_when_not_using_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn get_txn_boundary_starts_at_0() {
        tests::get_txn_boundary_starts_at_0(
            get_storage()
        );
    }

    #[test]
    fn set_txn_boundary_returns_err_when_not_using_txn_boundary() {
        tests::set_txn_boundary_returns_err_when_not_using_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn set_txn_boundary_does_not_change_boundary_when_not_using_txn_boundary() {
        tests::set_txn_boundary_does_not_change_boundary_when_not_using_txn_boundary(
            get_storage()
        );
    }

    #[test]
    fn set_txn_boundary_returns_err_when_closed() {
        tests::set_txn_boundary_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn set_txn_boundary_does_not_change_boundary_when_closed() {
        tests::set_txn_boundary_does_not_change_boundary_when_closed(
            get_storage()
        );
    }

    #[test]
    fn set_txn_boundary_returns_err_when_past_capacity() {
        tests::set_txn_boundary_returns_err_when_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn set_txn_boundary_does_not_change_boundary_when_past_capacity() {
        tests::set_txn_boundary_does_not_change_boundary_when_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn set_txn_boundary_does_not_expand_capacity_when_past_capacity() {
        tests::set_txn_boundary_does_not_expand_capacity_when_past_capacity(
            get_storage()
        );
    }

    #[test]
    fn set_txn_boundary_changes_boundary() {
        tests::set_txn_boundary_changes_boundary(
            get_storage()
        );
    }

    // get_expand_size() and set_expand_size() tests
    #[test]
    fn get_expand_size_returns_initial_expand_size() {
        tests::get_expand_size_returns_initial_expand_size(
            get_storage()
        );
    }

    #[test]
    fn set_expand_size_returns_err_when_expand_size_is_zero() {
        tests::set_expand_size_returns_err_when_expand_size_is_zero(
            get_storage()
        );
    }

    #[test]
    fn set_expand_size_does_not_change_expand_size_when_expand_size_is_zero() {
        tests::set_expand_size_does_not_change_expand_size_when_expand_size_is_zero(
            get_storage()
        );
    }

    #[test]
    fn set_expand_size_returns_err_when_expand_size_is_not_power_of_2() {
        tests::set_expand_size_returns_err_when_expand_size_is_not_power_of_2(
            get_storage()
        );
    }

    #[test]
    fn set_expand_size_does_not_change_expand_size_when_expand_size_is_not_power_of_2() {
        tests::set_expand_size_does_not_change_expand_size_when_expand_size_is_not_power_of_2(
            get_storage()
        );
    }

    #[test]
    fn set_expand_size_returns_true_when_checks_pass() {
        tests::set_expand_size_returns_true_when_checks_pass(
            get_storage()
        );
    }

    #[test]
    fn set_expand_size_changes_expand_size_when_checks_pass() {
        tests::set_expand_size_changes_expand_size_when_checks_pass(
            get_storage()
        );
    }

    #[test]
    fn capacity_increases_to_increments_of_last_set_expand_size() {
        tests::capacity_increases_to_increments_of_last_set_expand_size(
            get_storage()
        );
    }

    // get_capacity() tests
    #[test]
    fn get_capacity_returns_err_when_closed() {
        tests::get_capacity_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn get_capacity_returns_initial_capacity_when_open() {
        tests::get_capacity_returns_initial_capacity_when_open(
            get_storage()
        );
    }

    #[test]
    fn get_capacity_returns_new_capacity_after_expansion() {
        tests::get_capacity_returns_new_capacity_after_expansion(
            get_storage()
        );
    }

    // expand() tests
    #[test]
    fn expand_returns_err_when_closed() {
        tests::expand_returns_err_when_closed(
            get_storage()
        );
    }

    #[test]
    fn expand_does_not_change_capacity_when_closed() {
        tests::expand_does_not_change_capacity_when_closed(
            get_storage()
        );
    }

    #[test]
    fn expand_returns_ok_when_already_has_capacity() {
        tests::expand_returns_ok_when_already_has_capacity(
            get_storage()
        );
    }

    #[test]
    fn expand_does_not_change_capacity_when_already_has_capacity() {
        tests::expand_does_not_change_capacity_when_already_has_capacity(
            get_storage()
        );
    }

    #[test]
    fn expand_returns_err_when_allocation_arithmetic_overflows() {
        tests::expand_returns_err_when_allocation_arithmetic_overflows(
            get_storage()
        );
    }

    #[test]
    fn expand_does_not_change_capacity_when_allocation_arithmetic_overflows() {
        tests::expand_does_not_change_capacity_when_allocation_arithmetic_overflows(
            get_storage()
        );
    }

    #[test]
    fn expand_returns_err_when_allocation_fails() {
        tests::expand_returns_err_when_allocation_fails(
            get_storage()
        );
    }

    #[test]
    fn expand_does_not_change_capacity_when_allocation_fails() {
        tests::expand_does_not_change_capacity_when_allocation_fails(
            get_storage()
        );
    }

    #[test]
    fn expand_returns_ok_when_successful() {
        tests::expand_returns_ok_when_successful(
            get_storage()
        );
    }

    #[test]
    fn expand_changes_capacity_by_expand_size_when_successful() {
        tests::expand_changes_capacity_by_expand_size_when_successful(
            get_storage()
        );
    }

    #[test]
    fn expand_changes_capacity_by_multiples_of_expand_size_when_successful() {
        tests::expand_changes_capacity_by_multiples_of_expand_size_when_successful(
            get_storage()
        );
    }

}

