@registrar
Feature: Import Names by admin

  Background:
    Given Reinstall registrar related canisters

  Scenario: Import Name by admin successfully
    When Admin import names as following
      | name       | owner | years |
      | name1.icp  | user1 | 1     |
      | na2.icp    | user1 | 2     |
      | iiiiii.icp | user2 | 3     |
    Then registrar get_details "name1.icp" result is
      | key        | value     |
      | owner      | user1     |
      | name       | name1.icp |
      | expired_at | 1         |
      | created_at | 0         |
    And registrar get_details "na2.icp" result is
      | key        | value   |
      | owner      | user1   |
      | name       | na2.icp |
      | expired_at | 2       |
      | created_at | 0       |
    And registrar get_details "iiiiii.icp" result is
      | key        | value      |
      | owner      | user2      |
      | name       | iiiiii.icp |
      | expired_at | 3          |
      | created_at | 0          |

  Scenario: Import Name by admin twice then data no change
    When Admin import names as following
      | name      | owner | years |
      | name1.icp | user1 | 1     |
      | name1.icp | user2 | 3     |
    Then registrar get_details "name1.icp" result is
      | key        | value     |
      | owner      | user1     |
      | name       | name1.icp |
      | expired_at | 1         |
      | created_at | 0         |
