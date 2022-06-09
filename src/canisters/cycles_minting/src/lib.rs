use candid::candid_method;
use ic_cdk::api;
use ic_cdk_macros::*;

use common::cycles_minting_types::{IcpXdrConversionRate, IcpXdrConversionRateCertifiedResponse};

/// Check if name is available.
/// Returns true if name is available.
///
/// * `name` - name to check, e.g. "hello.ic"
#[query(name = "get_icp_xdr_conversion_rate")]
#[candid_method(query, rename = "get_icp_xdr_conversion_rate")]
pub fn get_icp_xdr_conversion_rate() -> IcpXdrConversionRateCertifiedResponse {
    IcpXdrConversionRateCertifiedResponse {
        certificate: Vec::new(),
        hash_tree: Vec::new(),
        data: IcpXdrConversionRate {
            timestamp_seconds: api::time() / 1000_000_000,
            xdr_permyriad_per_icp: 20000,
        },
    }
}
