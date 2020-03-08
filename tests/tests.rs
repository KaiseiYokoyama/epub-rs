use epub_rs::*;

#[test]
fn new_epub_reader() {
    let path = "tests/data/childrens-literature.epub";
    let reader = EPUBReader::new(path);

    debug_assert!(reader.is_ok(), "{:?}", reader);
}