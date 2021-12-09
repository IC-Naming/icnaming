use candid::{candid_method, Principal};
use ic_cdk_macros::*;
use log::info;
use url::Url;

use common::http::{HttpRequest, HttpResponse};

use crate::service::RegistriesService;

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

    let service = RegistriesService::new();
    let name = name.unwrap();
    let result = service.get_resolver(&name);
    if result.is_err() {
        return HttpResponse::string(400, "resolver not found");
    }
    let resolver = result.unwrap();
    let url = create_redirect_url(name.as_str(), key.unwrap().as_str(), resolver);
    HttpResponse::redirect(url.as_str())
}

fn create_redirect_url(name: &str, key: &str, resolver: Principal) -> Url {
    let mut url = Url::parse("http://127.0.0.1:8000").unwrap();
    #[cfg(test)]
    {
        url = Url::parse("http://127.0.0.1:8000").unwrap();
    }
    url.query_pairs_mut()
        .append_pair("canisterId", resolver.to_text().as_str())
        .append_pair("name", name)
        .append_pair("key", key);
    url
}
