use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::str::FromStr;

use candid::{decode_args, encode_args, CandidType, Deserialize, Nat, Principal};
use rstest::*;

#[derive(CandidType, Deserialize, Eq, PartialEq, Debug)]
enum TestOrderStatus {
    New,
    InProgress,
    Done,
    Canceled(String),
}

#[derive(CandidType, Deserialize, Eq, PartialEq, Debug)]
struct TestSettingsV1 {
    limit: u8,
    create_at: u64,
}

#[derive(CandidType, Deserialize, Eq, PartialEq, Debug)]
struct TestSettingsV2RemovingField {
    create_at: u64,
}

#[derive(CandidType, Deserialize, Eq, PartialEq, Debug)]
struct TestSettingsV2RequiredEndAt {
    limit: u8,
    create_at: u64,
    end_at: u64,
}

#[derive(CandidType, Deserialize, Eq, PartialEq, Debug)]
struct TestSettingsV2OptionalEndAt {
    end_at: Option<u64>,
    create_at: u64,
    limit: u8,
}

#[derive(CandidType, Deserialize, Eq, PartialEq, Debug)]
struct TestSettingsV2NewSet {
    limit: u8,
    create_at: u64,
    users: HashSet<Principal>,
}

#[derive(CandidType, Deserialize, Eq, PartialEq, Debug)]
struct TestSettingsV2OptionalNewSet {
    limit: u8,
    create_at: u64,
    users: Option<HashSet<Principal>>,
}

#[derive(CandidType, Deserialize, Eq, PartialEq, Debug)]
struct UserInfo {
    name: String,
    age: u8,
}

#[derive(CandidType, Deserialize, Eq, PartialEq, Debug)]
struct UserInfoV2RequiredField {
    name: String,
    age: u8,
    email: String,
}

#[derive(CandidType, Deserialize, Eq, PartialEq, Debug)]
struct TestSettingsV2OptionalField {
    name: String,
    age: u8,
    email: Option<String>,
}

struct StateV1 {
    int_value: u32,
    string_value: String,
    nat_value: Nat,
    order_status: TestOrderStatus,
    user_count_map: HashMap<Principal, HashMap<String, u64>>,
    user_info_map: HashMap<Principal, UserInfo>,
    settings: TestSettingsV1,
}

#[fixture]
fn state_v1() -> StateV1 {
    let mut map = HashMap::new();
    map.insert(
        Principal::from_str("zo36k-iqaaa-aaaaj-qahdq-cai").unwrap(),
        HashMap::from_iter(vec![("test".to_string(), 1)]),
    );
    let mut user_info_map = HashMap::new();
    user_info_map.insert(
        Principal::from_str("zo36k-iqaaa-aaaaj-qahdq-cai").unwrap(),
        UserInfo {
            name: "test".to_string(),
            age: 122,
        },
    );
    StateV1 {
        int_value: 123,
        string_value: "test".to_string(),
        nat_value: Nat::from(4567),
        order_status: TestOrderStatus::Canceled("canceled".to_string()),
        user_count_map: map,
        user_info_map,
        settings: TestSettingsV1 {
            limit: 12,
            create_at: 3456,
        },
    }
}

#[fixture]
fn encoded_state_v1(state_v1: StateV1) -> Vec<u8> {
    let encoded_state = encode_args((
        state_v1.int_value,
        state_v1.string_value,
        state_v1.nat_value,
        state_v1.order_status,
        state_v1.user_count_map,
        state_v1.user_info_map,
        state_v1.settings,
    ));
    encoded_state.unwrap()
}

#[rstest]
fn test_encode_decode(encoded_state_v1: Vec<u8>) {
    let (ini_value, string_value, nat_value, order_status, user_count_map,user_info_map, settings): (
        u32,
        String,
        Nat,
        TestOrderStatus,
        HashMap<Principal, HashMap<String, u64>>,
        HashMap<Principal, UserInfo>,
        TestSettingsV1,
    ) = decode_args(&encoded_state_v1).unwrap();

    assert_eq!(ini_value, 123);
    assert_eq!(string_value, "test");
    assert_eq!(nat_value, Nat::from(4567));
    assert_eq!(
        order_status,
        TestOrderStatus::Canceled("canceled".to_string())
    );
    assert_eq!(
        user_count_map,
        HashMap::from_iter(vec![(
            Principal::from_str("zo36k-iqaaa-aaaaj-qahdq-cai").unwrap(),
            HashMap::from_iter(vec![("test".to_string(), 1)])
        )])
    );
    assert_eq!(
        user_info_map,
        HashMap::from_iter(vec![(
            Principal::from_str("zo36k-iqaaa-aaaaj-qahdq-cai").unwrap(),
            UserInfo {
                name: "test".to_string(),
                age: 122,
            }
        )])
    );
    assert_eq!(settings.limit, 12);
    assert_eq!(settings.create_at, 3456);
}

#[rstest]
fn test_decode_ignore_some_fields(encoded_state_v1: Vec<u8>) {
    {
        let (ini_value, string_value, _, order_status, _, _, settings): (
            u32,
            String,
            Nat,
            TestOrderStatus,
            HashMap<Principal, HashMap<String, u64>>,
            HashMap<Principal, UserInfo>,
            TestSettingsV1,
        ) = decode_args(&encoded_state_v1).unwrap();

        assert_eq!(ini_value, 123);
        assert_eq!(string_value, "test");
        assert_eq!(
            order_status,
            TestOrderStatus::Canceled("canceled".to_string())
        );
        assert_eq!(settings.limit, 12);
        assert_eq!(settings.create_at, 3456);
    }
    {
        let (ini_value, string_value): (u32, String) = decode_args(&encoded_state_v1).unwrap();

        assert_eq!(ini_value, 123);
        assert_eq!(string_value, "test");
    }
}

#[rstest]
#[should_panic]
fn test_throw_error_if_decode_in_wrong_order(encoded_state_v1: Vec<u8>) {
    let (string_value, ini_value): (String, u32) = decode_args(&encoded_state_v1).unwrap();

    assert_eq!(string_value, "test");
    assert_eq!(ini_value, 123);
}

#[rstest]
#[should_panic]
fn test_decode_failed_with_additional_required_field(encoded_state_v1: Vec<u8>) {
    let (_, _, _, _, _, _, settings): (
        u32,
        String,
        Nat,
        TestOrderStatus,
        HashMap<Principal, HashMap<String, u64>>,
        HashMap<Principal, UserInfo>,
        TestSettingsV2RequiredEndAt,
    ) = decode_args(&encoded_state_v1).unwrap();

    assert_eq!(settings.limit, 12);
    assert_eq!(settings.create_at, 3456);
}

#[rstest]
fn test_decode_ok_with_additional_optional_field(encoded_state_v1: Vec<u8>) {
    let (_, _, _, _, _, _, settings): (
        u32,
        String,
        Nat,
        TestOrderStatus,
        HashMap<Principal, HashMap<String, u64>>,
        HashMap<Principal, UserInfo>,
        TestSettingsV2OptionalEndAt,
    ) = decode_args(&encoded_state_v1).unwrap();

    assert_eq!(settings.limit, 12);
    assert_eq!(settings.create_at, 3456);
    assert_eq!(settings.end_at, None);
}

#[rstest]
fn test_decode_ok_with_removing_field(encoded_state_v1: Vec<u8>) {
    let (_, _, _, _, _, _, settings): (
        u32,
        String,
        Nat,
        TestOrderStatus,
        HashMap<Principal, HashMap<String, u64>>,
        HashMap<Principal, UserInfo>,
        TestSettingsV2RemovingField,
    ) = decode_args(&encoded_state_v1).unwrap();

    assert_eq!(settings.create_at, 3456);
}

#[rstest]
fn test_decode_ok_with_additional_required_set(encoded_state_v1: Vec<u8>) {
    let (_, _, _, _, _, _, settings): (
        u32,
        String,
        Nat,
        TestOrderStatus,
        HashMap<Principal, HashMap<String, u64>>,
        HashMap<Principal, UserInfo>,
        TestSettingsV2OptionalNewSet,
    ) = decode_args(&encoded_state_v1).unwrap();

    assert_eq!(settings.limit, 12);
    assert_eq!(settings.create_at, 3456);
    assert_eq!(settings.users, None);
}

#[rstest]
fn test_decode_ok_with_additional_map_optional_field(encoded_state_v1: Vec<u8>) {
    let (_, _, _, _, _, user_info_map, _): (
        u32,
        String,
        Nat,
        TestOrderStatus,
        HashMap<Principal, HashMap<String, u64>>,
        HashMap<Principal, TestSettingsV2OptionalField>,
        TestSettingsV2OptionalEndAt,
    ) = decode_args(&encoded_state_v1).unwrap();

    assert_eq!(
        user_info_map,
        HashMap::from_iter(vec![(
            Principal::from_str("zo36k-iqaaa-aaaaj-qahdq-cai").unwrap(),
            TestSettingsV2OptionalField {
                name: "test".to_string(),
                age: 122,
                email: None
            }
        )])
    );
}

#[rstest]
#[should_panic]
fn test_decode_ok_with_additional_map_required_set(encoded_state_v1: Vec<u8>) {
    let (_, _, _, _, _, user_info_map, _): (
        u32,
        String,
        Nat,
        TestOrderStatus,
        HashMap<Principal, HashMap<String, u64>>,
        HashMap<Principal, UserInfoV2RequiredField>,
        TestSettingsV2OptionalNewSet,
    ) = decode_args(&encoded_state_v1).unwrap();

    assert_eq!(
        user_info_map,
        HashMap::from_iter(vec![(
            Principal::from_str("zo36k-iqaaa-aaaaj-qahdq-cai").unwrap(),
            UserInfoV2RequiredField {
                name: "test".to_string(),
                age: 122,
                email: "".to_string()
            }
        )])
    );
}

#[rstest]
fn test_encode_decode_option(encoded_state_v1: Vec<u8>) {
    let (
        ini_value,
        string_value,
        nat_value,
        order_status,
        user_count_map,
        user_info_map,
        settings,
        something_option,
    ): (
        u32,
        String,
        Nat,
        TestOrderStatus,
        HashMap<Principal, HashMap<String, u64>>,
        HashMap<Principal, UserInfo>,
        TestSettingsV1,
        Option<TestSettingsV1>,
    ) = decode_args(&encoded_state_v1).unwrap();

    assert_eq!(ini_value, 123);
    assert_eq!(string_value, "test");
    assert_eq!(nat_value, Nat::from(4567));
    assert_eq!(
        order_status,
        TestOrderStatus::Canceled("canceled".to_string())
    );
    assert_eq!(
        user_count_map,
        HashMap::from_iter(vec![(
            Principal::from_str("zo36k-iqaaa-aaaaj-qahdq-cai").unwrap(),
            HashMap::from_iter(vec![("test".to_string(), 1)])
        )])
    );
    assert_eq!(
        user_info_map,
        HashMap::from_iter(vec![(
            Principal::from_str("zo36k-iqaaa-aaaaj-qahdq-cai").unwrap(),
            UserInfo {
                name: "test".to_string(),
                age: 122,
            }
        )])
    );
    assert_eq!(settings.limit, 12);
    assert_eq!(settings.create_at, 3456);
    assert_eq!(something_option.is_none(), true);
}
