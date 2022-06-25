@registrar
Feature: Register a name with payment

  Background:
    Given Reinstall registrar related canisters

  Scenario: Multiple user pay to buy different names
    Given User "user1" balance is set to be "10 icp"
    And User "user2" balance is set to be "10 icp"
    When User "user1" register name "icnamingtest1.ic" for "3" years and pay "3 icp"
    And User "user2" register name "icnamingtest2.ic" for "3" years and pay "3 icp"
    Then registrar get_details "icnamingtest1.ic" result is
      | key        | value            |
      | owner      | user1            |
      | name       | icnamingtest1.ic |
      | expired_at | 3                |
      | created_at | 0                |
    And registrar get_details "icnamingtest2.ic" result is
      | key        | value            |
      | owner      | user2            |
      | name       | icnamingtest2.ic |
      | expired_at | 3                |
      | created_at | 0                |
    And User "user1" balance is "7 icp"
    And User "user2" balance is "7 icp"

  Scenario: The user who is the first to pay will be the owner of the name
    Given User "user1" balance is set to be "10 icp"
    And User "user2" balance is set to be "10 icp"
    When User "user1" register name "icnamingtest.ic" for "3" years and pay "3 icp"
    And User "user2" register name "icnamingtest.ic" for "3" years and pay "3 icp"
    And registrar get_details "icnamingtest.ic" result is
      | key        | value           |
      | owner      | user1           |
      | name       | icnamingtest.ic |
      | expired_at | 3               |
      | created_at | 0               |
    And User "user1" balance is "7 icp"
    And User "user2" balance is "10 icp"
