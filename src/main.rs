use epub_rs::prelude::*;

fn main() -> Result<(), failure::Error>{
    let path = "tests/data/childrens-literature.epub";
    // let file = std::fs::File::open(path)?;
    // let buf_reader = BufReader::new(file);
    // let mut archive = ZipArchive::new(buf_reader)?;
    //
    // archive.file_names()
    //     .inspect(|n| println!("{}", n))
    //     .last();

    let epub = EPUBReader::new(path)?;
    // println!("{:?}", epub.package_document_paths());
    for package_document in epub.package_documents {
        println!("{:?}", package_document);
    }
    Ok(())
}