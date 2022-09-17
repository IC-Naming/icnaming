@registrar
Feature: EXT token standard query API

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


  Scenario: Metadata
    When registrar metadata "name1.ic" result is
      | key  | value    |
      | name | name1.ic |
    And registrar metadata "na2.ic" result is
      | key  | value  |
      | name | na2.ic |
    And registrar metadata "iiiiii.ic" result is
      | key  | value     |
      | name | iiiiii.ic |


  Scenario: getTokens
    When registrar getTokens result is
      | index | key  | value     |
      | 1     | name | name1.ic  |
      | 2     | name | na2.ic    |
      | 3     | name | iiiiii.ic |

  Scenario: getRegistry
    When registrar getRegistry result is
      | index | name  |
      | 1     | user1 |
      | 2     | user1 |
      | 3     | user2 |

  Scenario: supply
    When registrar supply result is "3"

  Scenario: bearer
    When registrar bearer result is
      | name      | user  |
      | name1.ic  | user1 |
      | na2.ic    | user1 |
      | iiiiii.ic | user2 |

  Scenario: ext_tokens_of
    When registrar ext_tokens_of "user1" result is
      | index |
      | 1     |
      | 2     |

  Scenario: ext_batch_tokens_of
    When registrar ext_batch_tokens_of result is
      | user  | index |
      | user1 | 1     |
      | user1 | 2     |
      | user2 | 3     |
