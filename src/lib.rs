pub mod read;
pub mod epub;
pub mod media_type;

pub mod prelude {
    pub use crate::EPUBError;
    pub use crate::read::EPUBReader;
    pub use crate::epub::*;
    pub use crate::media_type::*;
}

use failure::Fail;

#[derive(Fail, Debug)]
pub enum EPUBError {
    #[fail(display = "PACKAGE DOCUMENT :ERROR\t: {}", err_msg)]
    PackageDocumentError {
        err_msg: String,
    },
    #[fail(display = "MEDIA TYPE :ERROR\t: {}", err_msg)]
    MediaTypeError {
        err_msg: String,
    },
    #[fail(display = "CONTAINER :ERROR\t: {}", err_msg)]
    ContainerError {
        err_msg: String,
    },
    #[fail(display = "MIMETYPE ERROR\t: {}", err_msg)]
    MimeTypeError {
        err_msg: String,
    },
    #[fail(display = "ZIP ERROR\t: {:?}", error)]
    ZipError {
        error: zip::result::ZipError
    },
    #[fail(display = "XML ERROR\t: {:?}", error)]
    XMLError {
        error: xml::reader::Error
    },
}