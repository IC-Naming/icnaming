use rstest::*;

mod test_http_request {
    use super::*;
    use crate::state::STATE;

    use common::named_canister_ids::{get_named_get_canister_id, CanisterNames};
    use common::token_identifier::{encode_token_id, CanisterId, TokenIndex};

    use candid::Principal;
    use std::string::String;
    use test_common::create_test_name;

    use crate::http_nft::{get_nft_http_response, time_format};
    use crate::registration_store::Registration;
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

    #[rstest]
    fn test_time_format() {
        let now = 1598264204000000000;
        let time_str = time_format(now);
        assert_eq!(time_str, "10:16 UTC 8/24 2020");
    }

    #[rstest]
    fn test_http_request(
        mock_user1: Principal,
        mock_std_time_tomorrow: u64,
        mock_std_time_now: u64,
    ) {
        let test_name_str = create_test_name("icnaming");
        let time_str = time_format(mock_std_time_tomorrow);
        registration_name_init(
            &test_name_str.to_string(),
            mock_user1,
            mock_std_time_tomorrow,
        );
        let canisterid = get_named_get_canister_id(CanisterNames::Registrar);
        let token_id = encode_token_id(CanisterId(canisterid), TokenIndex(1u32));
        let param_str = format!("tokenid={}", token_id);
        let res = get_nft_http_response(param_str.as_str(), mock_std_time_now);
        let str = String::from_utf8(res.body.to_vec()).unwrap();
        assert!(str.contains(&time_str));
        assert!(str.contains(&test_name_str));
    }
}
