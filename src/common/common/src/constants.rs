use crate::errors::ICNSError;
use std::str::FromStr;

// const default ttl
pub const DEFAULT_TTL: u64 = 60 * 10;
#[cfg(feature = "dev_canister")]
pub const TOP_LABEL: &str = "icp";
#[cfg(feature = "staging_canister")]
pub const TOP_LABEL: &str = "ticp";
#[cfg(feature = "production_canister")]
pub const TOP_LABEL: &str = "icp";

pub const PAGE_INPUT_MIN_LIMIT: usize = 1;
pub const PAGE_INPUT_MAX_LIMIT: usize = 100_000;
pub const PAGE_INPUT_MIN_OFFSET: usize = 0;
pub const PAGE_INPUT_MAX_OFFSET: usize = 10_000_000;

pub const DEFAULT_MIN_REGISTRATION_YEAR: u32 = 1;
pub const DEFAULT_MAX_REGISTRATION_YEAR: u32 = 10;

// resolver keys
pub const RESOLVER_KEY_ETH: &str = "token.eth";
pub const RESOLVER_KEY_BTC: &str = "token.btc";
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
pub const RESOLVER_KEY_TWITTER: &str = "com.twitter";
pub const RESOLVER_KEY_GITHUB: &str = "com.github";

pub const RESOLVER_VALUE_MAX_LENGTH: usize = 512;
pub const RESOLVER_KEY_MAX_LENGTH: usize = 64;
pub const RESOLVER_ITEM_MAX_COUNT: usize = 30;

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

impl ResolverKey {
    pub fn parse(s: &str) -> Option<ResolverKey> {
        match s {
            RESOLVER_KEY_ETH => Some(ResolverKey::Eth),
            RESOLVER_KEY_BTC => Some(ResolverKey::Btc),
            RESOLVER_KEY_ICP => Some(ResolverKey::Icp),
            RESOLVER_KEY_LTC => Some(ResolverKey::Ltc),
            RESOLVER_KEY_ICP_CANISTER => Some(ResolverKey::IcpCanister),
            RESOLVER_KEY_ICP_PRINCIPAL => Some(ResolverKey::IcpPrincipal),
            RESOLVER_KEY_ICP_ACCOUNT_ID => Some(ResolverKey::IcpAccountId),
            RESOLVER_KEY_EMAIL => Some(ResolverKey::Email),
            RESOLVER_KEY_URL => Some(ResolverKey::Url),
            RESOLVER_KEY_AVATAR => Some(ResolverKey::Avatar),
            RESOLVER_KEY_DESCRIPTION => Some(ResolverKey::Description),
            RESOLVER_KEY_NOTICE => Some(ResolverKey::Notice),
            RESOLVER_KEY_KEYWORDS => Some(ResolverKey::Keywords),
            RESOLVER_KEY_TWITTER => Some(ResolverKey::Twitter),
            RESOLVER_KEY_GITHUB => Some(ResolverKey::Github),
            _ => None,
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
