@registrar
Feature: Register a name with quota

  Background:
    Given Reinstall registrar related canisters
    And Some users already have some quotas
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 10    |
      | user1 | LenGte      | 4           | 6     |
      | user1 | LenEq       | 5           | 10    |
      | user2 | LenGte      | 3           | 10    |

  Scenario: Register a name with quota
    When User "user1" register name "hello.icp" with quote "LenGte(3)"
    Then get_details "hello.icp" result is
      | key        | value     |
      | owner      | user1     |
      | name       | hello.icp |
      | expired_at | 1         |
      | created_at | 0         |
    And  User quota status should be as below
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 9     |


  Scenario: Register a name with not enough quota
    When User "user1" register name "hello1.icp" with quote "LenEq(6)"
    Then Register with quota result in status 'name is invalid, reason: "User has no quota for len_eq(6)"'

  Scenario: Register a name for other user
    When User "user1" register name "hello.icp" with quote "LenGte(4)" for "user2" with "3" years
    Then get_details "hello.icp" result is
      | key        | value     |
      | owner      | user2     |
      | name       | hello.icp |
      | expired_at | 3         |
      | created_at | 0         |

    And  User quota status should be as below
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 10    |
      | user1 | LenGte      | 4           | 3     |

  Scenario: Register a name with quota but registry canister down
    Given canister "registry" is down
    And User "user1" register name "hello2.icp" with quote "LenGte(3)"
    When Register with quota result in status 'error from remote, ErrorInfo { code: 26, message: "canister call error, rejected by \"DestinationInvalid\"" }'
    And  User quota status should be as below
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 10    |
      | user1 | LenGte      | 4           | 6     |
      | user1 | LenEq       | 5           | 10    |