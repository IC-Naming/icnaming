// const default ttl
pub const DEFAULT_TTL: u64 = 60 * 10;
pub const TOP_LABEL: &str = "icp";

pub const PAGE_INPUT_MIN_LIMIT: usize = 1;
pub const PAGE_INPUT_MAX_LIMIT: usize = 100;
pub const PAGE_INPUT_MIN_OFFSET: usize = 0;
pub const PAGE_INPUT_MAX_OFFSET: usize = 10_000;

pub const DEFAULT_MIN_REGISTRATION_YEAR: u64 = 1;
pub const DEFAULT_MAX_REGISTRATION_YEAR: u64 = 100;

pub const CANISTER_NAME_REGISTRY: &str = "registry";
pub const CANISTER_NAME_REGISTRAR: &str = "registrar";
pub const CANISTER_NAME_RESOLVER: &str = "resolver";
pub const CANISTER_NAME_ENS_ACTIVITY_CLIENT: &str = "ens_activity_client";

pub const ALL_CANISTER_NAMES: [&str; 4] = [
    CANISTER_NAME_REGISTRY,
    CANISTER_NAME_REGISTRAR,
    CANISTER_NAME_RESOLVER,
    CANISTER_NAME_ENS_ACTIVITY_CLIENT,
];

// resolver keys
pub const RESOLVER_KEY_ETH: &str = "token.eth";
pub const RESOLVER_KEY_BTC: &str = "token.btc";
pub const RESOLVER_KEY_ICP: &str = "token.icp";
pub const RESOLVER_KEY_LTC: &str = "token.ltc";
pub const RESOLVER_KEY_ICP_CANISTER: &str = "canister.icp";
pub const RESOLVER_KEY_EMAIL: &str = "email";
pub const RESOLVER_KEY_URL: &str = "url";
pub const RESOLVER_KEY_AVATAR: &str = "avatar";
pub const RESOLVER_KEY_DESCRIPTION: &str = "description";
pub const RESOLVER_KEY_NOTICE: &str = "notice";
pub const RESOLVER_KEY_KEYWORDS: &str = "keywords";
pub const RESOLVER_KEY_TWITTER: &str = "com.twitter";
pub const RESOLVER_KEY_GITHUB: &str = "com.github";

pub const RESOLVER_VALUE_MAX_LENGTH: usize = 256;

// all keys
pub const ALL_RESOLVER_KEYS: [&str; 13] = [
    RESOLVER_KEY_ETH,
    RESOLVER_KEY_BTC,
    RESOLVER_KEY_ICP,
    RESOLVER_KEY_LTC,
    RESOLVER_KEY_ICP_CANISTER,
    RESOLVER_KEY_EMAIL,
    RESOLVER_KEY_URL,
    RESOLVER_KEY_AVATAR,
    RESOLVER_KEY_DESCRIPTION,
    RESOLVER_KEY_NOTICE,
    RESOLVER_KEY_KEYWORDS,
    RESOLVER_KEY_TWITTER,
    RESOLVER_KEY_GITHUB,
];

pub const MAX_COUNT_USER_FAVORITES: usize = 100;
pub const MAX_LENGTH_USER_FAVORITES: usize = 256;
