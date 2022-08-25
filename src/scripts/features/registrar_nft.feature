@registrar
Feature: nft test

  Background:
    Given Reinstall registrar related canisters


  Scenario: Metadata
    When Admin import names as following
      | name      | owner | years |
      | name1.ic  | user1 | 1     |
      | na2.ic    | user1 | 2     |
      | iiiiii.ic | user2 | 3     |
    Then registrar get_details "name1.ic" result is
      | key        | value    |
      | owner      | user1    |
      | name       | name1.ic |
      | expired_at | 1        |
      | created_at | 0        |
    And registrar metadata "name1.ic"
