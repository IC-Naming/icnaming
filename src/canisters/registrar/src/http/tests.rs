use rstest::*;
mod test_http_request {
    use super::*;
    use crate::state::STATE;
    use candid::Principal;

    use crate::http::http_request;
    use crate::registration_store::Registration;
    use common::http::HttpRequest;
    use common::named_canister_ids::{get_named_get_canister_id, CanisterNames};
    use common::token_identifier::{encode_token_id, CanisterId, TokenIndex};

    use test_common::create_test_name;
    use test_common::user::{mock_std_time_now, mock_std_time_tomorrow, mock_user1};

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

    // #[rstest]
    // fn test_ntf_routing(mock_user1: Principal, mock_std_time_tomorrow: u64, mock_std_time_now: u64) {
    //     let test_icnaming_str = create_test_name("icnaming");
    //     let test_hello_str = create_test_name("hello");
    //     registration_name_init(&test_icnaming_str.to_string(), mock_user1, mock_std_time_tomorrow);
    //     registration_name_init(&test_hello_str.to_string(), mock_user1, mock_std_time_tomorrow);
    //     let canisterid = get_named_get_canister_id(CanisterNames::Registrar);
    //     let icnaming_token_id = encode_token_id(CanisterId(canisterid), TokenIndex(1u32));
    //     let hello_token_id = encode_token_id(CanisterId(canisterid), TokenIndex(2u32));
    //
    //     let icnaming_request = HttpRequest {
    //         url: format!("/?tokenid={}", icnaming_token_id),
    //         method: "GET".to_string(),
    //         headers: vec![],
    //         body: vec![],
    //     };
    //     let hello_request = HttpRequest {
    //         url: format!("/?tokenid={}", hello_token_id),
    //         method: "GET".to_string(),
    //         headers: vec![],
    //         body: vec![],
    //     };
    //     let response = http_request(icnaming_request);
    //     assert_eq!(response.status_code, 200);
    //     assert!(String::from_utf8(response.body.to_vec())
    //         .unwrap()
    //         .contains("icnaming"));
    //
    //     let response = http_request(hello_request);
    //     assert_eq!(response.status_code, 200);
    //     assert!(String::from_utf8(response.body.to_vec())
    //         .unwrap()
    //         .contains("hello"));
    // }
}
