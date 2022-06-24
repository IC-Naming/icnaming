@resolver
Feature: Query Api

  Background:
    Given Reinstall registrar related canisters
    And Name "hello.ic" is already taken
    And Name "wonderful.ic" is already taken

  Scenario: It is impossible to call ensure_resolver_created from other principal but registry
    When I call ensure_resolver_created "hello.ic"
    Then ensure_resolver_created result in status "Unauthorized, please login first"

  Scenario: Query default resolver values
    Then get_record_value "hello.ic" should be as below
      | key | value |

  Scenario: Update resolver values
    When I update resolver "hello.ic" with values
      | key                                   | value                                                           |
      | token.icp                             | qjdve-lqaaa-aaaaa-aaaeq-cai                                     |
      | token.btc                             | 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa                              |
      | token.ltc                             | LUwxSibYhxq2u6RfhQmkuTPZRk2wNjwLbE                              |
      | token.eth                             | 0xb436ef6cc9f24193ccb42f98be2b1db764484514                      |
      | canister.icp                          | qsgjb-riaaa-aaaaa-aaaga-cai                                     |
      | settings.reverse_resolution.principal | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe |
    Then get_record_value "hello.ic" should be as below
      | key                                   | value                                                           |
      | token.icp                             | qjdve-lqaaa-aaaaa-aaaeq-cai                                     |
      | token.btc                             | 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa                              |
      | token.ltc                             | LUwxSibYhxq2u6RfhQmkuTPZRk2wNjwLbE                              |
      | token.eth                             | 0xb436ef6cc9f24193ccb42f98be2b1db764484514                      |
      | canister.icp                          | qsgjb-riaaa-aaaaa-aaaga-cai                                     |
      | settings.reverse_resolution.principal | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe |
    And Reverse resolve name "2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe" should be "hello.ic"

  Scenario: Update resolver values with invalid key
    When I update resolver "hello.ic" with values
      | key         | value                       |
      | invalid.key | qsgjb-riaaa-aaaaa-aaaga-cai |
    Then update_record_value result in status 'invalid resolver key: "invalid.key"'

  Scenario: Delete resolver value by setting value to None
    Given I update resolver "hello.ic" with values
      | key          | value                       |
      | token.icp    | qjdve-lqaaa-aaaaa-aaaeq-cai |
      | canister.icp | qsgjb-riaaa-aaaaa-aaaga-cai |
    When I update resolver "hello.ic" with values
      | key          | value                       |
      | token.icp    | qjdve-lqaaa-aaaaa-aaaeq-cai |
      | canister.icp |                             |
    Then get_record_value "hello.ic" should be as below
      | key       | value                       |
      | token.icp | qjdve-lqaaa-aaaaa-aaaeq-cai |

  Scenario: Delete reverse resolution principal
    Given I update resolver "hello.ic" with values
      | key                                   | value                                                           |
      | settings.reverse_resolution.principal | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe |
    When I update resolver "hello.ic" with values
      | key                                   | value |
      | settings.reverse_resolution.principal |       |
    Then get_record_value "hello.ic" should be as below
      | key | value |
    And Reverse resolve name "2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe" should be "none"

  Scenario: Update reverse resolution principal multiple times
    Given I update resolver "hello.ic" with values
      | key                                   | value                                                           |
      | settings.reverse_resolution.principal | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe |
    When I update resolver "wonderful.ic" with values
      | key                                   | value                                                           |
      | settings.reverse_resolution.principal | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe |
    Then get_record_value "hello.ic" should be as below
      | key | value |
    And get_record_value "wonderful.ic" should be as below
      | key                                   | value                                                           |
      | settings.reverse_resolution.principal | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe |
    And Reverse resolve name "2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe" should be "wonderful.ic"

