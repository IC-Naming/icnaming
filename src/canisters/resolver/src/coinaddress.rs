//! Functions for validating the base58 hash checksums, including specifically
//! the bitcoin and litecoin addresses.
//! source from https://github.com/viraptor/coinaddress/blob/master/src/lib.rs

use num_bigint::BigUint;
use num_integer::Integer;
use num_traits::{ToPrimitive, Zero};
use sha2::Digest;
use sha2::Sha256;
use std::io::Write;

#[derive(PartialEq, Debug)]
pub enum ValidationError {
    /// Given address is too short to be valid
    TooShort,
    /// Encoding is not a valid base58
    InvalidEncoding,
    /// Computed hash does not match the embedded one
    HashMismatch,

    // currency specific
    /// This address is not a bitcoin address (testnet or real).
    /// May happen when attempting to validate btc address
    NotBitcoin,
    /// This address is not a litecoin address.
    /// May happen when attempting to validate ltc address
    NotLitecoin,
}

static BASE58_CHARS: &'static str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

fn decode_base58(bc: &str) -> Option<BigUint> {
    // decode base58 string as BigUint
    let mut result = BigUint::zero();
    let mut count = 0;
    let base = BigUint::from(58u32);
    for c in bc.chars() {
        let digit = BASE58_CHARS.find(c)?;
        result = result * &base + BigUint::from(digit);
        count += 1;
    }
    if count == 0 {
        return None;
    }
    Some(result)
}

fn biguint_to_bytes(n: BigUint) -> Vec<u8> {
    // convert BigUint to bytes
    let mut bytes = Vec::new();
    let mut n = n;
    let base = &BigUint::from(256u32);
    while n > Zero::zero() {
        let (q, r) = n.div_rem(base);
        bytes.push(r.to_u8().unwrap());
        n = q;
    }
    bytes.reverse();
    bytes
}

fn pad_to(v: Vec<u8>, len: usize) -> Vec<u8> {
    let mut tmp: Vec<u8> = v;
    while tmp.len() < len {
        tmp.insert(0, 0)
    }
    tmp
}

fn double_sha256(chunk: &[u8]) -> Vec<u8> {
    // sha256 for twice
    let mut hasher = Sha256::new();
    hasher.write(chunk);
    let mut hash = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.write(&hash);
    hash = hasher.finalize();
    // to vec
    hash.to_vec()
}

/// Validate provided generic base58 hash.
/// Returns the hash version/type if correct and an error otherwise.
pub fn validate_base58_hash(addr: &str) -> Result<usize, ValidationError> {
    if addr.len() == 0 {
        return Err(ValidationError::TooShort);
    }

    let big = match decode_base58(addr) {
        None => return Err(ValidationError::InvalidEncoding),
        Some(x) => x,
    };
    let bytes = biguint_to_bytes(big);
    let padded = pad_to(bytes, 25);

    let hash = double_sha256(&padded[0..padded.len() - 4]);
    let short_hash = &hash[0..4];
    let known = &padded[padded.len() - 4..padded.len()];
    if &short_hash[..] == known {
        Ok(padded[0] as usize)
    } else {
        Err(ValidationError::HashMismatch)
    }
}

/// Validate bitcoin address checksum.
/// Returns the hash version/type if correct and an error otherwise.
pub fn validate_btc_address(addr: &str) -> Result<usize, ValidationError> {
    match validate_base58_hash(addr) {
        Ok(0) => Ok(0),     // real address
        Ok(5) => Ok(5),     // script hash
        Ok(111) => Ok(111), // testnet address
        Ok(_) => Err(ValidationError::NotBitcoin),
        Err(x) => Err(x),
    }
}

/// Validate litecoin address checksum.
/// Returns the hash version/type if correct and an error otherwise.
pub fn validate_ltc_address(addr: &str) -> Result<usize, ValidationError> {
    match validate_base58_hash(addr) {
        Ok(48) => Ok(48),   // real address
        Ok(111) => Ok(111), // testnet address
        Ok(_) => Err(ValidationError::NotLitecoin),
        Err(x) => Err(x),
    }
}
