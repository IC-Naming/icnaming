@registrar
Feature: Name Transaction

  Background:
    Given Reinstall canisters
      | name      |
      | registrar |
      | registry  |
      | resolver  |
    And Some users already have some quotas
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 10    |
      | user1 | LenGte      | 4           | 6     |
      | user1 | LenEq       | 5           | 10    |
      | user2 | LenGte      | 3           | 10    |

  Scenario: Transfer name by user
    Given User "user1" register name "hello.icp" with quote "LenGte(3)"
    And User "user1" update resolver "hello.icp" with values
      | key          | value                                      |
      | token.icp    | qjdve-lqaaa-aaaaa-aaaeq-cai                |
      | token.btc    | 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa         |
      | token.ltc    | LUwxSibYhxq2u6RfhQmkuTPZRk2wNjwLbE         |
      | token.eth    | 0xb436ef6cc9f24193ccb42f98be2b1db764484514 |
      | canister.icp | qsgjb-riaaa-aaaaa-aaaga-cai                |
    When User "user1" transfer name "hello.icp" to User "user2"
    Then last name transfer result status is "Ok"
    And registrar get_details "hello.icp" result is
      | key        | value     |
      | owner      | user2     |
      | name       | hello.icp |
      | expired_at | 1         |
      | created_at | 0         |
    And get_record_value "hello.icp" should be as below
      | key | value |
    And registry get_details "hello.icp" should be as below
      | key      | value     |
      | name     | hello.icp |
      | owner    | user2     |
      | resolver | public    |
      | ttl      | 600       |

  Scenario: Transfer name twice
    Given User "user1" register name "hello.icp" with quote "LenGte(3)"
    When User "user1" transfer name "hello.icp" to User "user2"
    And User "user2" update resolver "hello.icp" with values
      | key          | value                                      |
      | token.icp    | qjdve-lqaaa-aaaaa-aaaeq-cai                |
      | token.btc    | 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa         |
      | token.ltc    | LUwxSibYhxq2u6RfhQmkuTPZRk2wNjwLbE         |
      | token.eth    | 0xb436ef6cc9f24193ccb42f98be2b1db764484514 |
      | canister.icp | qsgjb-riaaa-aaaaa-aaaga-cai                |
    And User "user2" transfer name "hello.icp" to User "user3"
    Then last name transfer result status is "Ok"
    And registrar get_details "hello.icp" result is
      | key        | value     |
      | owner      | user3     |
      | name       | hello.icp |
      | expired_at | 1         |
      | created_at | 0         |
    And get_record_value "hello.icp" should be as below
      | key | value |
    And registry get_details "hello.icp" should be as below
      | key      | value     |
      | name     | hello.icp |
      | owner    | user3     |
      | resolver | public    |
      | ttl      | 600       |

  Scenario: Transfer name fail without permission
    Given User "user1" register name "hello.icp" with quote "LenGte(3)"
    When User "user2" transfer name "hello.icp" to User "user3"
    Then last name transfer result status is "permission deny"

  Scenario: Transfer name by transfer_from
    Given User "user1" register name "hello.icp" with quote "LenGte(3)"
    And User "user1" update resolver "hello.icp" with values
      | key          | value                                      |
      | token.icp    | qjdve-lqaaa-aaaaa-aaaeq-cai                |
      | token.btc    | 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa         |
      | token.ltc    | LUwxSibYhxq2u6RfhQmkuTPZRk2wNjwLbE         |
      | token.eth    | 0xb436ef6cc9f24193ccb42f98be2b1db764484514 |
      | canister.icp | qsgjb-riaaa-aaaaa-aaaga-cai                |
    And User "user1" approve name "hello.icp" to User "user2"
    When User "user2" transfer name "hello.icp" by transfer_from
    Then last name transfer_from result status is "Ok"
    And registrar get_details "hello.icp" result is
      | key        | value     |
      | owner      | user2     |
      | name       | hello.icp |
      | expired_at | 1         |
      | created_at | 0         |
    And get_record_value "hello.icp" should be as below
      | key | value |
    And registry get_details "hello.icp" should be as below
      | key      | value     |
      | name     | hello.icp |
      | owner    | user2     |
      | resolver | public    |
      | ttl      | 600       |

  Scenario: Transfer name by transfer_from failed without approve
    Given User "user1" register name "hello.icp" with quote "LenGte(3)"
    When User "user2" transfer name "hello.icp" by transfer_from
    Then last name transfer_from result status is "permission deny"

  Scenario: Removing approval manually
    Given User "user1" register name "hello.icp" with quote "LenGte(3)"
    And User "user1" approve name "hello.icp" to User "user2"
    When User "user1" approve name "hello.icp" to User "anonymous"
    And User "user2" transfer name "hello.icp" by transfer_from
    Then last name transfer_from result status is "permission deny"

  Scenario: Removing approval after transfer
    Given User "user1" register name "hello.icp" with quote "LenGte(3)"
    And User "user1" approve name "hello.icp" to User "user2"
    And User "user1" transfer name "hello.icp" to User "user3"
    When User "user3" transfer name "hello.icp" by transfer_from
    Then last name transfer_from result status is "permission deny"

  Scenario: Reclaim name successfully if owner of registry is changed
    Given User "user1" register name "hello.icp" with quote "LenGte(3)"
    And User "user1" update resolver "hello.icp" with values
      | key       | value                              |
      | token.icp | qjdve-lqaaa-aaaaa-aaaeq-cai        |
      | token.btc | 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa |
    And User "user1" set registry owner for "hello.icp" to "user2"
    When User "user1" reclaim name "hello.icp"
    And registrar get_details "hello.icp" result is
      | key        | value     |
      | owner      | user1     |
      | name       | hello.icp |
      | expired_at | 1         |
      | created_at | 0         |
    And get_record_value "hello.icp" should be as below
      | key       | value                              |
      | token.icp | qjdve-lqaaa-aaaaa-aaaeq-cai        |
      | token.btc | 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa |
    And registry get_details "hello.icp" should be as below
      | key      | value     |
      | name     | hello.icp |
      | owner    | user1     |
      | resolver | public    |
      | ttl      | 600       |
