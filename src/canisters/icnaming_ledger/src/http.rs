use candid::candid_method;
use ic_cdk_macros::*;
use serde_bytes::ByteBuf;

use common::http::{HeaderField, HttpRequest, HttpResponse};
use common::metrics_encoder::MetricsEncoder;

use crate::stats_service::encode_metrics;

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
