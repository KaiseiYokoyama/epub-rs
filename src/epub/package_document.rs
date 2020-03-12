use std::collections::HashMap;
use std::io::Read;

use crate::{EPUBError, util::*};

use xml::name::OwnedName;
use xml::reader::XmlEvent;

use failure::Error;
use meta_data::Metadata;
use failure::_core::convert::TryFrom;

#[derive(Debug)]
pub struct PackageDocument {
    attributes: HashMap<OwnedName, String>,
    pub unique_identifier: String,
    pub version: String,
    pub meta_data: Metadata,
}

impl PackageDocument {
    /// sourceからXMLをパースしてPackage Documentの情報を得る
    pub fn new<R: Read>(source: R) -> Result<Self, Error> {
        let parser = xml::EventReader::new(source);
        let xml = Xml::new(&mut parser.into_iter().peekable());

        let (package_element, attributes) = xml
            .get_by(&Box::new(|e: &XmlElement| match &e.event {
                XmlEvent::StartElement {
                    name,
                    attributes, ..
                } => &name.local_name == "package",
                _ => false
            }))
            .ok_or(EPUBError::PackageDocumentError {
                err_msg: "Package element not found.".to_string()
            })
            .map(|e| match &e.event {
                XmlEvent::StartElement {
                    attributes, ..
                } => (e, attributes),
                _ => unreachable!()
            })?;
        // let (package_element, attributes) = xml.iter()
        //     .find_map(|e| match &e.event {
        //         XmlEvent::StartElement {
        //             name,
        //             attributes, ..
        //         } => if &name.local_name == "package" {
        //             Some((e, attributes))
        //         } else {
        //             None
        //         }
        //         _ => None
        //     })
        //     .ok_or(EPUBError::PackageDocumentError {
        //         err_msg: "Package element not found.".to_string()
        //     })?;

        let attributes = attributes.into_iter()
            .map(|atr| (atr.name.clone(), atr.value.clone()))
            .collect::<HashMap<OwnedName, String>>();

        let unique_identifier = attributes.get(&OwnedName {
            local_name: "unique-identifier".to_string(),
            namespace: None,
            prefix: None,
        })
            .ok_or(EPUBError::PackageDocumentError {
                err_msg: "unique-identifier attribute is undefined.".to_string(),
            })?
            .to_string();

        let version = attributes.get(&OwnedName {
            local_name: "version".to_string(),
            namespace: None,
            prefix: None,
        })
            .ok_or(EPUBError::PackageDocumentError {
                err_msg: "version attribute is undefined.".to_string(),
            })?
            .to_string();

        let meta_data = Metadata::new(package_element, &unique_identifier)?;

        Ok(
            Self {
                attributes,
                unique_identifier,
                version,
                meta_data,
            }
        )
    }

    pub fn dir(&self) -> Option<Dir> {
        self.attributes.get(&OwnedName::local("dir"))
            .map(|s| match s.as_str() {
                "ltr" => Some(Dir::ltr),
                "rtl" => Some(Dir::rtl),
                _ => None,
            })
            .flatten()
    }

    pub fn id(&self) -> Option<&String> {
        self.attributes.get(&OwnedName::local("id"))
    }

    pub fn prefix(&self) -> Option<&String> {
        self.attributes.get(&OwnedName::local("prefix"))
    }

    pub fn xml_lang(&self) -> Option<&String> {
        self.attributes.get(&OwnedName {
            namespace: None,
            prefix: Some("xml".to_string()),
            local_name: "lang".to_string(),
        })
    }
}

#[allow(non_camel_case_types)]
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Dir {
    ltr,
    rtl,
}

impl TryFrom<&str> for Dir {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ltr" => Ok(Dir::ltr),
            "rtl" => Ok(Dir::rtl),
            _ => Err(())
        }
    }
}

pub mod meta_data {
    use super::*;
    use failure::_core::convert::{TryFrom, TryInto};

    /// EPUBのパッケージドキュメントに記載された<metadata>要素, およびそのコンテンツ
    #[derive(Debug)]
    pub struct Metadata {
        /// UUID や DOI、ISBN など、特定のレンディションに関連付けられている識別子
        identifier: Vec<Identifier>,
        unique_identifier: Identifier,
        attributes: HashMap<OwnedName, String>,
    }

    impl Metadata {
        pub fn new(package_element: &XmlElement, unique_identifier: &str) -> Result<Self, Error> {
            let (meta_data_elem, attributes) = package_element.children.iter()
                .find_map(|e| match &e.event {
                    XmlEvent::StartElement {
                        name,
                        attributes, ..
                    } => if &name.local_name == "metadata" {
                        Some((e, attributes))
                    } else {
                        None
                    }
                    _ => None
                })
                .ok_or(EPUBError::PackageDocumentError {
                    err_msg: "Metadata element not found.".to_string()
                })?;

            let attributes = attributes.into_iter()
                .map(|atr| (atr.name.clone(), atr.value.clone()))
                .collect::<HashMap<OwnedName, String>>();

            let identifier = meta_data_elem.children.iter()
                .flat_map(|e| Identifier::try_from(e))
                .collect::<Vec<Identifier>>();

            let unique_identifier = identifier.iter()
                .find_map(|id| if &id.id == &Some(unique_identifier.to_string()) {
                    Some(id.clone())
                } else { None })
                .ok_or(EPUBError::PackageDocumentError {
                    err_msg: "Unique identifier element not found.".to_string()
                })?;


            Ok(Self {
                unique_identifier,
                identifier,
                attributes,
            })
        }

        pub fn unique_identifier(&self) -> &Identifier {
            &self.unique_identifier
        }

        pub fn identifiers(&self) -> &Vec<Identifier> {
            &self.identifier
        }
    }


    /// The identifier element contains an identifier associated with the given Rendition,
    /// such as a UUID, DOI or ISBN.
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct Identifier {
        pub id: Option<String>,
        pub identifier: String,
    }

    impl TryFrom<&XmlElement> for Identifier {
        type Error = ();

        fn try_from(value: &XmlElement) -> Result<Self, Self::Error> {
            match &value.event {
                XmlEvent::StartElement {
                    name,
                    attributes, ..
                } => {
                    if &name.prefix == &Some("dc".to_string()) && &name.local_name == "identifier" {
                        let identifier = value.inner_text();
                        let id = attributes.iter()
                            .find_map(|a| {
                                if &a.name.local_name == "id" {
                                    Some(a.value.to_string())
                                } else { None }
                            });

                        Ok(Identifier { id, identifier })
                    } else { Err(()) }
                }
                _ => Err(())
            }
        }
    }
