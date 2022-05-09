@resolver
Feature: Query Api

  Background:
    Given Reinstall registrar related canisters
    And Name "hello.ark" is already taken

  Scenario: It is impossible to call ensure_resolver_created from other principal but registry
    When I call ensure_resolver_created "hello.ark"
    Then ensure_resolver_created result in status "Unauthorized, please login first"

  Scenario: Query default resolver values
    Then get_record_value "hello.ark" should be as below
      | key | value |

  Scenario: Update resolver values
    When I update resolver "hello.ark" with values
      | key          | value                                      |
      | token.ark    | qjdve-lqaaa-aaaaa-aaaeq-cai                |
      | token.btc    | 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa         |
      | token.ltc    | LUwxSibYhxq2u6RfhQmkuTPZRk2wNjwLbE         |
      | token.eth    | 0xb436ef6cc9f24193ccb42f98be2b1db764484514 |
      | canister.ark | qsgjb-riaaa-aaaaa-aaaga-cai                |
    Then get_record_value "hello.ark" should be as below
      | key          | value                                      |
      | token.ark    | qjdve-lqaaa-aaaaa-aaaeq-cai                |
      | token.btc    | 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa         |
      | token.ltc    | LUwxSibYhxq2u6RfhQmkuTPZRk2wNjwLbE         |
      | token.eth    | 0xb436ef6cc9f24193ccb42f98be2b1db764484514 |
      | canister.ark | qsgjb-riaaa-aaaaa-aaaga-cai                |

  Scenario: Update resolver values with invalid key
    When I update resolver "hello.ark" with values
      | key         | value                       |
      | invalid.key | qsgjb-riaaa-aaaaa-aaaga-cai |
    Then update_record_value result in status 'invalid resolver key: "invalid.key"'


  Scenario: Delete resolver value by setting value to None
    Given I update resolver "hello.ark" with values
      | key          | value                       |
      | token.ark    | qjdve-lqaaa-aaaaa-aaaeq-cai |
      | canister.ark | qsgjb-riaaa-aaaaa-aaaga-cai |
    When I update resolver "hello.ark" with values
      | key          | value                       |
      | token.ark    | qjdve-lqaaa-aaaaa-aaaeq-cai |
      | canister.ark |                             |
    Then get_record_value "hello.ark" should be as below
      | key       | value                       |
      | token.ark | qjdve-lqaaa-aaaaa-aaaeq-cai |
