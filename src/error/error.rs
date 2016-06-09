
use std::error;
use std::fmt;
use std::io;


pub struct MemoryError {
    desc: String
}
impl MemoryError {
    pub fn new(desc: &str) -> MemoryError {
        MemoryError {
            desc: desc.to_string()
        }
    }
}
impl error::Error for MemoryError {
    fn description(&self) -> &str {
        self.desc.as_str()
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}
impl fmt::Debug for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}


pub enum Error {
    Io(io::Error),
    Memory(MemoryError)
}
impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::Memory(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::Memory(ref err) => Some(err),
        }
    }
}
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}
impl From<MemoryError> for Error {
    fn from(err: MemoryError) -> Error {
        Error::Memory(err)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IoError: {}", err),
            Error::Memory(ref err) => write!(f, "MemoryError: {}", err)
        }
    }
}
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
