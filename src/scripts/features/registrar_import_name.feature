@registrar
Feature: Import Names by admin

  Background:
    Given Reinstall registrar related canisters

  Scenario: Import Name by admin successfully
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
    And registrar get_details "na2.ic" result is
      | key        | value  |
      | owner      | user1  |
      | name       | na2.ic |
      | expired_at | 2      |
      | created_at | 0      |
    And registrar get_details "iiiiii.ic" result is
      | key        | value     |
      | owner      | user2     |
      | name       | iiiiii.ic |
      | expired_at | 3         |
      | created_at | 0         |

  Scenario: Import Name by admin twice then data no change
    When Admin import names as following
      | name     | owner | years |
      | name1.ic | user1 | 1     |
      | name1.ic | user2 | 3     |
    Then registrar get_details "name1.ic" result is
      | key        | value    |
      | owner      | user1    |
      | name       | name1.ic |
      | expired_at | 1        |
      | created_at | 0        |

  Scenario: Import name from csv file with 5000 entries and then validate the first and last. Maximum timeout is 5 minutes
    Given Import registrar names data from csv file "RegistrarImportNames.csv"
    Then registrar get_details "sourceforge.ic" result is
      | key        | value          |
      | owner      | user3          |
      | name       | sourceforge.ic |
      | expired_at | 1              |
      | created_at | 0              |
    And registrar get_details "xn--80auehs.ic" result is
      | key        | value          |
      | owner      | user1          |
      | name       | xn--80auehs.ic |
      | expired_at | 2              |
      | created_at | 0              |
