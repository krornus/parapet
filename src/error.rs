use std::fmt;

#[derive(Debug)]
pub enum Error {
    Connection(xcb::ConnError),
    ImageFormat(String),
}

impl From<xcb::ConnError> for Error {
    fn from(e: xcb::ConnError) -> Error {
        Error::Connection(e)
    }
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ImageFormat(e) => write!(f, "Image format error: unsupported format", e),
            Error::Connection(e) => write!(f, "X11 connection error: {}", e),
        }
    }
}
