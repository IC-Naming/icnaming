@registrar
Feature: Register a name with payment

  Background:
    Given Reinstall registrar related canisters
    And User "user1" balance is set to be "10 icp"

  Scenario: Pay enough to my pending order
    Given User "user1" submit a order to register name "what-a-nice-day.ark" for "3" years
    When User "user1" pay for my pending order with amount "3 icp"
    Then I found there is no pending order
    And registrar get_details "what-a-nice-day.ark" result is
      | key        | value               |
      | owner      | user1                |
      | name       | what-a-nice-day.ark |
      | expired_at | 3                   |
      | created_at | 0                   |
    And User "user1" balance is "7 icp"

  Scenario: Pay not enough to my pending order
    Given User "user1" submit a order to register name "what-a-nice-day.ark" for "3" years
    When User "user1" pay for my pending order with amount "1 icp"
    Then User "user1" found my pending order with "what-a-nice-day.ark" for "3" years, status "New"
    And name "what-a-nice-day.ark" is available
    And User "user1" balance is "10 icp"
