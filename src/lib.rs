mod media_type;

use std::io::{Read, Seek, BufReader};
use std::fs::File;
use std::path::{Path, PathBuf};
use zip::read::{ZipArchive, ZipFile};
use failure::{Fail, Error};
use failure::_core::ops::Deref;

#[derive(Debug)]
pub struct EPUBReader<R: Read + Seek> {
    archive: ZipArchive<R>,
}

impl<R: Read + Seek> EPUBReader<R> {
    const MIMETYPE_PATH: &'static str = "mimetype";
    const MIMETYPE_CONTENT: &'static str = "application/epub+zip";
    const CONTAINER_XML_PATH: &'static str = "META-INF/container.xml";
}

impl EPUBReader<BufReader<File>> {
    pub fn new<P>(path: P) -> Result<Self, Error> where P: AsRef<Path> {
        let archive: ZipArchive<BufReader<File>> = std::fs::File::open(path)
            .map(|file| {
                let buf_reader = std::io::BufReader::new(file);
                ZipArchive::new(buf_reader)
            })??;

        Ok(Self {
            archive
        })
    }

    pub fn rootfiles(&mut self) -> Result<Vec<PathBuf>, Error> {
        use xml::attribute::OwnedAttribute;
        use xml::reader::XmlEvent;

        let archive = &mut self.archive;
        let container_xml = archive.by_name(Self::CONTAINER_XML_PATH)?;

        let file = BufReader::new(container_xml);
        let mut parser = xml::EventReader::new(file);

        let package_documents = parser.into_iter()
            .filter_map(|e| match e.ok()? {
                XmlEvent::StartElement { name, attributes, .. } => {
                    if name.local_name == "rootfile".to_string() {
                        Some(
                            attributes.into_iter()
                                .find_map(|atr| {
                                    let OwnedAttribute { name, value } = atr;
                                    if name.local_name == "full-path".to_string() {
                                        let path_buf = PathBuf::from(value);
                                        Some(path_buf)
                                    } else {
                                        None
                                    }
                                })?
                        )
                    } else {
                        None
                    }
                }
                _ => None
            })
            .collect::<Vec<PathBuf>>();

        Ok(package_documents)
    }

    pub fn rootfile(&mut self) -> Result<Option<PathBuf>, Error> {
        self.rootfiles()
            .map(|ps| ps.into_iter().next())
    }

    pub fn check_mimetype(&mut self) -> Result<(), Error> {
        let archive = &mut self.archive;
        let mut mimetype = archive.by_index(0)?;
        if mimetype.name() != Self::MIMETYPE_PATH {
            Err(EPUBError::MimeTypeError {
                err_msg: "The mimetype not found".to_string()
            })?
        }
        let mut mimetype_content = [0u8; 20];
        mimetype.read(&mut mimetype_content);
        match String::from_utf8(mimetype_content.to_vec()) {
            Ok(content) if content == Self::MIMETYPE_CONTENT.to_string() => {}
            Ok(content) => {
                Err(EPUBError::MimeTypeError {
                    err_msg: format!("The mimetype file's content {} is invalid. It MUST be `application/epub+zip`.", &content)
                })?
            }
            Err(e) => {
                Err(EPUBError::MimeTypeError {
                    err_msg: format!("{}", std::error::Error::description(&e))
                })?
            }
        }

        Ok(())
    }

    pub fn check_container_xml(&mut self) -> Result<(), Error> {
        use xml::name::OwnedName;
        use xml::attribute::OwnedAttribute;
        use xml::reader::XmlEvent;

        let archive = &mut self.archive;
        let container_xml = archive.by_name(Self::CONTAINER_XML_PATH)?;

        let file = BufReader::new(container_xml);
        let mut parser = xml::EventReader::new(file);

        parser.into_iter()
            .filter_map(|e| match e.ok()? {
                XmlEvent::StartElement { name, attributes, .. } => {
                    if name.local_name == "rootfile".to_string() {
                        Some(
                            attributes.into_iter()
                                .find_map(|atr| {
                                    let OwnedAttribute { name, value } = atr;
                                    if name.local_name == "full-path".to_string() {
                                        let path_buf = PathBuf::from(value);
                                        Some(path_buf)
                                    } else {
                                        None
                                    }
                                })
                                .ok_or(
                                    EPUBError::ContainerError {
                                        err_msg: "Full-path of package document is undefined in <rootfile>".to_string()
                                    }.into()
                                )
                        )
                    } else {
                        None
                    }
                }
                _ => None
            })
            .collect::<Result<Vec<PathBuf>, Error>>()?;

        Ok(())
    }
}

impl<R: Read + Seek> Deref for EPUBReader<R> {
    type Target = ZipArchive<R>;

    fn deref(&self) -> &Self::Target {
        &self.archive
    }
}

#[derive(Fail, Debug)]
pub enum EPUBError {
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
