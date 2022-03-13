use std::collections::HashSet;



use candid::{decode_args, encode_args};





use common::state::StableState;



#[derive(Default)]
pub struct QuotaImportStore {
    imported_file_hashes: HashSet<Vec<u8>>,
    // obsolete: acceptable_file_hashes
    acceptable_file_hashes: HashSet<Vec<u8>>,
}

impl StableState for QuotaImportStore {
    fn encode(&self) -> Vec<u8> {
        encode_args((&self.imported_file_hashes, &self.acceptable_file_hashes)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (imported_file_hashes, mut acceptable_file_hashes): (
            HashSet<Vec<u8>>,
            HashSet<Vec<u8>>,
        ) = decode_args(&bytes).unwrap();

        // data have been moved to registrar_control_gateway
        acceptable_file_hashes.clear();

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
    pub fn verify_hash(&self, file_hash: &Vec<u8>) -> Result<(), ImportError> {
        if self.imported_file_hashes.contains(file_hash) {
            return Err(ImportError::FileAlreadyImported);
        }
        Ok(())
    }

    pub fn add_imported_file_hash(&mut self, file_hash: Vec<u8>) {
        self.imported_file_hashes.insert(file_hash);
    }
}
