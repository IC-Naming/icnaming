use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(Serialize, Deserialize, CandidType, Clone, PartialEq, Eq, Debug)]
pub struct IcpXdrConversionRateCertifiedResponse {
    pub data: IcpXdrConversionRate,
    pub hash_tree: Vec<u8>,
    pub certificate: Vec<u8>,
}

#[derive(Serialize, Deserialize, CandidType, Clone, PartialEq, Eq, Debug, Default)]
pub struct IcpXdrConversionRate {
    /// The time for which the market data was queried, expressed in UNIX epoch
    /// time in seconds.
    pub timestamp_seconds: u64,
    /// The number of 10,000ths of IMF SDR (currency code XDR) that corresponds
    /// to 1 ICP. This value reflects the current market price of one ICP
    /// token. In other words, this value specifies the ICP/XDR conversion
    /// rate to four decimal places.
    pub xdr_permyriad_per_icp: u64,
}
