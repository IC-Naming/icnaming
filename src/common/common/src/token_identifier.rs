#![allow(dead_code)]
use crate::types::*;
use ic_cdk::export::Principal;
const CANISTER_ID_HASH_LEN_IN_BYTES: usize = 10;
const TOKEN_ID_PREFIX: [u8; 4] = [10, 116, 105, 100]; //b"\x0Atid"
pub fn is_valid_token_id(tid: TokenIdentifier, p: Principal) -> bool {
    let t_parsed = decode_token_id(tid);
    match t_parsed {
        Ok(t) => t.canister == p.as_slice().to_vec(),
        Err(_) => false,
    }
}

pub fn get_token_index(tid: TokenIdentifier) -> TokenIndex {
    let tobj = decode_token_id(tid).unwrap();
    tobj.index
}

pub fn decode_token_id(tid: TokenIdentifier) -> Result<TokenObj, String> {
    let principal_parse_res = Principal::from_text(tid);
    match principal_parse_res {
        Ok(principal) => {
            let bytes = principal.as_slice();
            if !bytes.starts_with(&TOKEN_ID_PREFIX) {
                return Ok(TokenObj {
                    index: 0,
                    canister: bytes.into(),
                });
            }
            let canister: Vec<u8> = bytes[4..(4 + CANISTER_ID_HASH_LEN_IN_BYTES)].to_vec();
            let mut token_index: [u8; 4] = Default::default();
            token_index.copy_from_slice(&bytes[14..]);

            return Ok(TokenObj {
                index: u32::from_be_bytes(token_index),
                canister: canister,
            });
        }
        Err(_) => Err("invalid token id".to_string()),
    }
}

pub fn encode_token_id(token_id: Principal, token_index: TokenIndex) -> TokenIdentifier {
    let mut blob: Vec<u8> = Vec::new();
    blob.extend_from_slice(&TOKEN_ID_PREFIX);
    blob.extend_from_slice(token_id.as_slice());
    blob.extend_from_slice(&token_index.to_be_bytes());

    Principal::from_slice(blob.as_slice()).to_text()
}

#[test]
fn encode_decode_tx_id() {
    let token_id = Principal::from_text("e3izy-jiaaa-aaaah-qacbq-cai").unwrap();
    let tx_id = "7hsvu-sikor-uwiaa-aaaaa-b4aaq-maqca-aabke-q".to_string();
    assert!(is_valid_token_id(tx_id.clone(), token_id));
    let decode_res = decode_token_id(tx_id.clone()).unwrap();
    let de_tx_index = decode_res.index;
    let de_canister = Principal::from_slice(decode_res.canister.as_slice());
    assert_eq!(de_canister, token_id);
    assert_eq!(decode_res.index, de_tx_index);

    let en_tx_id = encode_token_id(token_id, de_tx_index);
    assert_eq!(en_tx_id, tx_id);
}

#[test]
fn test_tx_id() {
    let token_id = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    let tx_index = 1000u32;

    let en_tx_id = encode_token_id(token_id, tx_index);
    assert!(en_tx_id.len() > 0, "result is {}", en_tx_id);
}
