use crate::token_identifier::TokenIdentifier;
use crate::NamingError;
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

pub type NFTServiceResult<T> = anyhow::Result<T, CommonError>;

//NFT error respone
#[derive(CandidType, Debug, Clone, Deserialize)]
pub enum CommonError {
    InvalidToken(TokenIdentifier),
    Other(String),
}
impl From<NamingError> for CommonError {
    fn from(error: NamingError) -> Self {
        CommonError::Other(error.to_string())
    }
}
