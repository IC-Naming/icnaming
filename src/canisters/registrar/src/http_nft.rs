use crate::{RegistrarService, RegistrationNameQueryContext};

use common::http::{HeaderField, HttpResponse};
use ic_cdk::api;
use serde_bytes::ByteBuf;
use std::collections::HashMap;

use crate::state::STATE;
use crate::token_index_store::UnexpiredRegistrationAggDto;
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

pub fn get_nft_http_response(param: &str, now: u64) -> HttpResponse {
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
    let _service = RegistrarService::default();
    if params.contains_key(token_id_key) {
        let token_id_res = params.get(token_id_key);
        match token_id_res {
            Some(token_id) => STATE.with(|s| {
                let token_index_store = s.token_index_store.borrow();
                let registration_store = s.registration_store.borrow();
                let query =
                    RegistrationNameQueryContext::new(&token_index_store, &registration_store);
                let registration_result = query.get_unexpired_registration(&token_id, now);
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
            }),
            _ => http_response(404, vec![], ByteBuf::from(vec![])),
        }
    } else {
        http_response(200, generate_svg_headers(), ByteBuf::from(vec![]))
    }
}

fn get_nft_svg_bytes(registration: &UnexpiredRegistrationAggDto) -> ByteBuf {
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

#[cfg(test)]
mod tests;
