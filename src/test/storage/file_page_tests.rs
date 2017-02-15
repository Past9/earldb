use storage::file_page::FilePage;

// FilePage::new() tests
#[test]
fn new_returns_none_when_max_size_not_power_of_2() {
  let p = FilePage::new(257);
  assert!(p.is_none());
}

#[test]
fn new_returns_file_page_instance_when_checks_pass() {
  let p = FilePage::new(256);
  assert!(p.is_some());
}

#[test]
fn new_sets_max_size() {
  let p = FilePage::new(512).unwrap();
  assert_eq!(512, p.get_max_size());
}

#[test]
fn new_inits_memory_to_zeros() {
  let mut p = FilePage::new(256).unwrap();
  p.write(255, &[0x0]);
  let data = p.read(0, 256);
  assert_eq!(256, data.len());
  for b in data {
    assert_eq!(b, 0x0);
  }
}

// FilePage::read() tests
#[test]
fn read_returns_empty_when_new() {
  let p = FilePage::new(256).unwrap();
  let data = p.read(0, 4);
  assert_eq!(0, data.len());
}

#[test]
fn read_returns_empty_when_reading_from_past_actual_size() {
  let mut p = FilePage::new(256).unwrap();
  p.write(0, &[0x0, 0x1, 0x2, 0x3]);
  let data = p.read(4, 4);
  assert_eq!(0, data.len());
}

#[test]
fn read_returns_remaining_data_when_past_actual_size() {
  let mut p = FilePage::new(256).unwrap();
  p.write(0, &[0x0, 0x1, 0x2, 0x3]);
  let data = p.read(2, 4);
  assert_eq!(2, data.len());
  assert_eq!(vec!(0x2, 0x3), data);
}

#[test]
fn read_past_actual_size_does_not_increase_actual_size() {
  let mut p = FilePage::new(256).unwrap();
  p.write(0, &[0x0, 0x1, 0x2, 0x3]);
  assert_eq!(4, p.get_actual_size());
  p.read(2, 4);
  assert_eq!(4, p.get_actual_size());
}

#[test]
fn read_returns_zeros_for_unwritten_data() {
  let mut p = FilePage::new(256).unwrap();
  p.write(255, &[0x1]);
  assert_eq!(vec!(0x0, 0x0, 0x0, 0x0), p.read(0, 4));
  assert_eq!(vec!(0x0, 0x0, 0x0, 0x0), p.read(64, 4));
  assert_eq!(vec!(0x0, 0x0, 0x0, 0x0), p.read(128, 4));
  assert_eq!(vec!(0x0, 0x0, 0x0, 0x1), p.read(252, 4));
}

#[test]
fn read_returns_written_data() {
  let mut p = FilePage::new(256).unwrap();
  p.write(10, &[0x1, 0x2, 0x3, 0x4]);
  p.write(20, &[0x5, 0x6, 0x7, 0x8]);
  assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), p.read(10, 4));
  assert_eq!(vec!(0x5, 0x6, 0x7, 0x8), p.read(20, 4));
}

// FilePage::write() tests
#[test] 
fn write_writes_data_at_beginning() {
  let mut p = FilePage::new(256).unwrap();
  p.write(0, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), p.read(0, 4)); 
}

#[test]
fn write_writes_data_at_offset() {
  let mut p = FilePage::new(256).unwrap();
  p.write(10, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), p.read(10, 4)); 
}

#[test]
fn write_writes_remaining_data_until_end_when_writing_past_max_size() {
  let mut p = FilePage::new(256).unwrap();
  p.write(254, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(vec!(0x1, 0x2), p.read(254, 2)); 
}

#[test]
fn writes_nothing_when_starting_after_max_size() {
  let mut p = FilePage::new(256).unwrap();
  p.write(256, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(vec!(0x0, 0x0), p.read(254, 2)); 
}

#[test]
fn write_increases_actual_size() {
  let mut p = FilePage::new(256).unwrap();
  assert_eq!(0, p.get_actual_size());
  p.write(0, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(4, p.get_actual_size());
  p.write(100, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(104, p.get_actual_size());
}

#[test]
fn write_does_not_increase_actual_size_past_max_size() {
  let mut p = FilePage::new(256).unwrap();
  assert_eq!(0, p.get_actual_size());
  p.write(0, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(4, p.get_actual_size());
  p.write(252, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(256, p.get_actual_size());
  p.write(400, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(256, p.get_actual_size());
}

// FilePage::truncate() tests
#[test]
fn truncate_after_actual_size_does_not_truncate() {
  let mut p = FilePage::new(256).unwrap();
  p.write(128, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(132, p.get_actual_size());
  p.truncate(133);
  assert_eq!(132, p.get_actual_size());
  p.truncate(132);
  assert_eq!(132, p.get_actual_size());
}

#[test]
fn truncate_reduces_actual_size() {
  let mut p = FilePage::new(256).unwrap();
  p.write(128, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(132, p.get_actual_size());
  p.truncate(100);
  assert_eq!(100, p.get_actual_size());
}

#[test]
fn truncate_inits_truncated_memory_to_zeros() {
  let mut p = FilePage::new(256).unwrap();
  p.write(128, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(132, p.get_actual_size());
  p.truncate(130);
  assert_eq!(130, p.get_actual_size());
  p.write(200, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(204, p.get_actual_size());
  assert_eq!(vec!(0x1, 0x2, 0x0, 0x0), p.read(128, 4));
  assert_eq!(vec!(0x1, 0x2, 0x3, 0x4), p.read(200, 4));
}


// FilePage::get_max_size() tests
#[test]
fn get_max_size_returns_max_size() {
  let p = FilePage::new(256).unwrap();
  assert_eq!(256, p.get_max_size());
}

#[test]
fn get_max_size_does_not_change_on_writes() {
  let mut p = FilePage::new(256).unwrap();
  assert_eq!(256, p.get_max_size());
  p.write(0, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(256, p.get_max_size());
  p.write(252, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(256, p.get_max_size());
  p.write(400, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(256, p.get_max_size());
}

// FilePage::get_actual_size() tests
#[test]
fn get_actual_size_returns_zero_when_new() {
  let p = FilePage::new(256).unwrap();
  assert_eq!(0, p.get_actual_size());
}

#[test]
fn get_actual_size_returns_actual_size() {
  let mut p = FilePage::new(256).unwrap();
  assert_eq!(0, p.get_actual_size());
  p.write(0, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(4, p.get_actual_size());
  p.write(100, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(104, p.get_actual_size());
}

#[test]
fn actual_size_only_increases() {
  let mut p = FilePage::new(256).unwrap();
  assert_eq!(0, p.get_actual_size());
  p.write(0, &[0x1, 0x2, 0x3, 0x4]);
  assert_eq!(4, p.get_actual_size());
  p.write(0, &[0x3, 0x4]);
  assert_eq!(4, p.get_actual_size());
}
