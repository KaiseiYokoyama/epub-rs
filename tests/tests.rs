use epub_rs::prelude::*;

const PATH: &'static str = "tests/data/childrens-literature.epub";

#[test]
fn new_epub_reader() {
    let reader = EPUBReader::new(PATH);

    debug_assert!(reader.is_ok(), "{:?}", reader);
}

#[test]
fn package_attributes() -> Result<(), failure::Error>{
    let reader = EPUBReader::new(PATH)?;

    if let Some(pd) = reader.package_document() {
        assert_eq!("id", &pd.unique_identifier);
        assert_eq!("3.0", &pd.version);
    } else {
        debug_assert!(false, "No Package Documents.")
    }

    Ok(())
}

#[test]
fn meta_data() -> Result<(), failure::Error> {
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
fn xml() -> Result<(), failure::Error> {
    let file = std::fs::File::open("tests/data/sample_package.opf")?;
    let mut xml_reader = xml::reader::EventReader::new(file);

    let xml = epub_rs::util::Xml::new(&mut xml_reader.into_iter().peekable());
    debug_assert!(false, "{:?}", xml);

    Ok(())
}