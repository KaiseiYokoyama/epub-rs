use epub_rs::prelude::*;
use std::io::BufReader;
use std::fs::File;

const PATH: &'static str = "tests/data/childrens-literature.epub";

fn reader() -> Result<EPUBReader<BufReader<File>>, failure::Error> {
    EPUBReader::new(PATH)
}

#[test]
fn new_epub_reader() {
    let reader = reader();

    debug_assert!(reader.is_ok(), "{:?}", reader);
}

#[test]
fn package_attributes() -> Result<(), failure::Error>{
    let reader = reader()?;

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