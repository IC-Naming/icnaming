@registrar
Feature: Quota Management

  Background:
    Given Reinstall canisters
      | name      |
      | registrar |

  Scenario: Update user quota
    When Update quota as follow operations
      | operation | user  | quota_type1 | quota_type2 | value |
      | add       | user1 | LenGte      | 3           | 10    |
      | add       | user1 | LenGte      | 4           | 10    |
      | add       | user1 | LenEq       | 5           | 10    |
      | add       | user2 | LenGte      | 3           | 10    |
      | sub       | user1 | LenGte      | 3           | 4     |
    Then User quota status should be as below
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 6     |
      | user1 | LenGte      | 4           | 10    |
      | user1 | LenEq       | 5           | 10    |
      | user2 | LenGte      | 3           | 10    |

