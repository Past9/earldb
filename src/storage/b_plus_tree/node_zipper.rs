use std::collections::VecDeque;

use bytesorder::{ LittleEndian, ReadBytesExt, WriteBytesExt };

use error::{ Error, AssertionError };
use storage::binary_storage::BinaryStorage;

