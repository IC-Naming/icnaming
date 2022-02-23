use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use ic_cdk::export::Principal;
use ic_crypto_sha256::Sha224;
use rstest::*;

#[fixture]
pub fn anonymous_user() -> Principal {
    Principal::anonymous()
}

pub fn mock_user(index: u32) -> Principal {
    let mut principal_bytes = vec![0u8; 29];
    // The first four bytes are the index.
    principal_bytes[0..4].copy_from_slice(&index.to_be_bytes());
    Principal::from_slice(&principal_bytes)
}

pub fn mock_account_id(index: u32, subaccount: u32) -> Vec<u8> {
    // create a subaccount from subaccount
    let mut account_id_bytes = vec![0u8; 32];
    // The first four bytes are the subaccount.
    account_id_bytes[0..4].copy_from_slice(&subaccount.to_be_bytes());
    let account_bytes: [u8; 32] = account_id_bytes.as_slice().try_into().unwrap();
    let subaccount = Subaccount(account_bytes);
    AccountIdentifier::new(mock_user(index), Some(subaccount)).to_vec()
}

#[fixture]
pub fn mock_user1() -> Principal {
    mock_user(1)
}

#[fixture]
pub fn mock_user2() -> Principal {
    mock_user(2)
}

#[fixture]
pub fn mock_user3() -> Principal {
    mock_user(3)
}

#[fixture]
pub fn mock_now() -> u64 {
    15_844_844_000_000_000
}

#[derive(Serialize, Deserialize, CandidType, Clone, Hash, Debug, PartialEq, Eq, Copy)]
#[serde(transparent)]
pub struct Subaccount(pub [u8; 32]);

pub static SUB_ACCOUNT_ZERO: Subaccount = Subaccount([0; 32]);
static ACCOUNT_DOMAIN_SEPERATOR: &[u8] = b"\x0Aaccount-id";

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccountIdentifier {
    pub hash: [u8; 28],
}

impl AccountIdentifier {
    pub fn new(account: Principal, sub_account: Option<Subaccount>) -> AccountIdentifier {
        let mut hash = Sha224::new();
        hash.write(ACCOUNT_DOMAIN_SEPERATOR);
        hash.write(account.as_slice());

        let sub_account = sub_account.unwrap_or(SUB_ACCOUNT_ZERO.clone());
        hash.write(&sub_account.0[..]);

        AccountIdentifier {
            hash: hash.finish(),
        }
    }

    pub fn from_hex(hex_str: &str) -> Result<AccountIdentifier, String> {
        let hex: Vec<u8> = hex::decode(hex_str).map_err(|e| e.to_string())?;
        Self::from_slice(&hex[..])
    }

    /// Goes from the canonical format (with checksum) encoded in bytes rather
    /// than hex to AccountIdentifier
    pub fn from_slice(v: &[u8]) -> Result<AccountIdentifier, String> {
        // Trim this down when we reach rust 1.48
        let hex: Box<[u8; 32]> = match v.to_vec().into_boxed_slice().try_into() {
            Ok(h) => h,
            Err(_) => {
                let hex_str = hex::encode(v);
                return Err(format!(
                    "{} has a length of {} but we expected a length of 64",
                    hex_str,
                    hex_str.len()
                ));
            }
        };
        check_sum(*hex)
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_vec())
    }

    pub fn to_vec(&self) -> Vec<u8> {
        [&self.generate_checksum()[..], &self.hash[..]].concat()
    }

    pub fn generate_checksum(&self) -> [u8; 4] {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&self.hash);
        hasher.finalize().to_be_bytes()
    }
}

impl Display for AccountIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.to_hex().fmt(f)
    }
}

impl FromStr for AccountIdentifier {
    type Err = String;

    fn from_str(s: &str) -> Result<AccountIdentifier, String> {
        AccountIdentifier::from_hex(s)
    }
}

fn check_sum(hex: [u8; 32]) -> Result<AccountIdentifier, String> {
    // Get the checksum provided
    let found_checksum = &hex[0..4];

    // Copy the hash into a new array
    let mut hash = [0; 28];
    hash.copy_from_slice(&hex[4..32]);

    let account_id = AccountIdentifier { hash };
    let expected_checksum = account_id.generate_checksum();

    // Check the generated checksum matches
    if expected_checksum == found_checksum {
        Ok(account_id)
    } else {
        Err(format!(
            "Checksum failed for {}, expected check bytes {} but found {}",
            hex::encode(&hex[..]),
            hex::encode(expected_checksum),
            hex::encode(found_checksum),
        ))
    }
}
