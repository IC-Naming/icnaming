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
    When User "user1" register name "hello.ic" with quote "LenGte(3)"
    Then registrar get_details "hello.ic" result is
      | key        | value     |
      | owner      | user1     |
      | name       | hello.ic |
      | expired_at | 1         |
      | created_at | 0         |
    And  User quota status should be as below
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 9     |

  Scenario: Register a name with not enough quota
    When User "user1" register name "hello1.ic" with quote "LenEq(6)"
    Then Register with quota result in status 'name is invalid, reason: "User has no quota for len_eq(6)"'

  Scenario: Register a name for other user
    When User "user1" register name "hello.ic" with quote "LenGte(4)" for "user2" with "3" years
    Then registrar get_details "hello.ic" result is
      | key        | value     |
      | owner      | user2     |
      | name       | hello.ic |
      | expired_at | 3         |
      | created_at | 0         |

    And  User quota status should be as below
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 10    |
      | user1 | LenGte      | 4           | 3     |

  Scenario: Register a name with quota but registry canister down
    Given canister "registry" is down
    And User "user1" register name "hello2.ic" with quote "LenGte(3)"
    When Register with quota result in status 'error from remote, ErrorInfo { code: 26, message: "canister call error, rejected by \"DestinationInvalid\"" }'
    And  User quota status should be as below
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 10    |
      | user1 | LenGte      | 4           | 6     |
      | user1 | LenEq       | 5           | 10    |

  Scenario: Register name and get last registrations
    When User "user1" register name "hello1.ic" with quote "LenGte(3)"
    And User "user1" register name "hello2.ic" with quote "LenGte(3)"
    And User "user1" register name "hello3.ic" with quote "LenGte(3)"
    Then Get last registrations result is
      | name |
      | hello3.ic |
      | hello2.ic |
      | hello1.ic |
