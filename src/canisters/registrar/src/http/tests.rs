mod test_http_request {

    use crate::state::STATE;
    use candid::Principal;

    use crate::registration_store::Registration;

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
}
