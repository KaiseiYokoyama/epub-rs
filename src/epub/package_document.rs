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
        /// EPUB 出版物に指定した名前のインスタンス
        titles: Vec<Title>,
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

            let titles = meta_data_elem.children.iter()
                .flat_map(|e| Title::try_from(e))
                .collect::<Vec<Title>>();
            // title要素の有無を確認する
            titles.get(0)
                .ok_or(EPUBError::PackageDocumentError {
                    err_msg: "Title not found.".to_string()
                })?;

            Ok(Self {
                unique_identifier,
                identifier,
                attributes,
                titles,
            })
        }

        pub fn unique_identifier(&self) -> &Identifier {
            &self.unique_identifier
        }

        pub fn identifiers(&self) -> &Vec<Identifier> {
            &self.identifier
        }

        pub fn title(&self) -> Option<&Title> {
            self.titles.get(0)
        }

        pub fn titles(&self) -> &Vec<Title> {
            &self.title
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

    /// The title element represents an instance of a name given to the EPUB Publication.
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct Title {
        pub title: String,
        pub dir: Option<Dir>,
        pub id: Option<String>,
        pub xml_lang: Option<String>,
    }

    impl TryFrom<&XmlElement> for Title {
        type Error = ();

        fn try_from(value: &XmlElement) -> Result<Self, Self::Error> {
            // println!("{}, {:?}", line!(), &value);
            match &value.event {
                XmlEvent::StartElement {
                    name,
                    attributes, ..
                } => {
                    if &name.prefix == &Some("dc".to_string()) && &name.local_name == "title" {
                        let title = value.inner_text();

                        let dir = attributes.iter()
                            .find_map(|a|
                                if &a.name.local_name == "dir" {
                                    a.value.as_str().try_into().ok()
                                } else { None }
                            );

                        let id = attributes.iter()
                            .find_map(|a| {
                                if &a.name.local_name == "id" {
                                    Some(a.value.to_string())
                                } else { None }
                            });

                        let xml_lang = attributes.iter()
                            .find_map(|a|
                                if &a.name.local_name == "lang" && &a.name.prefix == &Some("xml".to_string()) {
                                    Some(a.value.to_string())
                                } else { None }
                            );

                        Ok(Title {
                            title,
                            dir,
                            id,
                            xml_lang,
                        })
                    } else { Err(()) }
                }
                _ => Err(())
            }
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::read::EPUBReader;
        use failure::Error;

        const PATH: &'static str = "tests/data/childrens-literature.epub";

        #[test]
        fn unique_identifier() -> Result<(), Error> {
            let reader = EPUBReader::new(PATH)?;

            if let Some(pd) = reader.package_document() {
                let unique_id = Identifier {
                    id: Some("id".to_string()),
                    identifier: "http://www.gutenberg.org/ebooks/25545".to_string()
                };
                assert_eq!(&unique_id, pd.meta_data.unique_identifier());
            } else {
                debug_assert!(false, "No Package Documents.")
            }

            Ok(())
        }

        #[test]
        fn title() -> Result<(), Error> {
            let reader = EPUBReader::new(PATH)?;

            if let Some(pd) = reader.package_document() {
                let title = Title {
                    title: "Children\'s Literature".into(),
                    dir: None,
                    id: Some("t1".into()),
                    xml_lang: None,
                };
                assert_eq!(Some(&title),pd.meta_data.title());
            } else {
                debug_assert!(false, "No Package Documents");
            }

            Ok(())
        }
    }
}