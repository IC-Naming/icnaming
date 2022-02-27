@registrar
Feature: Register a name with payment

  Background:
    Given Reinstall registrar related canisters
    And Record payment version

  Scenario: Submit a name with 6 characters
    When I submit a order to register name "hello.icp" for "3" years
    Then Order submitting result in status 'name is invalid, reason: "the name need to be at least 7 characters long"'

  Scenario: Submit a order
    When I submit a order to register name "what-a-nice-day.icp" for "3" years
    Then I found my pending order with "what-a-nice-day.icp" for "3" years

  Scenario: Cancel pending order
    Given I submit a order to register name "what-a-nice-day.icp" for "3" years
    When I cancel my pending order
    Then I found there is no pending order

  Scenario Outline: Submit a order and waiting for payment
    Given I submit a order to register name "<name>" for "<years>" years
    Then I found my pending order as bellow
      | key              | value              |
      | name             | <name>             |
      | years            | <years>            |
      | price_icp_in_e8s | <price_icp_in_e8s> |
      | quota_type       | <quota_type>       |
    Examples:
      | name         | years | price_icp_in_e8s | quota_type |
      | s6d9w5r1.icp | 3     | 300_000_000      | LenGte(7)  |
      | 6s3d2f1.icp  | 3     | 300_000_000      | LenGte(7)  |

  Scenario: Pay enough to my pending order
    Given I submit a order to register name "what-a-nice-day.icp" for "3" years
    When Pay for my pending order with amount "3 icp"
    And Wait for payment version increased with "1"
    Then I found there is no pending order
    And get_details "what-a-nice-day.icp" result is
      | key        | value               |
      | owner      | main                |
      | name       | what-a-nice-day.icp |
      | expired_at | 3                   |
      | created_at | 0                   |

  Scenario: Pay enough to my pending order
    Given I submit a order to register name "what-a-nice-day.icp" for "3" years
    When Pay for my pending order with amount "3 icp"
    And User "main" confirm pay order with block height "1"
    Then I found there is no pending order
    And get_details "what-a-nice-day.icp" result is
      | key        | value               |
      | owner      | main                |
      | name       | what-a-nice-day.icp |
      | expired_at | 3                   |
      | created_at | 0                   |

  Scenario: Pay not enough to my pending order
    Given I submit a order to register name "what-a-nice-day.icp" for "3" years
    When Pay for my pending order with amount "1 icp"
    And Wait for payment version increased with "1"
    Then I found my pending order with "what-a-nice-day.icp" for "3" years
    And name "what-a-nice-day.icp" is available

  Scenario: Pay multiple times enough to my pending order
    Given I submit a order to register name "what-a-nice-day.icp" for "3" years
    When Pay for my pending order with amount "1 icp"
    And Pay for my pending order with amount "2 icp"
    And Wait for payment version increased with "2"
    Then I found there is no pending order
    And get_details "what-a-nice-day.icp" result is
      | key        | value               |
      | owner      | main                |
      | name       | what-a-nice-day.icp |
      | expired_at | 3                   |
      | created_at | 0                   |

  Scenario: Pay multiple times not enough to my pending order
    Given I submit a order to register name "what-a-nice-day.icp" for "3" years
    When Pay for my pending order with amount "1 icp"
    And Pay for my pending order with amount "1 icp"
    And Wait for payment version increased with "2"
    Then I found my pending order with "what-a-nice-day.icp" for "3" years
    And name "what-a-nice-day.icp" is available


  Scenario: Multiple user pay to buy different names
    Given User "user1" submit a order to register name "icnamingtest1.icp" for "3" years
    And User "user2" submit a order to register name "icnamingtest2.icp" for "3" years
    And User "user1" balance is set to be "10 icp"
    And User "user2" balance is set to be "10 icp"
    When User "user1" pay for my pending order with amount "3 icp"
    And User "user2" pay for my pending order with amount "3 icp"
    And Wait for payment version increased with "2"
    Then User "user1" found there is no pending order
    And User "user2" found there is no pending order
    And get_details "icnamingtest1.icp" result is
      | key        | value             |
      | owner      | user1             |
      | name       | icnamingtest1.icp |
      | expired_at | 3                 |
      | created_at | 0                 |
    And get_details "icnamingtest2.icp" result is
      | key        | value             |
      | owner      | user2             |
      | name       | icnamingtest2.icp |
      | expired_at | 3                 |
      | created_at | 0                 |

  Scenario: The user who is the first to pay will be the owner of the name
    Given User "user1" submit a order to register name "icnamingtest.icp" for "3" years
    And User "user2" submit a order to register name "icnamingtest.icp" for "3" years
    And User "user1" balance is set to be "10 icp"
    And User "user2" balance is set to be "10 icp"
    When User "user1" pay for my pending order with amount "3 icp"
    And User "user2" pay for my pending order with amount "3 icp"
    And Wait for payment version increased with "2"
    Then User "user1" found there is no pending order
    And User "user2" found my pending order with "icnamingtest.icp" for "3" years, status "WaitingToRefund"
    And get_details "icnamingtest.icp" result is
      | key        | value            |
      | owner      | user1            |
      | name       | icnamingtest.icp |
      | expired_at | 3                |
      | created_at | 0                |

  Scenario: A user need to refund the pending order when name is taken by something else
    Given ICNaming ledger refund subaccount balance is set to be "10 icp"
    And User "user1" balance is set to be "10 icp"
    And User "user2" balance is set to be "10 icp"
    And User "user1" submit a order to register name "icnamingtest.icp" for "3" years
    And User "user2" submit a order to register name "icnamingtest.icp" for "3" years
    And User "user1" pay for my pending order with amount "3 icp"
    And User "user2" pay for my pending order with amount "3 icp"
    And Wait for payment version increased with "2"
    And User "user2" found my pending order with "icnamingtest.icp" for "3" years, status "WaitingToRefund"
    When User "user2" refund my pending order
    Then Last refund response is "Ok"
    And User "user2" found there is no pending order
    And User "user2" balance is "9.9999 icp"

  Scenario: A user is able to refund the pending order when refund subaccount balance is enough
    Given ICNaming ledger refund subaccount balance is set to be "1 icp"
    And User "user1" balance is set to be "10 icp"
    And User "user2" balance is set to be "10 icp"
    And User "user1" submit a order to register name "icnamingtest.icp" for "3" years
    And User "user2" submit a order to register name "icnamingtest.icp" for "3" years
    And User "user1" pay for my pending order with amount "3 icp"
    And User "user2" pay for my pending order with amount "3 icp"
    And Wait for payment version increased with "2"
    And User "user2" found my pending order with "icnamingtest.icp" for "3" years, status "WaitingToRefund"
    And User "user2" refund my pending order
    And Last refund response is "refund failed, please try again later"
    And ICNaming ledger refund subaccount balance is topped up with "3 icp"
    When User "user2" refund my pending order
    Then Last refund response is "Ok"
    And User "user2" found there is no pending order
    And User "user2" balance is "9.9999 icp"
    And ICNaming ledger refund subaccount balance is "0.9999 icp"
    And ICNaming ledger receiver subaccount balance is "6.0 icp"


