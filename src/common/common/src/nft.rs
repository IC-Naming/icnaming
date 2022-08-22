use crate::canister_api::{AccountIdentifier, Subaccount};
use crate::token_identifier::TokenIdentifier;
use crate::NamingError;
use candid::{CandidType, Deserialize, Principal};

// Additional data field for transfers to describe the tx
// Data will also be forwarded to notify callback
pub type Memo = Vec<u8>;

#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct TransferRequest {
    pub from: User,
    pub to: User,
    pub token: TokenIdentifier,
    pub amount: u128,
    pub memo: Memo,
    pub notify: bool,
    pub subaccount: Option<Subaccount>,
}

#[derive(CandidType, Debug, Clone, Deserialize)]
pub enum TransferError {
    Unauthorized(AccountIdentifier),
    InsufficientBalance,
    Rejected,
    //Rejected by canister
    InvalidToken(TokenIdentifier),
    CannotNotify(AccountIdentifier),
    Other(String),
}
impl From<NamingError> for TransferError {
    fn from(error: NamingError) -> Self {
        TransferError::Other(error.to_string())
    }
}

#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct ApproveRequest {
    pub subaccount: Option<Subaccount>,
    pub spender: Principal,
    pub allowance: u128,
    pub token: TokenIdentifier,
}

#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct AllowanceRequest {
    pub owner: User,
    pub spender: Principal,
    pub token: TokenIdentifier,
}

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

// A user can be any principal or canister, which can hold a balance
#[derive(CandidType, Debug, Clone, Deserialize)]
pub enum User {
    #[serde(rename = "address")]
    Address(AccountIdentifier),
    #[serde(rename = "principal")]
    Principal(Principal),
}

impl User {
    pub fn get_principal(&self) -> Result<Principal, NamingError> {
        match self {
            User::Address(_) => Err(NamingError::AccountIdentifierNotSupported),
            User::Principal(principal) => Ok(principal.clone()),
        }
    }
}

#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct NonFungible {
    pub metadata: Option<Vec<u8>>,
}

pub type NFTServiceResult<T> = anyhow::Result<T, CommonError>;
pub type NFTTransferServiceResult<T> = anyhow::Result<T, TransferError>;

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
