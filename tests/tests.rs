use epub_rs::*;

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