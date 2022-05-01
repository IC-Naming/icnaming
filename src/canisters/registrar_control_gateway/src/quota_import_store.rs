use std::collections::HashSet;
use std::io::Read;
use std::str::FromStr;

use candid::{decode_args, encode_args};
use flate2::read::ZlibDecoder;
use ic_cdk::export::Principal;
use log::debug;
use sha2::Digest;
use sha2::Sha256;

use common::dto::ImportQuotaItem;
use common::state::StableState;

#[derive(Default)]
pub struct QuotaImportStore {
    imported_file_hashes: HashSet<Vec<u8>>,
    acceptable_file_hashes: HashSet<Vec<u8>>,
}

impl StableState for QuotaImportStore {
    fn encode(&self) -> Vec<u8> {
        encode_args((&self.imported_file_hashes, &self.acceptable_file_hashes)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (imported_file_hashes, acceptable_file_hashes): (HashSet<Vec<u8>>, HashSet<Vec<u8>>) =
            decode_args(&bytes).unwrap();

        Ok(QuotaImportStore {
            imported_file_hashes,
            acceptable_file_hashes,
        })
    }
}

#[derive(Debug)]
pub enum ImportError {
    FileAlreadyImported,
    FileNotAcceptable,
}

impl QuotaImportStore {
    pub fn verify_and_parse(
        &self,
        file_content: &[u8],
    ) -> Result<(Vec<ImportQuotaItem>, Vec<u8>), ImportError> {
        let mut decoder = ZlibDecoder::new(file_content);
        let mut file_content = Vec::new();
        decoder.read_to_end(&mut file_content).unwrap();
        let mut sha256 = Sha256::new();
        sha256.update(&file_content);
        let file_hash = sha256.finalize().to_vec();
        debug!("File hash: {}", hex::encode(&file_hash));
        if self.imported_file_hashes.contains(&file_hash) {
            return Err(ImportError::FileAlreadyImported);
        }
        if !self.acceptable_file_hashes.contains(&file_hash) {
            return Err(ImportError::FileNotAcceptable);
        }
        let mut import_quota_items = Vec::new();
        let file_content = String::from_utf8(file_content.to_vec()).unwrap();
        for line in file_content.lines() {
            let mut parts = line.split(',');
            let owner = parts.next().unwrap();
            let quota_type = parts.next().unwrap();
            let diff = parts.next().unwrap().parse::<u32>().unwrap();
            import_quota_items.push(ImportQuotaItem {
                owner: Principal::from_str(owner).unwrap(),
                quota_type: quota_type.to_string(),
                diff,
            });
        }
        Ok((import_quota_items, file_hash))
    }

    pub fn add_imported_file_hash(&mut self, file_hash: Vec<u8>) {
        self.imported_file_hashes.insert(file_hash);
    }

    pub fn add_acceptable_file_hash(&mut self, file_hashes: Vec<Vec<u8>>) {
        for file_hash in file_hashes {
            self.acceptable_file_hashes.insert(file_hash);
        }
    }

    pub fn get_acceptable_file_hashes(&self) -> &HashSet<Vec<u8>> {
        &self.acceptable_file_hashes
    }

    pub fn get_imported_file_hashes(&self) -> &HashSet<Vec<u8>> {
        &self.imported_file_hashes
    }
}
