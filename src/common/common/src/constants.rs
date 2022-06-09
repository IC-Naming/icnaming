use crate::canister_api::AccountIdentifier;
use crate::errors::NamingError;
use crate::named_canister_ids::{CanisterNames, DEV_NAMED_CANISTER_IDS};
use candid::Principal;
use const_env::from_env;
use log::{debug, info};
use once_cell::sync::Lazy;
use std::str::FromStr;

// const default ttl
pub const DEFAULT_TTL: u64 = 60 * 10;
pub const PAGE_INPUT_MIN_LIMIT: usize = 1;
pub const PAGE_INPUT_MAX_LIMIT: usize = 100;
pub const PAGE_INPUT_MIN_OFFSET: usize = 0;
pub const PAGE_INPUT_MAX_OFFSET: usize = 10_000;

// resolver keys
pub const RESOLVER_KEY_ETH: &str = "token.eth";
pub const RESOLVER_KEY_BTC: &str = "token.btc";
// obsolete: split into two keys RESOLVER_KEY_ICP_PRINCIPAL and RESOLVER_KEY_ICP_ACCOUNT_ID
pub const RESOLVER_KEY_ICP: &str = "token.ic";
pub const RESOLVER_KEY_LTC: &str = "token.ltc";
pub const RESOLVER_KEY_ICP_CANISTER: &str = "canister.ic";
pub const RESOLVER_KEY_ICP_PRINCIPAL: &str = "principal.ic";
pub const RESOLVER_KEY_ICP_ACCOUNT_ID: &str = "account_id.ic";
pub const RESOLVER_KEY_EMAIL: &str = "email";
pub const RESOLVER_KEY_URL: &str = "url";
pub const RESOLVER_KEY_AVATAR: &str = "avatar";
pub const RESOLVER_KEY_DESCRIPTION: &str = "description";
pub const RESOLVER_KEY_NOTICE: &str = "notice";
pub const RESOLVER_KEY_KEYWORDS: &str = "keywords";
pub const RESOLVER_KEY_TWITTER: &str = "com.twitter";
pub const RESOLVER_KEY_GITHUB: &str = "com.github";

pub const RESOLVER_VALUE_MAX_LENGTH: usize = 512;

pub enum ResolverKey {
    Eth,
    Btc,
    Icp,
    Ltc,
    IcpCanister,
    IcpPrincipal,
    IcpAccountId,
    Email,
    Url,
    Avatar,
    Description,
    Notice,
    Keywords,
    Twitter,
    Github,
}

impl FromStr for ResolverKey {
    type Err = NamingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            RESOLVER_KEY_ETH => Ok(ResolverKey::Eth),
            RESOLVER_KEY_BTC => Ok(ResolverKey::Btc),
            RESOLVER_KEY_ICP => Ok(ResolverKey::Icp),
            RESOLVER_KEY_LTC => Ok(ResolverKey::Ltc),
            RESOLVER_KEY_ICP_CANISTER => Ok(ResolverKey::IcpCanister),
            RESOLVER_KEY_ICP_PRINCIPAL => Ok(ResolverKey::IcpPrincipal),
            RESOLVER_KEY_ICP_ACCOUNT_ID => Ok(ResolverKey::IcpAccountId),
            RESOLVER_KEY_EMAIL => Ok(ResolverKey::Email),
            RESOLVER_KEY_URL => Ok(ResolverKey::Url),
            RESOLVER_KEY_AVATAR => Ok(ResolverKey::Avatar),
            RESOLVER_KEY_DESCRIPTION => Ok(ResolverKey::Description),
            RESOLVER_KEY_NOTICE => Ok(ResolverKey::Notice),
            RESOLVER_KEY_KEYWORDS => Ok(ResolverKey::Keywords),
            RESOLVER_KEY_TWITTER => Ok(ResolverKey::Twitter),
            RESOLVER_KEY_GITHUB => Ok(ResolverKey::Github),
            _ => Err(NamingError::InvalidResolverKey { key: s.to_string() }),
        }
    }
}

pub const MAX_REGISTRY_OPERATOR_COUNT: usize = 10;
pub const MAX_COUNT_USER_FAVORITES: usize = 100;
pub const MAX_LENGTH_USER_FAVORITES: usize = 256;

pub const MAX_QUOTA_ORDER_AMOUNT_EACH_TYPE: u32 = 10;
pub const MAX_LENGTH_OF_NAME_QUOTA_TYPE: u8 = 7;
pub const EXPIRE_TIME_OF_NAME_ORDER_IN_NS: u64 = 60 * 60 * 24 * 3 * 1_000_000_000;
pub const EXPIRE_TIME_OF_NAME_ORDER_AVAILABILITY_CHECK_IN_NS: u64 = 60 * 60 * 24 * 1_000_000_000;

pub const NAMING_ENV_DEV: &str = "dev";
pub const NAMING_ENV_STAGING: &str = "staging";
pub const NAMING_ENV_PRODUCTION: &str = "production";

#[from_env]
pub const NAMING_ENV: &str = NAMING_ENV_DEV;

pub enum NamingEnv {
    Dev,
    Staging,
    Production,
}

pub fn is_env(env: NamingEnv) -> bool {
    match env {
        NamingEnv::Dev => NAMING_ENV == NAMING_ENV_DEV,
        NamingEnv::Staging => NAMING_ENV == NAMING_ENV_STAGING,
        NamingEnv::Production => NAMING_ENV == NAMING_ENV_PRODUCTION,
    }
}

pub fn is_dev_env() -> bool {
    is_env(NamingEnv::Dev)
}

#[from_env]
pub const NAMING_TOP_LABEL: &str = "";

#[from_env]
pub const NAMING_MIN_REGISTRATION_YEAR: u32 = 1;

#[from_env]
pub const NAMING_MAX_REGISTRATION_YEAR: u32 = 10;

fn load_dev_or_env(name: CanisterNames, env_value: &str) -> Principal {
    if is_dev_env() {
        DEV_NAMED_CANISTER_IDS.with(|ids| {
            let ids = ids.borrow();
            let id = ids.get(&name);
            if let Some(id) = id {
                info!("load_dev_or_env: from dev id list {:?} = {}", name, id);
                *id
            } else {
                info!("load_dev_or_env: from env {:?} = {}", name, env_value);
                Principal::from_str(env_value).unwrap()
            }
        })
    } else {
        Principal::from_str(env_value).unwrap()
    }
}

#[from_env]
const NAMING_CANISTER_IDS_REGISTRAR: &str = "";
pub static CANISTER_IDS_REGISTRAR: Lazy<Principal> =
    Lazy::new(|| load_dev_or_env(CanisterNames::Registrar, NAMING_CANISTER_IDS_REGISTRAR));

#[from_env]
const NAMING_CANISTER_IDS_REGISTRAR_CONTROL_GATEWAY: &str = "";
pub static CANISTER_IDS_REGISTRAR_CONTROL_GATEWAY: Lazy<Principal> = Lazy::new(|| {
    load_dev_or_env(
        CanisterNames::RegistrarControlGateway,
        NAMING_CANISTER_IDS_REGISTRAR_CONTROL_GATEWAY,
    )
});

#[from_env]
const NAMING_CANISTER_IDS_REGISTRY: &str = "";
pub static CANISTER_IDS_REGISTRY: Lazy<Principal> =
    Lazy::new(|| load_dev_or_env(CanisterNames::Registry, NAMING_CANISTER_IDS_REGISTRY));
#[from_env]
const NAMING_CANISTER_IDS_RESOLVER: &str = "";
pub static CANISTER_IDS_RESOLVER: Lazy<Principal> =
    Lazy::new(|| load_dev_or_env(CanisterNames::Resolver, NAMING_CANISTER_IDS_RESOLVER));
#[from_env]
const NAMING_CANISTER_IDS_CYCLES_MINTING: &str = "";
pub static CANISTER_IDS_CYCLES_MINTING: Lazy<Principal> = Lazy::new(|| {
    load_dev_or_env(
        CanisterNames::CyclesMinting,
        NAMING_CANISTER_IDS_CYCLES_MINTING,
    )
});
#[from_env]
const NAMING_CANISTER_IDS_FAVORITES: &str = "";
pub static CANISTER_IDS_FAVORITES: Lazy<Principal> =
    Lazy::new(|| load_dev_or_env(CanisterNames::Favorites, NAMING_CANISTER_IDS_FAVORITES));
#[from_env]
const NAMING_CANISTER_IDS_LEDGER: &str = "";
pub static CANISTER_IDS_LEDGER: Lazy<Principal> =
    Lazy::new(|| load_dev_or_env(CanisterNames::Ledger, NAMING_CANISTER_IDS_LEDGER));
#[from_env]
const NAMING_CANISTER_IDS_DICP: &str = "";
pub static CANISTER_IDS_DICP: Lazy<Principal> =
    Lazy::new(|| load_dev_or_env(CanisterNames::DICP, NAMING_CANISTER_IDS_DICP));
#[from_env]
const NAMING_CANISTER_IDS_MYSTERY_BOX: &str = "";
pub static CANISTER_IDS_MYSTERY_BOX: Lazy<Principal> =
    Lazy::new(|| load_dev_or_env(CanisterNames::MysteryBox, NAMING_CANISTER_IDS_MYSTERY_BOX));
#[from_env]
const NAMING_CANISTER_IDS_NAMING_MARKETPLACE: &str = "";
pub static CANISTER_IDS_NAMING_MARKETPLACE: Lazy<Principal> = Lazy::new(|| {
    load_dev_or_env(
        CanisterNames::NamingMarketplace,
        NAMING_CANISTER_IDS_NAMING_MARKETPLACE,
    )
});

#[from_env]
pub const NAMING_PRINCIPAL_NAME_ADMIN: &str = "";
#[from_env]
pub const NAMING_PRINCIPAL_NAME_STATE_EXPORTER: &str = "";
#[from_env]
pub const NAMING_PRINCIPAL_NAME_TIMER_TRIGGER: &str = "";

#[from_env]
const NAMING_PRINCIPAL_DICP_RECEIVER: &str = "";
pub static DICP_RECEIVER: Lazy<&str> = Lazy::new(|| {
    if is_dev_env() {
        NAMING_CANISTER_IDS_REGISTRAR
    } else {
        NAMING_PRINCIPAL_DICP_RECEIVER
    }
});

#[from_env]
const NAMING_ACCOUNT_ID_ICP_RECEIVER: &str = "";
pub static ACCOUNT_ID_ICP_RECEIVER: Lazy<AccountIdentifier> =
    Lazy::new(|| AccountIdentifier::from_hex(NAMING_ACCOUNT_ID_ICP_RECEIVER).unwrap());

#[cfg(test)]
mod tests;
