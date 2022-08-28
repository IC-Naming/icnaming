@registrar
Feature: EXT token standard transfer API

  Background:
    Given Reinstall registrar related canisters
    And Admin import names as following
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

  Scenario: Ext transfer success
    Given registrar ext_transfer action
      | caller | name     | from  | to    | from_type | to_type   |
      | user1  | name1.ic | user1 | user2 | principal | principal |
    When all registrar ext_transfer is ok
    Then registrar get_details "name1.ic" result is
      | key        | value    |
      | owner      | user2    |
      | name       | name1.ic |
      | expired_at | 1        |
      | created_at | 0        |

  Scenario: Ext transfer from owner to receiver
    Given registrar ext_approve name to spender, the caller is the name owner
      | spender | name     |
      | user3   | name1.ic |
    And registrar ext_transfer action
      | caller | name     | from  | to    | from_type | to_type   |
      | user3  | name1.ic | user1 | user2 | principal | principal |
    When all registrar ext_transfer is ok
    Then registrar get_details "name1.ic" result is
      | key        | value    |
      | owner      | user2    |
      | name       | name1.ic |
      | expired_at | 1        |
      | created_at | 0        |

  Scenario: Ext transfer from owner to allowance
    Given registrar ext_approve name to spender, the caller is the name owner
      | spender | name     |
      | user3   | name1.ic |
    And registrar ext_transfer action
      | caller | name     | from  | to    | from_type | to_type   |
      | user3  | name1.ic | user1 | user3 | principal | principal |
    When all registrar ext_transfer is ok
    Then registrar get_details "name1.ic" result is
      | key        | value    |
      | owner      | user3    |
      | name       | name1.ic |
      | expired_at | 1        |
      | created_at | 0        |

  Scenario: Ext transfer failed, from is invalid owner, caller is none
    Given registrar ext_transfer action
      | caller | name     | from  | to    | from_type | to_type   |
      | none   | name1.ic | user3 | user2 | principal | principal |
    When last registrar ext_transfer result is err, expected err is "Other" and message is "owner is invalid"

  Scenario: Ext transfer failed, from is invalid owner, caller is owner
    Given registrar ext_transfer action
      | caller | name     | from  | to    | from_type | to_type   |
      | user1  | name1.ic | user3 | user2 | principal | principal |
    When last registrar ext_transfer result is err, expected err is "Other" and message is "owner is invalid"

  Scenario: Ext transfer failed, caller is not approved
    Given registrar ext_transfer action
      | caller | name     | from  | to    | from_type | to_type   |
      | user3  | name1.ic | user1 | user2 | principal | principal |
    When last registrar ext_transfer result is err, expected err is "Other" and message is "permission deny"

  Scenario: Ext transfer failed account id not supported
    Given registrar ext_approve name to spender, the caller is the name owner
      | spender | name     |
      | user3   | name1.ic |
    And registrar ext_transfer action
      | caller | name     | from                                                             | to    | from_type | to_type   |
      | user3  | name1.ic | 3352b4176f9818dfa25c862cbca82f0f05b8e150dded0263e2ef05b094103e34 | user2 | address   | principal |
    When last registrar ext_transfer result is err, expected err is "Other" and message is "account identifier is not supported"
    Given registrar ext_transfer action
      | caller | name     | from  | to                                                               | from_type | to_type |
      | user3  | name1.ic | user1 | 3352b4176f9818dfa25c862cbca82f0f05b8e150dded0263e2ef05b094103e34 | principal | address |
    When last registrar ext_transfer result is err, expected err is "Other" and message is "account identifier is not supported"
    Given registrar ext_transfer action
      | caller | name     | from                                                             | to                                                               | from_type | to_type |
      | user3  | name1.ic | 3352b4176f9818dfa25c862cbca82f0f05b8e150dded0263e2ef05b094103e34 | 3352b4176f9818dfa25c862cbca82f0f05b8e150dded0263e2ef05b094103e34 | address   | address |
    When last registrar ext_transfer result is err, expected err is "Other" and message is "account identifier is not supported"

  Scenario: Ext allowance success
    Given registrar allowance action, caller is none
      | name     | from  | to    | from_type |
      | name1.ic | user1 | user2 | principal |
    When all registrar allowance is ok, and the value is "1"

  Scenario: Ext allowance failed invalid owner
    Given registrar allowance action, caller is none
      | name     | from  | to    | from_type |
      | name1.ic | user3 | user2 | principal |
    When last registrar allowance result is err, expected err is "Other" and message is "owner is invalid"

  Scenario: Ext allowance failed account id not supported
    Given registrar allowance action, caller is none
      | name     | from                                                             | to    | from_type |
      | name1.ic | 3352b4176f9818dfa25c862cbca82f0f05b8e150dded0263e2ef05b094103e34 | user2 | address   |
    When last registrar allowance result is err, expected err is "Other" and message is "account identifier is not supported"

  Scenario: Import token id from registration
    Given registrar import token id from registration
    When last registrar import token id from registration result is ok, and value is "0"
