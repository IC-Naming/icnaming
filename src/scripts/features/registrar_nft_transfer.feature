@registrar
Feature: Yumi market transfer Api

  Background:
    Given Reinstall registrar related canisters
    Then Admin import names as following
      | name      | owner | years |
      | name1.ic  | user1 | 1     |
      | na2.ic    | user1 | 2     |
      | iiiiii.ic | user2 | 3     |

  Scenario: Ext transfer success
    Then registrar get_details "name1.ic" result is
      | key        | value    |
      | owner      | user1    |
      | name       | name1.ic |
      | expired_at | 1        |
      | created_at | 0        |
    Given registrar ext_transfer action
      | caller | name     | from  | to    |
      | user1  | name1.ic | user1 | user2 |
    When all registrar ext_transfer is ok
    And registrar get_details "name1.ic" result is
      | key        | value    |
      | owner      | user2    |
      | name       | name1.ic |
      | expired_at | 1        |
      | created_at | 0        |

  @dev
  Scenario: Ext transfer from the allowed caller success
    Then registrar get_details "name1.ic" result is
      | key        | value    |
      | owner      | user1    |
      | name       | name1.ic |
      | expired_at | 1        |
      | created_at | 0        |
    Given registrar ext_approve action
      | spender | name     |
      | user3   | name1.ic |
    Given registrar ext_transfer action
      | caller | name     | from  | to    |
      | user3  | name1.ic | user1 | user2 |
    When all registrar ext_transfer is ok


  Scenario: Ext transfer failed invalid owner
    Then registrar get_details "name1.ic" result is
      | key        | value    |
      | owner      | user1    |
      | name       | name1.ic |
      | expired_at | 1        |
      | created_at | 0        |
    Then registrar metadata "name1.ic" result is
      | key  | value    |
      | name | name1.ic |

  Scenario: Ext transfer failed caller unknown
    Then registrar get_details "name1.ic" result is
      | key        | value    |
      | owner      | user1    |
      | name       | name1.ic |
      | expired_at | 1        |
      | created_at | 0        |
    Then registrar metadata "name1.ic" result is
      | key  | value    |
      | name | name1.ic |
