
use xcb::base::Error as XcbError;
use xcb::ffi::base::xcb_generic_error_t as XcbGenericError;

use std::fmt;

#[derive(Debug)]
pub enum Error {
    Connection(xcb::ConnError),
    ImageFormat(String),
    ImageCreate,
    ImagePut,
    Flush,
    XcbError(XcbError<XcbGenericError>),
}

impl From<xcb::ConnError> for Error {
    fn from(e: xcb::ConnError) -> Error {
        Error::Connection(e)
    }
}

impl From<XcbError<XcbGenericError>> for Error {
    fn from(e: XcbError<XcbGenericError>) -> Error {
        Error::XcbError(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ImageFormat(e) => write!(f, "Image format error: unsupported format: {}", e),
            Error::Connection(e) => write!(f, "X11 connection error: {}", e),
            Error::XcbError(XcbError{ ptr: e }) => write!(f, "XCB error: {:?}", e),
            Error::ImageCreate => write!(f, "X11 failed to create image"),
            Error::ImagePut => write!(f, "XCB failed to put image"),
            Error::Flush => write!(f, "XCB failed to flush output to server"),
        }
    }
}
