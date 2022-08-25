use crate::registration_store::Registration;
use crate::{Principal, RegistrarService};

use common::http::{HeaderField, HttpResponse};
use ic_cdk::api;
use rstest::*;
use serde_bytes::ByteBuf;
use std::collections::HashMap;

use time::OffsetDateTime;

fn time_format(time: u64) -> String {
    let time = OffsetDateTime::from_unix_timestamp_nanos(time as i128).unwrap();
    format!(
        "{}:{} UTC {}/{} {}",
        time.time().hour(),
        time.time().minute(),
        time.date().month() as u32,
        time.date().day(),
        time.year()
    )
}

pub fn get_nft_http_response(param: &str) -> HttpResponse {
    let parsed_params_str_array = param.split('&');
    let mut params: HashMap<String, String> = HashMap::new();
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
                    let nft_svg_bytes = get_nft_svg_bytes(&registration);
                    http_response(200, generate_svg_headers(), nft_svg_bytes)
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

fn get_nft_svg_bytes(registration: &Registration) -> ByteBuf {
    let svg_content = include_str!("../../../../asset/icnaming_nft.svg").clone();
    let expired_at = registration.get_expired_at();
    let expired_at = time_format(expired_at);
    let svg_content = svg_content.replace("{{expired_at}}", expired_at.as_str());
    let svg_content = svg_content.replace("{{name}}", registration.get_name().as_str());
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
        HeaderField("Power-By".into(), "ICNaming".into()),
    ]
}

mod test_http_request {
    use super::*;
    use crate::state::STATE;

    use common::named_canister_ids::{get_named_get_canister_id, CanisterNames};
    use common::token_identifier::{encode_token_id, CanisterId, TokenIndex};

    use std::string::String;
    use test_common::create_test_name;

    use test_common::user::{mock_tomorrow, mock_user1};

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
    fn test_time_format() {
        let now = 1598264204000000000;
        let time_str = time_format(now);
        assert_eq!(time_str, "10:16 UTC 8/24 2020");
    }

    #[rstest]
    fn test_http_request(mock_user1: Principal, mock_tomorrow: u64) {
        let test_name_str = create_test_name("icnaming");
        let time_str = time_format(mock_tomorrow);
        registration_name_init(&test_name_str.to_string(), mock_user1, mock_tomorrow);
        let canisterid = get_named_get_canister_id(CanisterNames::Registrar);
        let token_id = encode_token_id(CanisterId(canisterid), TokenIndex(1u32));
        let param_str = format!("tokenid={}", token_id);
        let res = get_nft_http_response(param_str.as_str());
        let str = String::from_utf8(res.body.to_vec()).unwrap();
        assert!(str.contains(&time_str));
        assert!(str.contains(&test_name_str));
    }
}
