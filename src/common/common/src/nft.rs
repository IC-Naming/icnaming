use crate::token_identifier::{TokenIdentifier, User};
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
    pub name: User,
    pub symbol: Principal,
    pub decimals: TokenIdentifier,
    pub metadata: Option<Vec<u8>>,
}
#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct NonFungible {
    pub metadata: Option<Vec<u8>>,
}
