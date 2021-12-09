use candid::candid_method;
use ic_cdk_macros::*;
use log::info;

use common::http::{HttpRequest, HttpResponse};

use crate::service::ResolverService;

#[query]
#[candid_method(query, rename = "http_request")]
fn http_request(req: HttpRequest) -> HttpResponse {
    info!("request: {:?}", req);
    let name = req.get_query_value("name");
    let key = req.get_query_value("key");

    if name.is_none() {
        return HttpResponse::string(400, "name is required");
    }
    if key.is_none() {
        return HttpResponse::string(400, "key is required");
    }
    let service = ResolverService::new();
    let result = service.get_record_value(name.unwrap().as_str());
    match result {
        Ok(value) => value
            .get(key.unwrap().as_str())
            .map(|v| HttpResponse::string(200, v.to_string().as_str()))
            .unwrap_or_else(|| HttpResponse::string(404, "not found")),
        Err(e) => HttpResponse::string(400, format!("{:?}", e).as_str()),
    }
}
