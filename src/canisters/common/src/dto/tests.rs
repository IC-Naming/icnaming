use rstest::*;

use crate::constants::{PAGE_INPUT_MAX_LIMIT, PAGE_INPUT_MAX_OFFSET};
use crate::dto::*;
use crate::errors::ICNSError;
use crate::test_common::test::init_test;

#[fixture]
pub fn setup() {
    init_test();
}

mod get_page_input {
    use super::*;

    #[rstest]
    fn test_get_page_input(_setup: ()) {
        let input = GetPageInput {
            limit: 10,
            offset: 0,
        };
        assert_eq!(input.validate(), Ok(()));
    }

    #[rstest]
    fn test_get_page_input_limit_overflow(_setup: ()) {
        let input = GetPageInput {
            limit: 100_000,
            offset: 0,
        };
        assert_eq!(
            input.validate(),
            Err(ICNSError::ValueShouldBeInRangeError {
                field: "limit".to_string(),
                min: 1,
                max: PAGE_INPUT_MAX_LIMIT,
            })
        );
    }

    #[rstest]
    fn test_get_page_input_offset_overflow(_setup: ()) {
        let input = GetPageInput {
            limit: 100,
            offset: 100_000,
        };
        assert_eq!(
            input.validate(),
            Err(ICNSError::ValueShouldBeInRangeError {
                field: "offset".to_string(),
                min: 0,
                max: PAGE_INPUT_MAX_OFFSET,
            })
        );
    }
}

mod owner_can_operate {
    use super::*;

    #[rstest]
    fn test_owner_can_operate(_setup: ()) {
        let owner1 = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        let operator = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

        let mut operators = HashSet::new();
        operators.insert(operator);
        let users = RegistryUsers {
            owner: owner1,
            operators,
        };

        assert_eq!(users.can_operate(&Principal::anonymous()), false);
        assert_eq!(users.can_operate(&owner1), true);
        assert_eq!(users.can_operate(&operator), true);

        assert_eq!(users.is_owner(&owner1), true);
    }
}
