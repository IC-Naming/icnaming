@registrar
Feature: Register a name with payment

  Background:
    Given Reinstall registrar related canisters

  Scenario: Pay multiple times not enough to my pending order
    Given User "user1" submit a order to register name "what-a-nice-day.ic" for "3" years
    And User "user1" balance is set to be "10 icp"
    When User "user1" pay for my pending order with amount "1 icp"
    And User "user1" pay for my pending order with amount "1 icp"
    And User "user1" pay for my pending order with amount "1 icp"
    Then User "user1" found my pending order with "what-a-nice-day.ic" for "3" years, status "New"
    And name "what-a-nice-day.ic" is available
    And User "user1" balance is "10 icp"

  Scenario: Multiple user pay to buy different names
    Given User "user1" submit a order to register name "icnamingtest1.ic" for "3" years
    And User "user2" submit a order to register name "icnamingtest2.ic" for "3" years
    And User "user1" balance is set to be "10 icp"
    And User "user2" balance is set to be "10 icp"
    When User "user1" pay for my pending order with amount "3 icp"
    And User "user2" pay for my pending order with amount "3 icp"
    Then User "user1" found there is no pending order
    And User "user2" found there is no pending order
    And registrar get_details "icnamingtest1.ic" result is
      | key        | value             |
      | owner      | user1             |
      | name       | icnamingtest1.ic |
      | expired_at | 3                 |
      | created_at | 0                 |
    And registrar get_details "icnamingtest2.ic" result is
      | key        | value             |
      | owner      | user2             |
      | name       | icnamingtest2.ic |
      | expired_at | 3                 |
      | created_at | 0                 |
    And User "user1" balance is "7 icp"
    And User "user2" balance is "7 icp"

  Scenario: The user who is the first to pay will be the owner of the name
    Given User "user1" submit a order to register name "icnamingtest.ic" for "3" years
    And User "user2" submit a order to register name "icnamingtest.ic" for "3" years
    And User "user1" balance is set to be "10 icp"
    And User "user2" balance is set to be "10 icp"
    When User "user1" pay for my pending order with amount "3 icp"
    And User "user2" pay for my pending order with amount "3 icp"
    Then User "user1" found there is no pending order
    And User "user2" found my pending order with "icnamingtest.ic" for "3" years, status "New"
    And registrar get_details "icnamingtest.ic" result is
      | key        | value            |
      | owner      | user1            |
      | name       | icnamingtest.ic |
      | expired_at | 3                |
      | created_at | 0                |
    And User "user1" balance is "7 icp"
    And User "user2" balance is "10 icp"
