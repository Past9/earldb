
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

/*
  t.insert(&[0x02], &[0x88]).unwrap();
  t.insert(&[0x03], &[0x89]).unwrap();
  */
  t.insert(&[0x04], &[0x8a]).unwrap();
  t.insert(&[0x05], &[0x8b]).unwrap();
  t.insert(&[0x06], &[0x8c]).unwrap();
  //t.insert(&[0x05], &[0x8b]).unwrap();

  //assert_eq!(true, false);

/*
  assert_eq!(vec!(0xff), t.search(&[0x01]).unwrap().unwrap());
  assert_eq!(vec!(0x88), t.search(&[0x02]).unwrap().unwrap());
  assert_eq!(vec!(0x89), t.search(&[0x03]).unwrap().unwrap());
  */
  //assert_eq!(vec!(0x8a), t.search(&[0x04]).unwrap().unwrap());
  //assert_eq!(vec!(0x8b), t.search(&[0x05]).unwrap().unwrap());

}

