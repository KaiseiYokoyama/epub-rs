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

pub mod util {
    use std::io::Read;
    use std::iter::Peekable;
    use xml::reader::*;
    use failure::_core::ops::Deref;

    #[derive(Debug, Clone)]
    pub struct Xml {
        pub vec: Vec<XmlElement>
    }

    impl Deref for Xml {
        type Target = Vec<XmlElement>;

        fn deref(&self) -> &Self::Target {
            &self.vec
        }
    }

    impl Xml {
        pub fn new<R: Read>(iter: &mut Peekable<Events<R>>) -> Self {
            let mut vec = Vec::new();

            while let Some(_) = iter.peek() {
                if let Some(elem) = XmlElement::new(iter) {
                    vec.push(elem);
                }
            }

            Self {
                vec
            }
        }

        pub fn get_by<F: FnMut(&XmlElement) -> bool>(&self, mut f: F) -> Option<&XmlElement> {
            self.vec.iter()
                .find_map(|e| e.get_by(&mut f))
        }
    }

    #[derive(Debug, Clone)]
    pub struct XmlElement {
        pub event: XmlEvent,
        pub children: Vec<XmlElement>,
    }

    impl XmlElement {
        pub fn new<R: Read>(iter: &mut Peekable<Events<R>>) -> Option<Self> {
            let mut children = Vec::new();

            match iter.peek()?.as_ref().ok()? {
                XmlEvent::CData(_)
                | XmlEvent::Characters(_)
                | XmlEvent::Comment(_)
                | XmlEvent::Whitespace(_)
                | XmlEvent::ProcessingInstruction { .. } => {
                    let event = iter.next()?.ok()?;

                    Some(Self {
                        event,
                        children,
                    })
                }
                XmlEvent::StartElement { name, .. } => {
                    let start_name = name.clone();
                    let event = iter.next()?.ok()?;

                    while let Some(child) = Self::new(iter) {
                        children.push(child);

                        match iter.peek()?.as_ref().ok()? {
                            XmlEvent::EndElement { name } if &start_name == name => {
                                let _ = iter.next();
                                return Some(Self {
                                    event,
                                    children,
                                });
                            }
                            _ => {}
                        }
                    }

                    Some(Self {
                        event,
                        children,
                    })
                }
                XmlEvent::StartDocument { .. } => {
                    let event = iter.next()?.ok()?;

                    while let Some(child) = Self::new(iter) {
                        children.push(child);

                        match iter.peek()?.as_ref().ok()? {
                            XmlEvent::EndDocument => {
                                let _ = iter.next();
                                return Some(Self {
                                    event,
                                    children,
                                });
                            }
                            _ => {}
                        }
                    }

                    Some(Self {
                        event,
                        children,
                    })
                }
                _ => {
                    let _ = iter.next();
                    None
                }
            }
        }

        pub fn inner_text(&self) -> String {
            self.children.iter()
                .filter_map(|e| {
                    match &e.event {
                        XmlEvent::CData(cdata) => Some(cdata.to_string()),
                        XmlEvent::Characters(characters) => Some(characters.to_string()),
                        XmlEvent::StartDocument { .. } | XmlEvent::StartElement { .. } => {
                            Some(e.inner_text())
                        }
                        XmlEvent::EndDocument
                        | XmlEvent::EndElement { .. }
                        | XmlEvent::Whitespace(..)
                        | XmlEvent::ProcessingInstruction { .. }
                        | XmlEvent::Comment(..) => {
                            None
                        }
                    }
                })
                .fold(String::new(), |acc, s| acc + &s)
        }

        pub fn get_by<F: FnMut(&XmlElement) -> bool>(&self, f: &mut F) -> Option<&XmlElement> {
            if f(self) {
                Some(&self)
            } else {
                self.children.iter()
                    .find(|&e| f(e))
            }
        }
    }
}