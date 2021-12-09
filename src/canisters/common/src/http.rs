use std::fmt::Debug;

/**
copy from https://github.com/dfinity/agent-rs/blob/main/ic-utils/src/interfaces/http_request.rs
1. change HttpRequest.body to Vec<u8>,
2. remove HttpRequestCanister
3. add some new methods
 */
use candid::{CandidType, Deserialize, Func, Nat};
use serde_bytes::ByteBuf;
use url::Url;

#[cfg(test)]
mod tests;

#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct HeaderField(pub String, pub String);

#[derive(CandidType, Deserialize, Debug)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderField>,
    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
}

#[derive(CandidType, Deserialize)]
pub struct Token {
    key: String,
    content_encoding: String,
    index: Nat,
    // The sha ensures that a client doesn't stream part of one version of an asset
    // followed by part of a different asset, even if not checking the certificate.
    sha256: Option<ByteBuf>,
}

#[derive(CandidType, Deserialize)]
pub struct CallbackStrategy {
    pub callback: Func,
    pub token: Token,
}

#[derive(CandidType, Deserialize)]
pub enum StreamingStrategy {
    Callback(CallbackStrategy),
}

#[derive(CandidType, Deserialize)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
    pub streaming_strategy: Option<StreamingStrategy>,
}

#[derive(CandidType, Deserialize)]
pub struct StreamingCallbackHttpResponse {
    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
    pub token: Option<Token>,
}

impl HttpResponse {
    pub fn new(status_code: u16, body: Vec<u8>) -> HttpResponse {
        HttpResponse {
            status_code,
            headers: Vec::new(),
            body,
            streaming_strategy: None,
        }
    }
    pub fn string(status_code: u16, body: &str) -> HttpResponse {
        HttpResponse::new(status_code, body.as_bytes().to_vec())
    }
    pub fn redirect(url: &str) -> HttpResponse {
        HttpResponse {
            status_code: 302,
            headers: vec![HeaderField("Location".to_string(), url.to_string())],
            body: Vec::new(),
            streaming_strategy: None,
        }
    }
}

impl HttpRequest {
    pub fn get_query_value(&self, name: &str) -> Option<String> {
        let url = self.get_url();
        get_query_value(&url, name)
    }

    pub fn get_query_values(&self, name: &str) -> Vec<String> {
        let url = self.get_url();
        get_query_values(&url, name)
    }

    pub fn get_url(&self) -> Url {
        Url::parse("http://localhost")
            .unwrap()
            .join(self.url.as_str())
            .unwrap()
    }
}

pub fn get_query_value(url: &Url, name: &str) -> Option<String> {
    url.query_pairs()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.to_string())
}

pub fn get_query_values(url: &Url, name: &str) -> Vec<String> {
    url.query_pairs()
        .filter_map(|(k, v)| if k == name { Some(v.to_string()) } else { None })
        .collect()
}
