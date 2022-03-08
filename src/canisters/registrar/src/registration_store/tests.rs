use log::info;
use rstest::*;

use common::named_canister_ids::{get_named_get_canister_id, CANISTER_NAME_RESOLVER};
use test_common::ic_api::init_test;
use test_common::user::*;

use super::*;

#[fixture]
fn owner() -> Principal {
    mock_user1()
}

#[fixture]
fn quota_owner() -> Principal {
    mock_user2()
}

#[fixture]
fn default_resolver() -> Principal {
    get_named_get_canister_id(CANISTER_NAME_RESOLVER)
}

#[fixture]
fn store(_init_test: ()) -> RegistrationStore {
    let service = RegistrationStore::default();
    service
}

mod memory {
    use super::*;

    #[rstest]
    fn test_register_many(_init_test: ()) {
        let counts = vec![1u32, 10, 100, 1_000];

        // run each count and record the size of the store
        let mut sizes = vec![];
        for count in counts.iter() {
            let mut store = RegistrationStore::default();

            for index in 0..*count {
                let owner = mock_user(index);
                let name = format!("test-name-{:06}.icp", index);
                store.add_registration(Registration {
                    owner,
                    name,
                    expired_at: u64::MAX - 1,
                    created_at: u64::MAX - 1,
                });
            }

            let encode_state = store.encode();
            sizes.push(encode_state.len());
        }

        info!("add registrations done, print sizes");
        // info the sizes with count
        for (count, size) in counts.iter().zip(sizes.iter()) {
            info!(
                "{} registrations: {} bytes, average {} bytes each registration",
                count,
                size,
                size / *count as usize
            );
        }

        // please update comment below if you change the size of the store
        // 1 registrations: 131 bytes, average 131 bytes each registration
        // 10 registrations: 932 bytes, average 93 bytes each registration
        // 100 registrations: 8942 bytes, average 89 bytes each registration
        // 1000 registrations: 89043 bytes, average 89 bytes each registration
        // 10000 registrations: 890043 bytes, average 89 bytes each registration
        // 50000 registrations: 4450044 bytes, average 89 bytes each registration
        // 100000 registrations: 8900044 bytes, average 89 bytes each registration
    }
}
