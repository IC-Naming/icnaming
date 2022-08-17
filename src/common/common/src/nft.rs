use crate::token_identifier::TokenIdentifier;
use candid::{CandidType, Deserialize, Principal};

#[derive(CandidType, Debug, Clone, Deserialize)]
pub enum Metadata {
    #[serde(rename = "fungible")]
    Fungible(Fungible),
    #[serde(rename = "nonfungible")]
    NonFungible(NonFungible),
}

#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct Fungible {
    pub name: FungibleUser,
    pub symbol: Principal,
    pub decimals: TokenIdentifier,
    pub metadata: Option<Vec<u8>>,
}

// A user can be any principal or canister, which can hold a balance
#[derive(CandidType, Debug, Clone, Deserialize)]
pub enum FungibleUser {
    #[serde(rename = "address")]
    Address(String),
    #[serde(rename = "principal")]
    Principal(Principal),
}
#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct NonFungible {
    pub metadata: Option<Vec<u8>>,
}
