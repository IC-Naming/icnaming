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

  Scenario: Transfer quota to user 1
    Given Update quota as follow operations
      | operation | user  | quota_type1 | quota_type2 | value |
      | add       | user1 | LenGte      | 3           | 10    |
    When Do quota transfer as below
      | from  | to    | quota_type1 | quota_type2 | value |
      | user1 | user2 | LenGte      | 3           | 4     |
    Then User quota status should be as below
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 6     |
      | user2 | LenGte      | 3           | 4     |

  Scenario: Transfer quota to user 2
    Given Update quota as follow operations
      | operation | user  | quota_type1 | quota_type2 | value |
      | add       | user1 | LenGte      | 3           | 10    |
      | add       | user2 | LenGte      | 3           | 20    |
    When Do quota transfer as below
      | from  | to    | quota_type1 | quota_type2 | value |
      | user1 | user2 | LenGte      | 3           | 4     |
    Then User quota status should be as below
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 6     |
      | user2 | LenGte      | 3           | 24    |

  Scenario: Transfer quota failed 1
    Given Update quota as follow operations
      | operation | user  | quota_type1 | quota_type2 | value |
      | add       | user1 | LenGte      | 3           | 10    |
    When Do quota transfer as below
      | from  | to    | quota_type1 | quota_type2 | value |
      | user1 | user2 | LenGte      | 3           | 11    |
    Then User quota status should be as below
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 10    |

  Scenario: Transfer quota failed 2
    When Do quota transfer as below
      | from  | to    | quota_type1 | quota_type2 | value |
      | user1 | user2 | LenGte      | 3           | 1     |
    Then User quota status should be as below
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 0     |

  Scenario: Transfer quota by transfer_from_quota
    Given Update quota as follow operations
      | operation | user  | quota_type1 | quota_type2 | value |
      | add       | user1 | LenGte      | 3           | 10    |
    When Do quota transfer_from_quota as below by admin
      | from  | to    | quota_type1 | quota_type2 | value |
      | user1 | user2 | LenGte      | 3           | 4     |
    Then User quota status should be as below
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 6     |
      | user2 | LenGte      | 3           | 4     |

  Scenario: Transfer quota bu transfer_from_quota failed 1
    Given Update quota as follow operations
      | operation | user  | quota_type1 | quota_type2 | value |
      | add       | user1 | LenGte      | 3           | 10    |
    When Do quota transfer_from_quota as below by admin
      | from  | to    | quota_type1 | quota_type2 | value |
      | user1 | user2 | LenGte      | 3           | 11    |
    Then User quota status should be as below
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 10    |
      | user2 | LenGte      | 3           | 0     |

  Scenario: Batch transfer quota success
    Given Update quota as follow operations
      | operation | user  | quota_type1 | quota_type2 | value |
      | add       | user1 | LenGte      | 3           | 10    |
      | add       | user1 | LenGte      | 4           | 10    |
      | add       | user2 | LenGte      | 3           | 10    |
    When User "user1" transfer quota as below by batch
      | to    | quota_type1 | quota_type2 | diff |
      | user2 | LenGte      | 3           | 5    |
      | user2 | LenGte      | 4           | 6    |
    Then User quota status should be as below
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 5     |
      | user1 | LenGte      | 4           | 4     |
      | user2 | LenGte      | 3           | 15    |
      | user2 | LenGte      | 4           | 6     |

  Scenario: Batch transfer quota success 2
    Given Update quota as follow operations
      | operation | user  | quota_type1 | quota_type2 | value |
      | add       | user1 | LenGte      | 3           | 10    |
      | add       | user1 | LenGte      | 4           | 20    |
      | add       | user2 | LenGte      | 3           | 10    |
    When User "user1" transfer quota as below by batch
      | to    | quota_type1 | quota_type2 | diff |
      | user2 | LenGte      | 3           | 10   |
      | user2 | LenGte      | 4           | 20   |
    Then User quota status should be as below
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 0     |
      | user1 | LenGte      | 4           | 0     |
      | user2 | LenGte      | 3           | 20    |
      | user2 | LenGte      | 4           | 20    |

  Scenario: Batch transfer quota failed 1
    Given Update quota as follow operations
      | operation | user  | quota_type1 | quota_type2 | value |
      | add       | user1 | LenGte      | 3           | 10    |
      | add       | user1 | LenGte      | 4           | 10    |
      | add       | user2 | LenGte      | 3           | 10    |
    When User "user1" transfer quota as below by batch
      | to    | quota_type1 | quota_type2 | diff |
      | user2 | LenGte      | 3           | 11   |
      | user2 | LenGte      | 4           | 6    |
    Then User quota status should be as below
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 10    |
      | user1 | LenGte      | 4           | 10    |
      | user2 | LenGte      | 3           | 10    |

  Scenario: Batch transfer quota failed 1
    Given Update quota as follow operations
      | operation | user  | quota_type1 | quota_type2 | value |
      | add       | user1 | LenGte      | 3           | 10    |
      | add       | user1 | LenGte      | 4           | 10    |
      | add       | user2 | LenGte      | 3           | 10    |
    When User "user1" transfer quota as below by batch
      | to    | quota_type1 | quota_type2 | diff |
      | user2 | LenGte      | 3           | 10   |
      | user2 | LenGte      | 5           | 6    |
    Then User quota status should be as below
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 10    |
      | user1 | LenGte      | 4           | 10    |
      | user2 | LenGte      | 3           | 10    |
