use epub_rs::*;

fn main() -> Result<(), failure::Error>{
    use std::io::{Read, BufReader};
    use zip::ZipArchive;

    let path = "tests/data/childrens-literature.epub";
    // let file = std::fs::File::open(path)?;
    // let buf_reader = BufReader::new(file);
    // let mut archive = ZipArchive::new(buf_reader)?;
    //
    // archive.file_names()
    //     .inspect(|n| println!("{}", n))
    //     .last();

    let mut epub = EPUBReader::new(path)?;
    // println!("{:?}", epub.package_document_paths());
    Ok(())
}