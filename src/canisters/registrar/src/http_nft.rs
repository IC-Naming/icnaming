use crate::registration_store::Registration;
use crate::{Principal, RegistrarService};
use common::http::{HeaderField, HttpRequest, HttpResponse};
use ic_cdk::api;
use rstest::*;
use serde_bytes::ByteBuf;
use std::collections::{hash_map, HashMap};
use std::time::SystemTime;
use time::format_description::well_known::Rfc2822;
use time::OffsetDateTime;

mod test_http_request {
    use super::*;
    use crate::state::STATE;
    use crate::token_index_store::RegistrationName;
    use common::named_canister_ids::{get_named_get_canister_id, CanisterNames};
    use common::token_identifier::{encode_token_id, CanisterId, TokenIndex};
    use std::str::from_utf8;
    use std::string::String;
    use test_common::create_test_name;
    use test_common::user::mock_now;
    use test_common::user::mock_user1;

    fn registration_init(name: String, user: Principal, now: u64) {
        STATE.with(|s| {
            let mut store = s.token_index_store.borrow_mut();
            store.try_add_registration_name(RegistrationName(name.to_string()));
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
    fn test_http_request(mock_user1: Principal, mock_now: u64) {
        let test_name_str = create_test_name("icnaming");
        registration_init(test_name_str.to_string(), mock_user1, mock_now);
        let req = HttpRequest {
            method: "GET".to_string(),
            url: "https://n7ib3-4qaaa-aaaai-qagnq-cai.raw.ic0.app/nft?tokenid=1".to_string(),
            headers: vec![],
            body: ByteBuf::from(vec![]).into_vec(),
        };
        let canisterid = get_named_get_canister_id(CanisterNames::Registrar);
        let token_id = encode_token_id(CanisterId(canisterid), TokenIndex(1u32));
        let param_str = format!("tokenid={}", token_id);
        let res = get_nft_http_response(param_str.as_str());
        //let test_result = get_nft(&get_registration("1234".to_string()));
        println!("{:?}", String::from_utf8(res.body.to_vec()).unwrap());
    }
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
                println!("{:?}", registration_result);
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

fn http_request2(req: HttpRequest) -> HttpResponse {
    let parts: Vec<&str> = req.url.split('?').collect();
    let _resource = parts[0];
    let mut params: HashMap<String, String> = HashMap::new();

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
    let mut svg_content = include_str!("../../../../asset/icnaming_nft.svg").clone();
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
