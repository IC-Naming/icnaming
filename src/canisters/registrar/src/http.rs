use candid::{candid_method, Principal};

use ic_cdk_macros::*;
use serde_bytes::ByteBuf;

use crate::registration_store::Registration;

use common::http::{HeaderField, HttpRequest, HttpResponse};
use common::metrics_encoder::MetricsEncoder;

use crate::http_nft::get_nft_http_response;
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
        "/" => get_nft_http_response(&parts[1]),
        request_path => HttpResponse {
            status_code: 404,
            headers: vec![],
            body: ByteBuf::from(format!("Asset {} not found.", request_path)),
            streaming_strategy: None,
        },
    }
}

mod test_http_request {
    use super::*;
    use crate::state::STATE;

    use common::named_canister_ids::{get_named_get_canister_id, CanisterNames};
    use common::token_identifier::{encode_token_id, CanisterId, TokenIndex};
    use rstest::*;
    use test_common::create_test_name;
    use test_common::user::mock_tomorrow;
    use test_common::user::mock_user1;

    fn registration_name_init(name: &String, user: Principal, now: u64) {
        STATE.with(|s| {
            let mut store = s.token_index_store.borrow_mut();
            store.try_add_registration_name(name);
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                user.clone(),
                name.to_string(),
                now + 1,
                now,
            ));
        });
    }

    #[rstest]
    fn test_ntf_routing(mock_user1: Principal, mock_tomorrow: u64) {
        let test_icnaming_str = create_test_name("icnaming");
        let test_hello_str = create_test_name("hello");
        registration_name_init(&test_icnaming_str.to_string(), mock_user1, mock_tomorrow);
        registration_name_init(&test_hello_str.to_string(), mock_user1, mock_tomorrow);
        let canisterid = get_named_get_canister_id(CanisterNames::Registrar);
        let icnaming_token_id = encode_token_id(CanisterId(canisterid), TokenIndex(1u32));
        let hello_token_id = encode_token_id(CanisterId(canisterid), TokenIndex(2u32));

        let icnaming_request = HttpRequest {
            url: format!("/?tokenid={}", icnaming_token_id),
            method: "GET".to_string(),
            headers: vec![],
            body: vec![],
        };
        let hello_request = HttpRequest {
            url: format!("/?tokenid={}", hello_token_id),
            method: "GET".to_string(),
            headers: vec![],
            body: vec![],
        };
        let response = http_request(icnaming_request);
        assert_eq!(response.status_code, 200);
        assert!(String::from_utf8(response.body.to_vec())
            .unwrap()
            .contains("icnaming"));

        let response = http_request(hello_request);
        assert_eq!(response.status_code, 200);
        assert!(String::from_utf8(response.body.to_vec())
            .unwrap()
            .contains("hello"));
    }
}
