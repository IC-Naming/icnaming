use std::collections::HashSet;
use std::io::Read;
use std::str::FromStr;

use candid::{decode_args, encode_args};
use flate2::read::ZlibDecoder;
use ic_cdk::export::Principal;
use ic_crypto_sha256::Sha256;
use log::debug;

use common::state::StableState;

use crate::user_quota_store::QuotaType;

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

pub struct ImportQuotaItem {
    pub owner: Principal,
    pub quota_type: QuotaType,
    pub diff: u32,
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
        let file_hash = Sha256::hash(file_content.as_slice()).to_vec();
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
                quota_type: QuotaType::from_str(quota_type).unwrap(),
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
}

// -- auto-generated START ACCEPTABLE_HASHES build.rs --
pub const ACCEPTABLE_HASHES: &[&str] = &["af7619170a528b2ef642224d133297ce3756da745fa4cd84075b59f224e7ab9b", "64e72c990a42af6aaf4def6d20b04b827bc302c695efff6d101d39576a6e0232", "fdcbd2e084ffc0ad0211bdffa818f3a2d3b70e4652742239e94d6f79c484696e"];
// -- auto-generated END ACCEPTABLE_HASHES build.rs --
