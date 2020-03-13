use std::collections::HashMap;
use std::io::Read;

use crate::{EPUBError, util::*};

use xml::name::OwnedName;
use xml::attribute::OwnedAttribute;
use xml::reader::XmlEvent;

use failure::Error;
use meta_data::Metadata;
use manifest::Manifest;
use failure::_core::convert::{TryFrom, TryInto};

#[derive(Debug)]
pub struct PackageDocument {
    attributes: HashMap<OwnedName, String>,
    pub unique_identifier: String,
    pub version: String,
    pub meta_data: Metadata,
    pub manifest: Manifest,
}

impl PackageDocument {
    /// sourceからXMLをパースしてPackage Documentの情報を得る
    pub fn new<R: Read>(source: R) -> Result<Self, Error> {
        let parser = xml::EventReader::new(source);
        let xml = Xml::new(&mut parser.into_iter().peekable());

        let (package_element, attributes) = xml
            .get_by(&Box::new(|e: &XmlElement| match &e.event {
                XmlEvent::StartElement {
                    name, ..
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

        let manifest = Manifest::new(package_element)?;

        Ok(
            Self {
                attributes,
                unique_identifier,
                version,
                meta_data,
                manifest,
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
#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
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

trait Element {
    fn name() -> OwnedName;
    fn from_xml_element<T, F>(value: &XmlElement, f: F) -> Option<T>
        where F: FnOnce(&XmlElement, &Vec<OwnedAttribute>) -> T
    {
        match &value.event {
            XmlEvent::StartElement {
                name,
                attributes, ..
            } if name == &Self::name() => {
                Some(f(value, attributes))
            }
            _ => None,
        }
    }
    fn id(attrs: &Vec<OwnedAttribute>) -> Option<String> {
        Self::get_attr(attrs, "id")
        // attrs.iter()
        //     .find_map(|a| {
        //         if &a.name.local_name == "id" {
        //             Some(a.value.to_string())
        //         } else { None }
        //     })
    }
    fn dir(attrs: &Vec<OwnedAttribute>) -> Option<Dir> {
        Self::get_attr(attrs, "dir")
            .map(|a| a.as_str().try_into().ok())
            .flatten()
        // attrs.iter()
        //     .find_map(|a|
        //         if &a.name.local_name == "dir" {
        //             a.value.as_str().try_into().ok()
        //         } else { None }
        //     )
    }
    fn xml_lang(attrs: &Vec<OwnedAttribute>) -> Option<String> {
        attrs.iter()
            .find_map(|a|
                if &a.name.local_name == "lang" && &a.name.prefix == &Some("xml".to_string()) {
                    Some(a.value.to_string())
                } else { None }
            )
    }
    fn get_attr(attrs: &Vec<OwnedAttribute>, key: &str) -> Option<String> {
        attrs.iter()
            .find_map(|a| {
                if &a.name.local_name == key {
                    Some(a.value.to_string())
                } else { None }
            })
    }
}

pub mod meta_data {
    use super::*;
    use failure::_core::convert::TryFrom;

    use failure::_core::str::FromStr;

    ///! todo meta, link要素への対応

    /// EPUBのパッケージドキュメントに記載された<metadata>要素, およびそのコンテンツ
    #[derive(Debug)]
    pub struct Metadata {
        /// UUID や DOI、ISBN など、特定のレンディションに関連付けられている識別子
        identifier: Vec<Identifier>,
        unique_identifier: Identifier,
        /// EPUB 出版物に指定した名前のインスタンス
        titles: Vec<Title>,
        /// 指定されたレンディションのコンテンツの言語
        languages: Vec<Language>,
        attributes: HashMap<OwnedName, String>,
        optionals: Vec<OptionalElement>,
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

            let languages = meta_data_elem.children.iter()
                .flat_map(|e| Language::try_from(e))
                .collect::<Vec<Language>>();
            // language要素の有無を確認する
            languages.get(0)
                .ok_or(EPUBError::PackageDocumentError {
                    err_msg: "Language not found.".to_string()
                })?;

            let optionals = meta_data_elem.children.iter()
                .flat_map(|e| OptionalElement::try_from(e))
                .collect();

            Ok(Self {
                unique_identifier,
                identifier,
                attributes,
                titles,
                languages,
                optionals,
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
            &self.titles
        }

        pub fn language(&self) -> Option<&Language> { self.languages.get(0) }

        pub fn languages(&self) -> &Vec<Language> { &self.languages }
    }

    /// The identifier element contains an identifier associated with the given Rendition,
    /// such as a UUID, DOI or ISBN.
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct Identifier {
        pub id: Option<String>,
        pub identifier: String,
    }

    impl Element for Identifier {
        fn name() -> OwnedName {
            OwnedName {
                prefix: Some(String::from("dc")),
                local_name: String::from("identifier"),
                namespace: Some(String::from("http://purl.org/dc/elements/1.1/")),
            }
        }
    }

    impl TryFrom<&XmlElement> for Identifier {
        type Error = ();

        fn try_from(value: &XmlElement) -> Result<Self, Self::Error> {
            Self::from_xml_element(value, |elem, attrs| {
                let identifier = elem.inner_text();
                let id = Self::id(attrs);

                Identifier { id, identifier }
            })
                .ok_or(())
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

    impl Element for Title {
        fn name() -> OwnedName {
            OwnedName {
                prefix: Some(String::from("dc")),
                local_name: String::from("title"),
                namespace: Some(String::from("http://purl.org/dc/elements/1.1/")),
            }
        }
    }

    impl TryFrom<&XmlElement> for Title {
        type Error = ();

        fn try_from(value: &XmlElement) -> Result<Self, Self::Error> {
            Self::from_xml_element(value, |elem, attrs| {
                let title = elem.inner_text();
                let dir = Self::dir(attrs);
                let id = Self::id(attrs);
                let xml_lang = Self::xml_lang(attrs);

                Title { title, dir, id, xml_lang }
            })
                .ok_or(())
        }
    }

    /// The language element specifies the language of the content of the given Rendition.
    /// This value is not inherited by the individual resources of the Rendition.
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct Language {
        language: String,
        id: Option<String>,
    }

    impl Element for Language {
        fn name() -> OwnedName {
            OwnedName {
                prefix: Some(String::from("dc")),
                local_name: String::from("language"),
                namespace: Some(String::from("http://purl.org/dc/elements/1.1/")),
            }
        }
    }

    impl TryFrom<&XmlElement> for Language {
        type Error = ();

        fn try_from(value: &XmlElement) -> Result<Self, Self::Error> {
            Self::from_xml_element(value, |elem, attrs| {
                let language = elem.inner_text();
                let id = Self::id(attrs);

                Language { language, id }
            })
                .ok_or(())
        }
    }

    #[allow(non_camel_case_types)]
    #[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
    pub enum OptionalElementName {
        contributor,
        coverage,
        creator,
        date,
        description,
        format,
        publisher,
        relation,
        rights,
        source,
        subject,
        type_,
    }

    impl FromStr for OptionalElementName {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "contributor" => Ok(OptionalElementName::contributor),
                "coverage" => Ok(OptionalElementName::coverage),
                "creator" => Ok(OptionalElementName::creator),
                "date" => Ok(OptionalElementName::date),
                "description" => Ok(OptionalElementName::description),
                "format" => Ok(OptionalElementName::format),
                "publisher" => Ok(OptionalElementName::publisher),
                "relation" => Ok(OptionalElementName::relation),
                "rights" => Ok(OptionalElementName::rights),
                "source" => Ok(OptionalElementName::source),
                "subject" => Ok(OptionalElementName::subject),
                "type" => Ok(OptionalElementName::type_),
                _ => Err(())
            }
        }
    }

    impl ToString for OptionalElementName {
        fn to_string(&self) -> String {
            match &self {
                OptionalElementName::type_ => "type".to_string(),
                _ => format!("{:?}", &self)
            }
        }
    }

    /// Attributes of contributor, coverage, creator, description,
    /// publisher, relation, rights, and subject.
    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    pub struct OptionalElement {
        name: OptionalElementName,
        value: String,
        dir: Option<Dir>,
        id: Option<String>,
        xml_lang: Option<String>,
    }

    impl TryFrom<&XmlElement> for OptionalElement {
        type Error = ();

        fn try_from(value: &XmlElement) -> Result<Self, Self::Error> {
            match &value.event {
                XmlEvent::StartElement {
                    name,
                    attributes, ..
                } if name.prefix == Some(String::from("dc"))
                    && name.namespace == Some(String::from("http://purl.org/dc/elements/1.1/"))
                => if let Ok(name) = OptionalElementName::from_str(&name.local_name) {
                    let value = value.inner_text();
                    // let dir = Element::dir(attributes);
                    // let id =  Element::id(attributes);
                    // let xml_lang = Element::xml_lang(attributes);
                    let dir = Identifier::dir(attributes);
                    let id = Identifier::id(attributes);
                    let xml_lang = Identifier::xml_lang(attributes);

                    Ok(OptionalElement {
                        name,
                        value,
                        dir,
                        id,
                        xml_lang,
                    })
                } else {
                    Err(())
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
        use std::io::BufReader;
        use std::fs::File;

        const PATH: &'static str = "tests/data/childrens-literature.epub";

        fn reader() -> Result<EPUBReader<BufReader<File>>, Error> {
            EPUBReader::new(PATH)
        }

        #[test]
        fn unique_identifier() -> Result<(), Error> {
            let reader = reader()?;

            if let Some(pd) = reader.package_document() {
                let unique_id = Identifier {
                    id: Some("id".to_string()),
                    identifier: "http://www.gutenberg.org/ebooks/25545".to_string(),
                };
                assert_eq!(&unique_id, pd.meta_data.unique_identifier());
            } else {
                debug_assert!(false, "No Package Documents.")
            }

            Ok(())
        }

        #[test]
        fn title() -> Result<(), Error> {
            let reader = reader()?;

            if let Some(pd) = reader.package_document() {
                let title = Title {
                    title: "Children\'s Literature".into(),
                    dir: None,
                    id: Some("t1".into()),
                    xml_lang: None,
                };
                assert_eq!(Some(&title), pd.meta_data.title());
            } else {
                debug_assert!(false, "No Package Documents");
            }

            Ok(())
        }

        #[test]
        fn language() -> Result<(), Error> {
            let reader = reader()?;

            if let Some(pd) = reader.package_document() {
                let language = Language {
                    language: "en".into(),
                    id: None,
                };
                assert_eq!(Some(&language), pd.meta_data.language());
            } else {
                debug_assert!(false, "No Package Documents");
            }

            Ok(())
        }

        #[test]
        fn optionals() -> Result<(), Error> {
            let reader = reader()?;

            if let Some(pd) = reader.package_document() {
                use OptionalElementName::*;
                let optionals = &pd.meta_data.optionals.iter()
                    .collect::<std::collections::HashSet<&OptionalElement>>();
                let correct =
                    vec![OptionalElement {
                        name: creator,
                        value: "Charles Madison Curry".into(),
                        dir: None,
                        id: Some("curry".into()),
                        xml_lang: None,
                    },
                         OptionalElement {
                             name: creator,
                             value: "Erle Elsworth Clippinger".into(),
                             dir: None,
                             id: Some("clippinger".into()),
                             xml_lang: None,
                         },
                         OptionalElement {
                             name: date,
                             value: "2008-05-20".into(),
                             dir: None,
                             id: None,
                             xml_lang: None,
                         },
                         OptionalElement {
                             name: subject,
                             value: "Children -- Books and reading".into(),
                             dir: None,
                             id: None,
                             xml_lang: None,
                         },
                         OptionalElement {
                             name: subject,
                             value: "Children\'s literature -- Study and teaching".into(),
                             dir: None,
                             id: None,
                             xml_lang: None,
                         },
                         OptionalElement {
                             name: source,
                             value: "http://www.gutenberg.org/files/25545/25545-h/25545-h.htm".into(),
                             dir: None,
                             id: None,
                             xml_lang: None,
                         },
                         OptionalElement {
                             name: rights,
                             value: "Public domain in the USA.".into(),
                             dir: None,
                             id: None,
                             xml_lang: None,
                         }];
                let correct_set = &correct.iter()
                    .collect::<std::collections::HashSet<&OptionalElement>>();
                assert_eq!(correct_set, optionals);
            } else {
                debug_assert!(false, "No Package Documents");
            }

            Ok(())
        }
    }
}

pub mod manifest {
    use super::*;
    use std::collections::HashSet;
    use crate::media_type::MediaType;
    use failure::_core::str::FromStr;

    ///! レンディションを構成する出版物リソースの完全なリスト
    #[derive(Debug, Eq, PartialEq)]
    pub struct Manifest {
        id: Option<String>,
        items: HashSet<Item>,
        cover_image: Option<Item>,
        nav: Item,
    }

    impl Manifest {
        pub fn new(package_element: &XmlElement) -> Result<Self, Error> {
            let (manifest_elem, attributes) = package_element.children.iter()
                .find_map(|e| match &e.event {
                    XmlEvent::StartElement {
                        name,
                        attributes, ..
                    } => if &name.local_name == "manifest" {
                        Some((e, attributes))
                    } else {
                        None
                    }
                    _ => None
                })
                .ok_or(EPUBError::PackageDocumentError {
                    err_msg: "Manifest element not found.".to_string()
                })?;

            let id = attributes.iter()
                .find_map(|a| if &a.name.local_name == "id" {
                    Some(a.value.to_string())
                } else { None });

            let items: HashSet<Item> = manifest_elem.children.iter()
                .flat_map(|e| Item::try_from(e))
                .collect();

            let cover_image = items.iter()
                .find(|i| i.properties.contains(&Property::cover_image))
                .cloned();

            let nav = items.iter()
                .find(|i| i.properties.contains(&Property::nav))
                .ok_or(EPUBError::PackageDocumentError {
                    err_msg: "Navigation item not found in <manifest>".to_string()
                })?
                .clone();

            Ok(Self { id, items, cover_image, nav })
        }
    }

    #[derive(Clone, Eq, PartialEq, Hash, Debug)]
    pub struct Item {
        fallback: Option<String>,
        href: String,
        id: String,
        media_overlay: Option<String>,
        media_type: MediaType,
        properties: Vec<Property>,
    }

    impl Element for Item {
        fn name() -> OwnedName {
            OwnedName {
                prefix: None,
                local_name: String::from("item"),
                namespace: Some(String::from("http://www.idpf.org/2007/opf")),
            }
        }
    }

    impl TryFrom<&XmlElement> for Item {
        type Error = ();
        fn try_from(value: &XmlElement) -> Result<Self, Self::Error> {
            Item::from_xml_element(value, |_elem, attrs| {
                let fallback = Item::get_attr(attrs, "fallback");
                let id = Item::id(attrs)?;
                // .ok_or(EPUBError::PackageDocumentError {
                //     err_msg: "ID is undefined on <item>".to_string()
                // })?;
                let href = Item::get_attr(attrs, "href")?;
                // .ok_or(EPUBError::PackageDocumentError {
                //     err_msg: "Href is undefined on <item>".to_string()
                // })?;
                let media_overlay = Item::get_attr(attrs, "media-overlay");
                let media_type = Item::get_attr(attrs, "media-type")
                    .map(|s| MediaType::from_str(&s).ok())
                    .flatten()?;
                // .ok_or(EPUBError::PackageDocumentError {
                //     err_msg: "Media-type is undefined on <item>".to_string()
                // })?;
                let properties = Item::get_attr(attrs, "properties")
                    .iter()
                    .flat_map(|s| s.split_whitespace())
                    .flat_map(|s| Property::from_str(s))
                    .collect::<Vec<Property>>();

                Some(
                    Item {
                        fallback,
                        href,
                        id,
                        media_overlay,
                        media_type,
                        properties,
                    }
                )
            })
                .flatten()
                .ok_or(())
        }
    }

    #[allow(non_camel_case_types)]
    #[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
    pub enum Property {
        cover_image,
        mathml,
        nav,
        remote_resources,
        scripted,
        svg,
        switch,
    }

    impl FromStr for Property {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            use Property::*;
            match s {
                "cover-image" => Ok(cover_image),
                "mathml" => Ok(mathml),
                "nav" => Ok(nav),
                "remote_resources" => Ok(remote_resources),
                "scripted" => Ok(scripted),
                "svg" => Ok(svg),
                "switch" => Ok(switch),
                _ => Err(())
            }
        }
    }

    impl ToString for Property {
        fn to_string(&self) -> String {
            use Property::*;
            match &self {
                cover_image => "cover-image".to_string(),
                remote_resources => "remote-resources".to_string(),
                _ => format!("{:?}", &self)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::read::EPUBReader;
        use failure::Error;
        use std::io::BufReader;
        use std::fs::File;

        const PATH: &'static str = "tests/data/childrens-literature.epub";

        fn reader() -> Result<EPUBReader<BufReader<File>>, Error> {
            EPUBReader::new(PATH)
        }

        #[test]
        fn manifest() -> Result<(), Error> {
            use super::Property::*;
            use crate::media_type::{MediaType::*,
                                    ImageType::*,
                                    ApplicationType::*,
                                    TextType::*};

            let reader = reader()?;

            if let Some(pd) = reader.package_document() {
                let manifest = &pd.manifest;
                let correct = Manifest {
                    id: None,
                    items: vec![
                        Item {
                            fallback: None,
                            href: "cover.xhtml".into(),
                            id: "cover".into(),
                            media_overlay: None,
                            media_type: Application(XhtmlXml),
                            properties: vec![],
                        },
                        Item {
                            fallback: None,
                            href: "nav.xhtml".into(),
                            id: "nav".into(),
                            media_overlay: None,
                            media_type: Application(XhtmlXml),
                            properties: vec![nav, scripted],
                        },
                        Item {
                            fallback: None,
                            href: "images/cover.png".into(),
                            id: "cover-img".into(),
                            media_overlay: None,
                            media_type: Image(PNG),
                            properties: vec![cover_image],
                        },
                        Item {
                            fallback: None,
                            href: "css/epub.css".into(),
                            id: "css01".into(),
                            media_overlay: None,
                            media_type: Text(CSS),
                            properties: vec![],
                        },
                        Item {
                            fallback: None,
                            href: "css/nav.css".into(),
                            id: "css02".into(),
                            media_overlay: None,
                            media_type: Text(CSS),
                            properties: vec![],
                        },
                        Item {
                            fallback: None,
                            href: "s04.xhtml".into(),
                            id: "s04".into(),
                            media_overlay: None,
                            media_type: Application(XhtmlXml),
                            properties: vec![],
                        }
                    ].into_iter().collect(),
                    cover_image: Some(Item {
                        fallback: None,
                        href: "images/cover.png".into(),
                        id: "cover-img".into(),
                        media_overlay: None,
                        media_type: Image(PNG),
                        properties: vec![cover_image],
                    }),
                    nav: Item {
                        fallback: None,
                        href: "nav.xhtml".into(),
                        id: "nav".into(),
                        media_overlay: None,
                        media_type: Application(XhtmlXml),
                        properties: vec![nav, scripted],
                    },
                };
                assert_eq!(&correct, manifest);
            } else {
                debug_assert!(false, "No Package Documents.")
            }

            Ok(())
        }
    }
}