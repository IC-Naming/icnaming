use rstest::*;

use test_common::ic_api::init_test;
use test_common::user::*;

use super::*;

mod memory {
    use log::info;
    use num_bigint::BigUint;

    use crate::quota_order_store::ICPMemo;

    use super::*;

    #[rstest]
    fn test_name_order_many(_init_test: ()) {
        let counts = vec![1u32, 10, 100, 1_000];

        // run each count and record the size of the store
        let mut sizes = vec![];
        for count in counts.iter() {
            let mut store = NameOrderStore::default();

            for index in 0..*count {
                let created_user = mock_user(index);
                let name = format!("test-name-{:06}.icp", index);
                let years = index as u32;
                let payment_memo = PaymentMemo::ICP(ICPMemo(index as u64));
                let price_icp_in_e8s = Nat(BigUint::from(index as u64 * 100_000_000u64));
                let payment_id = index as u64;
                let payment_account_id = mock_account_id(index, 0);
                let quota_type = QuotaType::LenGte(7);
                store.add_name_order(
                    &created_user,
                    &name,
                    years,
                    &payment_memo,
                    price_icp_in_e8s,
                    &payment_id,
                    &payment_account_id,
                    &quota_type,
                );
            }

            let encode_state = store.encode();
            sizes.push(encode_state.len());
        }

        info!("add name orders done, print sizes");
        // info the sizes with count
        for (count, size) in counts.iter().zip(sizes.iter()) {
            info!(
                "{} name orders: {} bytes, average {} bytes each name order",
                count,
                size,
                size / *count as usize
            );
        }

        // add name orders done, print sizes
        // 1 name orders: 339 bytes, average 339 bytes each name order
        // 10 name orders: 2281 bytes, average 228 bytes each name order
        // 100 name orders: 21721 bytes, average 217 bytes each name order
        // 1000 name orders: 216779 bytes, average 216 bytes each name order
        // 10000 name orders: 2169779 bytes, average 216 bytes each name order
        // 50000 name orders: 10855800 bytes, average 217 bytes each name order
        // 100000 name orders: 21755800 bytes, average 217 bytes each name order
    }
}
