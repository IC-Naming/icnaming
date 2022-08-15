use crate::canister_api::AccountIdentifier;

use crate::named_canister_ids::{CanisterNames, DEV_NAMED_CANISTER_IDS};
use candid::Principal;
use const_env::from_env;
use log::info;
use once_cell::sync::Lazy;
use std::str::FromStr;

// const default ttl
pub const DEFAULT_TTL: u64 = 60 * 10;
pub const PAGE_INPUT_MIN_LIMIT: usize = 1;
pub const PAGE_INPUT_MAX_LIMIT: usize = 100_000;
pub const PAGE_INPUT_MIN_OFFSET: usize = 0;
pub const PAGE_INPUT_MAX_OFFSET: usize = 10_000_000;

// resolver keys
pub const RESOLVER_KEY_ETH: &str = "token.eth";
pub const RESOLVER_KEY_BTC: &str = "token.btc";
// obsolete: split into two keys RESOLVER_KEY_ICP_PRINCIPAL and RESOLVER_KEY_ICP_ACCOUNT_ID
pub const RESOLVER_KEY_ICP: &str = "token.icp";
pub const RESOLVER_KEY_LTC: &str = "token.ltc";
pub const RESOLVER_KEY_ICP_CANISTER: &str = "canister.icp";
pub const RESOLVER_KEY_ICP_PRINCIPAL: &str = "principal.icp";
pub const RESOLVER_KEY_ICP_ACCOUNT_ID: &str = "account_id.icp";
pub const RESOLVER_KEY_EMAIL: &str = "email";
pub const RESOLVER_KEY_URL: &str = "url";
pub const RESOLVER_KEY_AVATAR: &str = "avatar";
pub const RESOLVER_KEY_DESCRIPTION: &str = "description";
pub const RESOLVER_KEY_NOTICE: &str = "notice";
pub const RESOLVER_KEY_KEYWORDS: &str = "keywords";
pub const RESOLVER_KEY_LOCATION: &str = "location";
pub const RESOLVER_KEY_DISPLAY_NAME: &str = "display_name";
pub const RESOLVER_KEY_TWITTER: &str = "com.twitter";
pub const RESOLVER_KEY_GITHUB: &str = "com.github";
pub const RESOLVER_KEY_FACEBOOK: &str = "com.facebook";
pub const RESOLVER_KEY_MEDIUM: &str = "com.medium";
pub const RESOLVER_KEY_DISCORD: &str = "com.discord";
pub const RESOLVER_KEY_TELEGRAM: &str = "com.telegram";
pub const RESOLVER_KEY_INSTAGRAM: &str = "com.instagram";
pub const RESOLVER_KEY_REDDIT: &str = "com.reddit";
pub const RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL: &str =
    "settings.reverse_resolution.principal";

pub const RESOLVER_KEY_DSCVR: &str = "com.dscvr";
pub const RESOLVER_KEY_DISTRIKT: &str = "com.distrikt";
pub const RESOLVER_KEY_RELATION: &str = "com.relation";
pub const RESOLVER_KEY_OPENCHAT: &str = "com.openchat";

pub const RESOLVER_VALUE_MAX_LENGTH: usize = 512;
pub const RESOLVER_KEY_MAX_LENGTH: usize = 64;
pub const RESOLVER_ITEM_MAX_COUNT: usize = 30;

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum WellKnownResolverKey {
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
    Facebook,
    Medium,
    Discord,
    Telegram,
    Instagram,
    Reddit,
    Dscvr,
    Distrikt,
    Relation,
    OpenChat,
    SettingReverseResolutionPrincipal,
    Location,
    DisplayName,
}

impl WellKnownResolverKey {
    pub fn parse(s: &str) -> Option<WellKnownResolverKey> {
        match s {
            RESOLVER_KEY_ETH => Some(WellKnownResolverKey::Eth),
            RESOLVER_KEY_BTC => Some(WellKnownResolverKey::Btc),
            RESOLVER_KEY_ICP => Some(WellKnownResolverKey::Icp),
            RESOLVER_KEY_LTC => Some(WellKnownResolverKey::Ltc),
            RESOLVER_KEY_ICP_CANISTER => Some(WellKnownResolverKey::IcpCanister),
            RESOLVER_KEY_ICP_PRINCIPAL => Some(WellKnownResolverKey::IcpPrincipal),
            RESOLVER_KEY_ICP_ACCOUNT_ID => Some(WellKnownResolverKey::IcpAccountId),
            RESOLVER_KEY_EMAIL => Some(WellKnownResolverKey::Email),
            RESOLVER_KEY_URL => Some(WellKnownResolverKey::Url),
            RESOLVER_KEY_AVATAR => Some(WellKnownResolverKey::Avatar),
            RESOLVER_KEY_DESCRIPTION => Some(WellKnownResolverKey::Description),
            RESOLVER_KEY_NOTICE => Some(WellKnownResolverKey::Notice),
            RESOLVER_KEY_KEYWORDS => Some(WellKnownResolverKey::Keywords),
            RESOLVER_KEY_TWITTER => Some(WellKnownResolverKey::Twitter),
            RESOLVER_KEY_GITHUB => Some(WellKnownResolverKey::Github),
            RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL => {
                Some(WellKnownResolverKey::SettingReverseResolutionPrincipal)
            }
            RESOLVER_KEY_LOCATION => Some(WellKnownResolverKey::Location),
            RESOLVER_KEY_DISPLAY_NAME => Some(WellKnownResolverKey::DisplayName),
            RESOLVER_KEY_FACEBOOK => Some(WellKnownResolverKey::Facebook),
            RESOLVER_KEY_MEDIUM => Some(WellKnownResolverKey::Medium),
            RESOLVER_KEY_DISCORD => Some(WellKnownResolverKey::Discord),
            RESOLVER_KEY_TELEGRAM => Some(WellKnownResolverKey::Telegram),
            RESOLVER_KEY_INSTAGRAM => Some(WellKnownResolverKey::Instagram),
            RESOLVER_KEY_REDDIT => Some(WellKnownResolverKey::Reddit),
            RESOLVER_KEY_DSCVR => Some(WellKnownResolverKey::Dscvr),
            RESOLVER_KEY_DISTRIKT => Some(WellKnownResolverKey::Distrikt),
            RESOLVER_KEY_RELATION => Some(WellKnownResolverKey::Relation),
            RESOLVER_KEY_OPENCHAT => Some(WellKnownResolverKey::OpenChat),
            _ => None,
        }
    }
}

pub const MAX_REGISTRY_OPERATOR_COUNT: usize = 10;
pub const MAX_COUNT_USER_FAVORITES: usize = 100;
pub const MAX_LENGTH_USER_FAVORITES: usize = 256;

pub const MAX_LENGTH_OF_NAME_QUOTA_TYPE: u8 = 7;

pub const NAMING_ENV_DEV: &str = "dev";
pub const NAMING_ENV_STAGING: &str = "staging";
pub const NAMING_ENV_PRODUCTION: &str = "production";

#[from_env]
pub const NAMING_CANISTER_ENV: &str = "dev";

pub enum NamingEnv {
    Dev,
    Staging,
    Production,
}

pub fn is_env(env: NamingEnv) -> bool {
    match env {
        NamingEnv::Dev => NAMING_CANISTER_ENV == NAMING_ENV_DEV,
        NamingEnv::Staging => NAMING_CANISTER_ENV == NAMING_ENV_STAGING,
        NamingEnv::Production => NAMING_CANISTER_ENV == NAMING_ENV_PRODUCTION,
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
