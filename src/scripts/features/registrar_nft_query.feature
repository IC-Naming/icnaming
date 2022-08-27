@registrar
Feature: Yumi market query Api

  Background:
    Given Reinstall registrar related canisters
    Then Admin import names as following
      | name      | owner | years |
      | name1.ic  | user1 | 1     |
      | na2.ic    | user1 | 2     |
      | iiiiii.ic | user2 | 3     |

  @dev
  Scenario: Metadata
    Then registrar get_details "name1.ic" result is
      | key        | value    |
      | owner      | user1    |
      | name       | name1.ic |
      | expired_at | 1        |
      | created_at | 0        |
    Then registrar metadata "name1.ic" result is
      | key  | value    |
      | name | name1.ic |


  Scenario: getTokens
    Then registrar get_details "name1.ic" result is
      | key        | value    |
      | owner      | user1    |
      | name       | name1.ic |
      | expired_at | 1        |
      | created_at | 0        |
    Then registrar getTokens result is
      | key  | value    |
      | name | name1.ic |


  Scenario: getRegistry
    Then registrar get_details "name1.ic" result is
      | key        | value    |
      | owner      | user1    |
      | name       | name1.ic |
      | expired_at | 1        |
      | created_at | 0        |
    Then registrar getRegistry result is
      | key  | value    |
      | name | name1.ic |

  Scenario: supply
    Then registrar get_details "name1.ic" result is
      | key        | value    |
      | owner      | user1    |
      | name       | name1.ic |
      | expired_at | 1        |
      | created_at | 0        |
    Then registrar supply result is "3"
