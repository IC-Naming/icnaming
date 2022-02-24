use std::{fs, io};
use std::io::{Read, Write};

use flate2::write::ZlibEncoder;

fn main() {
    // create zlib encode bytes for each txt file in ../../quota_import_data/
    // and create get hash for each source file and write to ../../quota_import_data/*.hash

    let mut files = vec![];
    for entry in fs::read_dir("../../quota_import_data/").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().unwrap() == "csv" {
            files.push(path);
        }
    }

    let mut hashes = vec![];

    for file in files {
        let file_name = file.to_str().unwrap().to_string();
        let mut data = fs::read(file).unwrap();
        {
            let file_name = file_name.replace(".csv", ".hash");
            let hash = ic_crypto_sha256::Sha256::hash(&data);
            let hash = hex::encode(hash);
            hashes.push(hash.clone());

            let mut file = fs::File::create(file_name).unwrap();
            file.write_all(hash.as_bytes()).unwrap();
        }
        {
            let file_name = file_name.replace(".csv", ".zlib");
            let mut encoder = ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(&data).unwrap();
            let data = encoder.finish().unwrap();
            let mut file = fs::File::create(file_name).unwrap();
            file.write_all(&data).unwrap();
        }
    }

    // create illegal data file to test error handling
    {
        let mut file = fs::File::create("../../quota_import_data/illegal.zlib").unwrap();
        let mut encoder = ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all("illegal data".as_bytes()).unwrap();
        let data = encoder.finish().unwrap();
        file.write_all(&data).unwrap();
    }

    // update source code in src/quota_import_store.rs
    // update section between // -- auto-generated ACCEPTABLE_HASHES build.rs --

    let content = format!(
        "pub const ACCEPTABLE_HASHES: &[&str] = &[{}];",
        hashes
            .iter()
            .map(|hash| format!("\"{}\"", hash))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let mut content_pushed = false;

    let mut file = fs::File::open("src/quota_import_store.rs").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let mut lines = contents.lines();
    let mut new_contents = String::new();
    let mut found_start = false;
    let mut found_end = false;

    while let Some(line) = lines.next() {
        if line.contains("// -- auto-generated START ACCEPTABLE_HASHES build.rs --") {
            found_start = true;
            new_contents.push_str(line);
            new_contents.push_str("\n");
            new_contents.push_str(&content);
            new_contents.push_str("\n");
        }
        if !found_start || found_end {
            new_contents.push_str(line);
            new_contents.push_str("\n");
        }
        if line.contains("// -- auto-generated END ACCEPTABLE_HASHES build.rs --") {
            found_end = true;
            new_contents.push_str(line);
            new_contents.push_str("\n");
        }
    }

    let mut file = fs::File::create("src/quota_import_store.rs").unwrap();
    file.write_all(new_contents.as_bytes()).unwrap();
}
