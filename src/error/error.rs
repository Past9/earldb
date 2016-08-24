
use std::str;
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


pub struct AssertionError {
    desc: String
}
impl AssertionError {
    pub fn new(desc: &str) -> AssertionError {
        AssertionError {
            desc: desc.to_string()
        }
    }

    pub fn assert(condition: bool, msg: &str) -> Result<(), AssertionError> {
        if !condition { return Err(AssertionError::new(msg)) }
        Ok(())
    }

    pub fn assert_not(condition: bool, msg: &str) -> Result<(), AssertionError> {
        AssertionError::assert(!condition, msg)
    }
}
impl error::Error for AssertionError {
    fn description(&self) -> &str {
        self.desc.as_str()
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
impl fmt::Display for AssertionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}
impl fmt::Debug for AssertionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}



pub enum Error {
    Io(io::Error),
    Utf8(str::Utf8Error),
    Memory(MemoryError),
    Assertion(AssertionError)
}
impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::Utf8(ref err) => err.description(),
            Error::Memory(ref err) => err.description(),
            Error::Assertion(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::Utf8(ref err) => Some(err),
            Error::Memory(ref err) => Some(err),
            Error::Assertion(ref err) => Some(err)
        }
    }
}
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}
impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Error {
        Error::Utf8(err)
    }
}
impl From<MemoryError> for Error {
    fn from(err: MemoryError) -> Error {
        Error::Memory(err)
    }
}
impl From<AssertionError> for Error {
    fn from(err: AssertionError) -> Error {
        Error::Assertion(err)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IoError: {}", err),
            Error::Utf8(ref err) => write!(f, "Utf8Error: {}", err),
            Error::Memory(ref err) => write!(f, "MemoryError: {}", err),
            Error::Assertion(ref err) => write!(f, "AssertionError: {}", err)
        }
    }
}
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
