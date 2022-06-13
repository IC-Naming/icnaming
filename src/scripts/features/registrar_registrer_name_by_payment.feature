@registrar
Feature: Register a name with payment

  Background:
    Given Reinstall registrar related canisters
    And User "user1" balance is set to be "21 icp"

  Scenario: Submit a name with 5 characters
    When User "user1" register name "hello.ic" for "10" years and pay "3 icp"
    Then Last register_with_payment result is 'name is invalid, reason: "the name need to be at least 6 characters long"'

  Scenario: Register name with payment success
    When User "user1" register name "7654321.ic" for "10" years and pay "10 icp"
    Then Last register_with_payment result is 'Ok'
    And registrar get_details "7654321.ic" result is
      | key        | value      |
      | owner      | user1      |
      | name       | 7654321.ic |
      | expired_at | 10         |
      | created_at | 0          |
    And User "user1" balance is "11 icp"

  Scenario: Register name with insufficient payment
    When User "user1" register name "7654321.ic" for "10" years and pay "1 icp"
    Then Last register_with_payment result is 'price changed, please refresh and try again'
    And name "7654321.ic" is available
    And User "user1" balance is "21 icp"

  Scenario: Register name with payment success when 95% price changed
    When User "user1" register name "7654321.ic" for "10" years and pay "9.5 icp"
    Then registrar get_details "7654321.ic" result is
      | key        | value      |
      | owner      | user1      |
      | name       | 7654321.ic |
      | expired_at | 10         |
      | created_at | 0          |
    And User "user1" balance is "11.5 icp"