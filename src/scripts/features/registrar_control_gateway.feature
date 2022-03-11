@registrar @registrar_control_gateway
Feature: Registrar Control Gateway

  Background:
    Given Reinstall canisters
      | name                      |
      | registrar                 |
      | registry                  |
      | resolver                  |
      | registrar_control_gateway |

  Scenario: Import quota
    When admin import quota file "20220310_reserved_name_assignment.zlib"
    Then Last quota import status "Ok"
    And User quota status should be as below
      | user                        | quota_type1 | quota_type2 | value |
      | gjzpj-bqaaa-aaaam-aacya-cai | LenGte      | 1           | 41    |

  Scenario: Import quota duplicated
    When admin import quota file "20220310_reserved_name_assignment.zlib"
    And admin import quota file "20220310_reserved_name_assignment.zlib"
    Then Last quota import status "AlreadyExists"
    And User quota status should be as below
      | user                        | quota_type1 | quota_type2 | value |
      | gjzpj-bqaaa-aaaam-aacya-cai | LenGte      | 1           | 41    |

  Scenario: Import illegal data
    When admin import quota file "illegal.zlib"
    Then Last quota import status "InvalidRequest"

  Scenario: Assign a reserved name success
    Given Update quota as follow operations
      | operation | user                      | quota_type1 | quota_type2 | value |
      | add       | registrar_control_gateway | LenGte      | 1           | 10    |
    And admin assign name "icnaming.icp" to user "user1"
    And admin assign name "icp.icp" to user "user2"
    Then last assign name status is "Ok"
    And get_details "icnaming.icp" result is
      | key        | value        |
      | owner      | user1        |
      | name       | icnaming.icp |
      | expired_at | 1            |
      | created_at | 0            |
    And get_details "icp.icp" result is
      | key        | value   |
      | owner      | user2   |
      | name       | icp.icp |
      | expired_at | 1       |
      | created_at | 0       |
    Then User quota status should be as below
      | user                      | quota_type1 | quota_type2 | value |
      | registrar_control_gateway | LenGte      | 1           | 8     |

  Scenario: Assign a name more than once, and owner should be the first one
    Given Update quota as follow operations
      | operation | user                      | quota_type1 | quota_type2 | value |
      | add       | registrar_control_gateway | LenGte      | 1           | 10    |
    And admin assign name "icnaming.icp" to user "user1"
    And last assign name status is "Ok"
    When admin assign name "icnaming.icp" to user "user2"
    Then last assign name status is "AlreadyAssigned"
    And get_details "icnaming.icp" result is
      | key        | value        |
      | owner      | user1        |
      | name       | icnaming.icp |
      | expired_at | 1            |
      | created_at | 0            |
    Then User quota status should be as below
      | user                      | quota_type1 | quota_type2 | value |
      | registrar_control_gateway | LenGte      | 1           | 9     |

  Scenario: Assign a name without quota result in fail
    Given admin assign name "icnaming.icp" to user "user1"
    And last assign name status is "FailFromRegistrar"
