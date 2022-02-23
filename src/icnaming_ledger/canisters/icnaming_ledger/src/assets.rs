use crate::metrics_encoder::MetricsEncoder;
use crate::payments_store::encode_metrics;
use crate::state::STATE;
use crate::StableState;
use candid::{CandidType, Decode, Encode};
use dfn_core::api::ic0::time;
use serde::Deserialize;
use serde_bytes::ByteBuf;
use std::collections::HashMap;
use std::io::Read;

type HeaderField = (String, String);

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpRequest {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: ByteBuf,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpResponse {
    status_code: u16,
    headers: Vec<HeaderField>,
    body: ByteBuf,
}

pub fn http_request(req: HttpRequest) -> HttpResponse {
    let parts: Vec<&str> = req.url.split('?').collect();
    match parts[0] {
        "/metrics" => {
            let now;
            unsafe {
                now = time();
            };
            let mut writer = MetricsEncoder::new(vec![], now / 1_000_000);
            match encode_metrics(&mut writer) {
                Ok(()) => {
                    let body = writer.into_inner();
                    HttpResponse {
                        status_code: 200,
                        headers: vec![
                            (
                                "Content-Type".to_string(),
                                "text/plain; version=0.0.4".to_string(),
                            ),
                            ("Content-Length".to_string(), body.len().to_string()),
                        ],
                        body: ByteBuf::from(body),
                    }
                }
                Err(err) => HttpResponse {
                    status_code: 500,
                    headers: vec![],
                    body: ByteBuf::from(format!("Failed to encode metrics: {}", err)),
                },
            }
        }
        request_path => HttpResponse {
            status_code: 404,
            headers: vec![],
            body: ByteBuf::from(format!("Asset {} not found.", request_path)),
        },
    }
}

/// List of recommended security headers as per https://owasp.org/www-project-secure-headers/
/// These headers enable browser security features (like limit access to platform apis and set
/// iFrame policies, etc.).
/// TODO https://dfinity.atlassian.net/browse/L2-185: Add CSP and Permissions-Policy
fn security_headers() -> Vec<HeaderField> {
    vec![
        ("X-Frame-Options".to_string(), "DENY".to_string()),
        ("X-Content-Type-Options".to_string(), "nosniff".to_string()),
        (
            "Strict-Transport-Security".to_string(),
            "max-age=31536000 ; includeSubDomains".to_string(),
        ),
        // "Referrer-Policy: no-referrer" would be more strict, but breaks local dev deployment
        // same-origin is still ok from a security perspective
        ("Referrer-Policy".to_string(), "same-origin".to_string()),
    ]
}
