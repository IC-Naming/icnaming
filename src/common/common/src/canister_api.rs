use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use async_trait::async_trait;
use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::call::RejectionCode;
use ic_cdk::call;
use ic_crypto_sha256::Sha224;
use log::{debug, error};
use serde::Serialize;

use crate::cycles_minting_types::IcpXdrConversionRateCertifiedResponse;
use crate::dto::*;
use crate::errors::{ErrorInfo, ICNSActorResult, ICNSError};
use crate::icnaming_ledger_types::*;
use crate::named_canister_ids::get_named_get_canister_id;

pub mod ic_impl;

async fn call_core<T, TResult>(
    canister_name: &str,
    method: &str,
    args: T,
    logging: bool,
) -> Result<TResult, ICNSError>
where
    T: candid::utils::ArgumentEncoder,
    TResult: for<'a> Deserialize<'a> + CandidType + Debug,
{
    if logging {
        debug!("Calling {}::{}", canister_name, method);
    }
    let canister_id = get_named_get_canister_id(canister_name);
    let call_result: Result<(TResult,), (RejectionCode, String)> =
        call(canister_id, method, args).await;
    if call_result.is_err() {
        let (code, message) = call_result.err().unwrap();
        let code_string = format!("{:?}", code);
        error!(
            "{}::{} failed with code {}: {}",
            canister_name, method, code_string, message
        );
        return Err(ICNSError::CanisterCallError {
            message,
            rejection_code: code_string,
        });
    }
    let result = call_result.unwrap();
    if logging {
        debug!(
            "Call canister {} with method {} result: {:?}",
            canister_name, method, result
        );
    }
    Ok(result.0)
}

async fn call_canister_as_icns_result<T, TResult>(
    canister_name: &str,
    method: &str,
    args: T,
) -> ICNSActorResult<TResult>
where
    T: candid::utils::ArgumentEncoder,
    TResult: for<'a> Deserialize<'a> + CandidType + Debug,
{
    let result = call_core::<T, ICNSActorResult<TResult>>(canister_name, method, args, true).await;
    match result {
        Ok(result) => result,
        Err(error) => Err(ErrorInfo::from(error)),
    }
}

async fn call_canister_as_result<T, TResult>(
    canister_name: &str,
    method: &str,
    args: T,
) -> ICNSActorResult<TResult>
where
    T: candid::utils::ArgumentEncoder,
    TResult: for<'a> Deserialize<'a> + CandidType + Debug,
{
    call_core::<T, TResult>(canister_name, method, args, true)
        .await
        .map_err(ErrorInfo::from)
}

async fn call_canister_as_result_no_logging<T, TResult>(
    canister_name: &str,
    method: &str,
    args: T,
) -> ICNSActorResult<TResult>
where
    T: candid::utils::ArgumentEncoder,
    TResult: for<'a> Deserialize<'a> + CandidType + Debug,
{
    call_core::<T, TResult>(canister_name, method, args, false)
        .await
        .map_err(ErrorInfo::from)
}

#[async_trait]
pub trait IRegistryApi {
    async fn set_subdomain_owner(
        &self,
        label: String,
        parent_name: String,
        sub_owner: Principal,
        ttl: u64,
        resolver: Principal,
    ) -> ICNSActorResult<RegistryDto>;

    async fn reclaim_name(
        &self,
        name: String,
        owner: Principal,
        resolver: Principal,
    ) -> ICNSActorResult<bool>;

    async fn transfer(
        &self,
        name: String,
        new_owner: Principal,
        resolver: Principal,
    ) -> ICNSActorResult<bool>;

    async fn get_resolver(&self, label: &str) -> ICNSActorResult<Principal>;
    async fn get_users(&self, name: &str) -> ICNSActorResult<RegistryUsers>;
}

#[async_trait]
pub trait IResolverApi {
    async fn ensure_resolver_created(&self, name: String) -> ICNSActorResult<bool>;
    async fn remove_resolvers(&self, names: Vec<String>) -> ICNSActorResult<bool>;
}

#[async_trait]
pub trait IRegistrarApi {
    async fn import_quota(&self, request: ImportQuotaRequest)
        -> ICNSActorResult<ImportQuotaStatus>;
    async fn register_from_gateway(&self, name: String, owner: Principal) -> ICNSActorResult<bool>;
}

#[async_trait]
pub trait IICNamingLedgerApi {
    async fn add_payment(&self, request: AddPaymentRequest) -> ICNSActorResult<AddPaymentResponse>;
    async fn verify_payment(
        &self,
        request: VerifyPaymentRequest,
    ) -> ICNSActorResult<VerifyPaymentResponse>;
    async fn get_tip_of_ledger(
        &self,
        request: GetTipOfLedgerRequest,
    ) -> ICNSActorResult<GetTipOfLedgerResponse>;
    async fn refund_payment(
        &self,
        request: RefundPaymentRequest,
    ) -> ICNSActorResult<RefundPaymentResponse>;
    async fn sync_icp_payment(
        &self,
        request: SyncICPPaymentRequest,
    ) -> ICNSActorResult<SyncICPPaymentResponse>;
}

#[async_trait]
pub trait ICyclesMintingApi {
    async fn get_icp_xdr_conversion_rate(
        &self,
    ) -> ICNSActorResult<IcpXdrConversionRateCertifiedResponse>;
}

#[derive(Serialize, Deserialize, CandidType, Clone, Hash, Debug, PartialEq, Eq, Copy)]
#[serde(transparent)]
pub struct Subaccount(pub [u8; 32]);

pub const EMPTY_SUBACCOUNT: Subaccount = Subaccount([0; 32]);

static ACCOUNT_DOMAIN_SEPERATOR: &[u8] = b"\x0Aaccount-id";

pub type AccountId = [u8; 32];

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccountIdentifier {
    hash: [u8; 28],
}

impl AccountIdentifier {
    pub fn new(account: Principal, sub_account: Option<Subaccount>) -> AccountIdentifier {
        let mut hash = Sha224::new();
        hash.write(ACCOUNT_DOMAIN_SEPERATOR);
        hash.write(account.as_slice());

        let sub_account = sub_account.unwrap_or(EMPTY_SUBACCOUNT.clone());
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

    /// Converts this account identifier into a binary "address".
    /// The address is CRC32(identifier) . identifier.
    pub fn to_address(&self) -> AccountId {
        let mut result = [0u8; 32];
        result[0..4].copy_from_slice(&self.generate_checksum());
        result[4..32].copy_from_slice(&self.hash);
        result
    }
}

impl Display for AccountIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_hex().as_str())
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

// Amount of tokens, measured in 10^-8 of a token.
#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct Tokens {
    e8s: u64,
}

impl Tokens {
    pub fn new(e8s: u64) -> Self {
        Tokens { e8s }
    }
}

pub const ICP_FEE: Tokens = Tokens { e8s: 10_000 };

// Number of nanoseconds from the UNIX epoch in UTC timezone.
#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct TimeStamp {
    timestamp_nanos: u64,
}

// An arbitrary number associated with a transaction.
// The caller can set it in a `transfer` call as a correlation identifier.
pub type Memo = u64;

#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct TransferArgs {
    // Transaction memo.
    // See comments for the `Memo` type.
    pub memo: Memo,
    // The amount that the caller wants to transfer to the destination address.
    pub amount: Tokens,
    // The amount that the caller pays for the transaction.
    // Must be 10000 e8s.
    pub fee: Tokens,
    // The subaccount from which the caller wants to transfer funds.
    // If null, the ledger uses the default (all zeros) subaccount to compute the source address.
    // See comments for the `SubAccount` type.
    pub from_subaccount: Option<Subaccount>,
    // The destination account.
    // If the transfer is successful, the balance of this address increases by `amount`.
    pub to: AccountId,
    // The point in time when the caller created this request.
    // If null, the ledger uses current IC time as the timestamp.
    pub created_at_time: Option<TimeStamp>,
}

pub type BlockHeight = u64;
pub type BlockIndex = u64;

#[derive(CandidType, Debug, Clone, Deserialize)]
pub enum TransferError {
    // The fee that the caller specified in the transfer request was not the one that ledger expects.
    // The caller can change the transfer fee to the `expected_fee` and retry the request.
    BadFee { expected_fee: Tokens },
    // The account specified by the caller doesn't have enough funds.
    InsufficientFunds { balance: Tokens },
    // The request is too old.
    // The ledger only accepts requests created within 24 hours window.
    // This is a non-recoverable error.
    TxTooOld { allowed_window_nanos: u64 },
    // The caller specified `created_at_time` that is too far in future.
    // The caller can retry the request later.
    TxCreatedInFuture,
    // The ledger has already executed the request.
    // `duplicate_of` field is equal to the index of the block containing the original transaction.
    TxDuplicate { duplicate_of: BlockIndex },
}

#[derive(CandidType, Debug, Clone, Deserialize)]
pub enum TransferResult {
    Ok(BlockHeight),
    Err(TransferError),
}

#[async_trait]
pub trait ILedgerApi {
    async fn transfer(&self, args: TransferArgs) -> ICNSActorResult<TransferResult>;
}
