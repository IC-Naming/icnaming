use candid::{candid_method, Principal};
use ic_cdk::api;
use ic_cdk_macros::*;
use serde_bytes::ByteBuf;
use std::collections::{hash_map, HashMap};
use std::fmt::Debug;

use crate::registration_store::Registration;
use crate::{metadata, RegistrarService};
use common::http::{HeaderField, HttpRequest, HttpResponse};
use common::metrics_encoder::MetricsEncoder;
use common::nft::Metadata;

use crate::stats_service::encode_metrics;
use std::time::{Duration, SystemTime};
use time::format_description::well_known::{Iso8601, Rfc2822};
use time::OffsetDateTime;

#[query]
#[candid_method(query, rename = "http_request")]
fn http_request(req: HttpRequest) -> HttpResponse {
    let parts: Vec<&str> = req.url.split('?').collect();
    match parts[0] {
        "/metrics" => {
            let now;
            now = ic_cdk::api::time();
            let mut writer = MetricsEncoder::new(vec![], (now / 1_000_000) as i64);
            match encode_metrics(&mut writer, now) {
                Ok(()) => {
                    let body = writer.into_inner();
                    HttpResponse {
                        status_code: 200,
                        headers: vec![
                            HeaderField(
                                "Content-Type".to_string(),
                                "text/plain; version=0.0.4".to_string(),
                            ),
                            HeaderField("Content-Length".to_string(), body.len().to_string()),
                        ],
                        body: ByteBuf::from(body),
                        streaming_strategy: None,
                    }
                }
                Err(err) => HttpResponse {
                    status_code: 500,
                    headers: vec![],
                    body: ByteBuf::from(format!("Failed to encode metrics: {}", err)),
                    streaming_strategy: None,
                },
            }
        }
        request_path => HttpResponse {
            status_code: 404,
            headers: vec![],
            body: ByteBuf::from(format!("Asset {} not found.", request_path)),
            streaming_strategy: None,
        },
    }
}

#[test]
fn test_http_request() {
    let req = HttpRequest {
        method: "GET".to_string(),
        url: "https://n7ib3-4qaaa-aaaai-qagnq-cai.raw.ic0.app/metrics/nft?tokenid=1234&name=test"
            .to_string(),
        headers: vec![],
        body: ByteBuf::from(vec![]).into_vec(),
    };
    let res = http_request(req);
    let test_result = get_nft(&get_registration("1234".to_string()));
}

fn http_request2(req: HttpRequest) -> HttpResponse {
    let parts: Vec<&str> = req.url.split('?').collect();
    let _resource = parts[0];
    let mut params: hash_map::HashMap<String, String> = HashMap::new();

    if parts.len() == 1 {
        return http_response(404, vec![], ByteBuf::from(vec![]));
    }
    let parsed_params_str_array = parts[1].split('&');

    for p in parsed_params_str_array {
        let kv: Vec<&str> = p.split('=').collect();
        if kv.len() == 2 {
            params.insert(kv[0].trim().to_lowercase(), kv[1].trim().to_lowercase());
        } else {
            api::print(format!("kv is {:?}", kv));
        }
    }
    let token_id_key = "tokenid";
    let service = RegistrarService::default();
    if params.contains_key(token_id_key) {
        let token_id_res = params.get(token_id_key);
        match token_id_res {
            Some(token_id) => {
                let registration_result = service.get_registration_by_token_id(&token_id);
                return if registration_result.is_ok() {
                    let registration = registration_result.unwrap();
                    let nft = get_nft(&registration);
                    http_response(200, generate_svg_headers(), nft)
                } else {
                    http_response(
                        500,
                        vec![],
                        ByteBuf::from(format!("registration not found: {}", token_id)),
                    )
                };
            }
            _ => http_response(404, vec![], ByteBuf::from(vec![])),
        }
    } else {
        http_response(200, generate_svg_headers(), ByteBuf::from(vec![]))
    }
}

fn get_registration(token_id: String) -> Registration {
    //get SystemTime::now() to u64
    let now: u64 = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Registration::new(Principal::anonymous(), "icnaming".to_string(), now, now)
}

fn get_nft(registration: &Registration) -> ByteBuf {
    let file_path = "../../../asset/icnaming_nft.svg";
    //read as string
    let mut svg_content = std::fs::read_to_string(file_path).unwrap();
    // u64 to DateTime String
    let expired_at = registration.get_expired_at();
    let expired_at = OffsetDateTime::from_unix_timestamp_nanos(expired_at as i128).unwrap();
    let expired_at = expired_at.format(&Rfc2822).unwrap();
    let mut svg_content = svg_content.replace("{{expired_at}}", expired_at.as_str());
    let mut svg_content = svg_content.replace("{{name}}", registration.get_name().as_str());
    println!("{}", svg_content);
    ByteBuf::from(svg_content.as_bytes())
}

fn http_response(status_code: u16, headers: Vec<HeaderField>, body: ByteBuf) -> HttpResponse {
    HttpResponse {
        status_code: status_code,
        headers: headers,
        body: body,
        streaming_strategy: None,
    }
}

fn generate_svg_headers() -> Vec<HeaderField> {
    vec![
        HeaderField("Access-Control-Allow-Origin".into(), "*".into()),
        HeaderField("Cache-Control".into(), "public,max-age=2592000".into()),
        HeaderField("Content-Type".into(), "image/svg+xml".into()),
        HeaderField("Power-By".into(), "Deland Labs".into()),
    ]
}
