@registrar
Feature: Register a name with payment

  Background:
    Given Reinstall registrar related canisters

  Scenario: Submit a name with 6 characters
    When I submit a order to register name "hello.ark" for "3" years
    Then Order submitting result in status 'name is invalid, reason: "the name need to be at least 6 characters long"'

  Scenario: Submit a order
    When I submit a order to register name "what-a-nice-day.ark" for "3" years
    Then I found my pending order with "what-a-nice-day.ark" for "3" years

  Scenario: Cancel pending order
    Given I submit a order to register name "what-a-nice-day.ark" for "3" years
    When I cancel my pending order
    Then I found there is no pending order
    And I submit a order to register name "what-a-nice-day.ark" for "3" years
    And I found my pending order as bellow
      | key              | value               |
      | name             | what-a-nice-day.ark |
      | years            | 3                   |
      | price_icp_in_e8s | 300_000_000         |
      | quota_type       | LenGte(7)           |

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
      | s6d9w5r1.ark | 3     | 300_000_000      | LenGte(7)  |
      | 6s3d2f1.ark  | 3     | 300_000_000      | LenGte(7)  |
      | 365214.ark   | 3     | 330_000_000      | LenGte(6)  |
