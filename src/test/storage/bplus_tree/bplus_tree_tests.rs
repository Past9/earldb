
use storage::binary_storage::BinaryStorage;
use storage::memory_binary_storage::MemoryBinaryStorage;
use storage::bplus_tree::bplus_tree::BPlusTree;

#[test]
pub fn inserts_and_finds() {

  let mut s = MemoryBinaryStorage::new(256, 256).unwrap();
  s.open().unwrap();
  s.w_u8(0, 0x02).unwrap();
  s.w_u32(25, 3);
  s.w_u8(29, 0x01); // 1 => 255
  s.w_u8(30, 0xff);
  s.w_u8(31, 0x02); // 2 => 254
  s.w_u8(32, 0xfe);
  s.w_u8(33, 0x03); // 3 => 253
  s.w_u8(34, 0xfd);
  s.close().unwrap();

  let mut t = BPlusTree::new(
    s,
    1,
    1,
    40,
  );


  t.open().unwrap();

  assert_eq!(vec!(0xfe), t.search(&[0x02]).unwrap().unwrap());

}

