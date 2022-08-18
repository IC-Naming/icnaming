#![allow(dead_code)]

use crate::canister_api::AccountIdentifier;
use crate::{NamingError, ServiceResult};
use candid::{CandidType, Deserialize, Principal};

const CANISTER_ID_HASH_LEN_IN_BYTES: usize = 10;
const TOKEN_ID_PREFIX: [u8; 4] = [10, 116, 105, 100]; //b"\x0Atid"

#[derive(Default, Deserialize, Copy, CandidType, Clone, Hash, Eq, PartialEq, Debug)]
pub struct TokenIndex {
    pub value: u32,
}

#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct CanisterId {
    pub value: Principal,
}

#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct TokenObj {
    pub index: TokenIndex,
    pub canister: Vec<u8>,
}

#[derive(Deserialize, CandidType, Clone, Hash, Eq, PartialEq, Debug)]
pub struct TokenIdentifier {
    pub value: String,
}

pub fn is_valid_token_id(tid: &TokenIdentifier, p: &CanisterId) -> bool {
    let t_parsed = decode_token_id(tid);
    match t_parsed {
        Ok(t) => t.canister == p.value.as_slice().to_vec(),
        Err(_) => false,
    }
}

pub fn get_token_index(tid: &TokenIdentifier) -> TokenIndex {
    let tobj = decode_token_id(tid).unwrap();
    tobj.index
}

pub fn get_valid_token_index(
    tid: &TokenIdentifier,
    canister_id: &CanisterId,
) -> ServiceResult<TokenIndex> {
    if is_valid_token_id(tid, canister_id) {
        let index = get_token_index(tid);
        return Ok(index);
    }
    Err(NamingError::InvalidTokenIdentifier)
}

pub fn get_valid_canister_id(principal: &Principal) -> ServiceResult<CanisterId> {
    if principal.as_slice().len() == CANISTER_ID_HASH_LEN_IN_BYTES {
        return Ok(CanisterId {
            value: principal.clone(),
        });
    }
    Err(NamingError::InvalidCanisterId)
}

pub fn decode_token_id(tid: &TokenIdentifier) -> ServiceResult<TokenObj> {
    let principal_parse_res = Principal::from_text(tid.value.clone());
    match principal_parse_res {
        Ok(principal) => {
            let bytes = principal.as_slice();
            if !bytes.starts_with(&TOKEN_ID_PREFIX) {
                return Ok(TokenObj {
                    index: TokenIndex { value: 0 },
                    canister: bytes.into(),
                });
            }
            let canister: Vec<u8> = bytes[4..(4 + CANISTER_ID_HASH_LEN_IN_BYTES)].to_vec();
            let mut token_index: [u8; 4] = Default::default();
            token_index.copy_from_slice(&bytes[14..]);

            return Ok(TokenObj {
                index: TokenIndex {
                    value: u32::from_be_bytes(token_index),
                },
                canister,
            });
        }
        Err(_) => Err(NamingError::InvalidTokenIdentifier),
    }
}

pub fn encode_token_id(canister_id: CanisterId, token_index: TokenIndex) -> TokenIdentifier {
    let mut blob: Vec<u8> = Vec::new();
    blob.extend_from_slice(&TOKEN_ID_PREFIX);
    blob.extend_from_slice(canister_id.value.as_slice());
    blob.extend_from_slice(&token_index.value.to_be_bytes());
    TokenIdentifier {
        value: Principal::from_slice(blob.as_slice()).to_text(),
    }
}

#[test]
fn encode_decode_tx_id() {
    let token_id = CanisterId {
        value: Principal::from_text("e3izy-jiaaa-aaaah-qacbq-cai").unwrap(),
    };
    let tx_id = "7hsvu-sikor-uwiaa-aaaaa-b4aaq-maqca-aabke-q".to_string();
    assert!(is_valid_token_id(
        &TokenIdentifier {
            value: tx_id.clone()
        },
        &token_id,
    ));
    let decode_res = decode_token_id(&TokenIdentifier {
        value: tx_id.clone(),
    })
    .unwrap();
    let de_tx_index = decode_res.index;
    let de_canister = Principal::from_slice(decode_res.canister.as_slice());
    assert_eq!(de_canister, token_id);
    assert_eq!(decode_res.index.value, de_tx_index.value);
    println!("{:?}", de_tx_index);
    let en_tx_id = encode_token_id(token_id, de_tx_index);
    assert_eq!(en_tx_id.value, tx_id);
}

#[test]
fn test_tx_id() {
    let token_id = CanisterId {
        value: Principal::from_text("e3izy-jiaaa-aaaah-qacbq-cai").unwrap(),
    };
    let tx_index = 1000u32;

    let en_tx_id = encode_token_id(token_id, TokenIndex { value: tx_index });
    assert!(en_tx_id.value.len() > 0, "result is {:?}", en_tx_id);
}
