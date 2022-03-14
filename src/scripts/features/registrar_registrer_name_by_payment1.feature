@registrar
Feature: Register a name with payment

  Background:
    Given Reinstall registrar related canisters
    And Record payment version

  Scenario: Pay enough to my pending order
    Given I submit a order to register name "what-a-nice-day.icp" for "3" years
    When Pay for my pending order with amount "3 icp"
    And Wait for payment version increased with "1"
    Then I found there is no pending order
    And registrar get_details "what-a-nice-day.icp" result is
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
    And registrar get_details "what-a-nice-day.icp" result is
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
    And registrar get_details "what-a-nice-day.icp" result is
      | key        | value               |
      | owner      | main                |
      | name       | what-a-nice-day.icp |
      | expired_at | 3                   |
      | created_at | 0                   |
