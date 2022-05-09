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
    When admin assign name "icnaming.ark" to user "user1"
    And admin assign name "icp.ark" to user "user2"
    Then last assign name status is "Ok"
    And registrar get_details "icnaming.ark" result is
      | key        | value        |
      | owner      | user1        |
      | name       | icnaming.ark |
      | expired_at | 1            |
      | created_at | 0            |
    And registrar get_details "icp.ark" result is
      | key        | value   |
      | owner      | user2   |
      | name       | icp.ark |
      | expired_at | 1       |
      | created_at | 0       |
    And User quota status should be as below
      | user                      | quota_type1 | quota_type2 | value |
      | registrar_control_gateway | LenGte      | 1           | 8     |

  Scenario: Assign a name more than once, and owner should be the first one
    Given Update quota as follow operations
      | operation | user                      | quota_type1 | quota_type2 | value |
      | add       | registrar_control_gateway | LenGte      | 1           | 10    |
    And admin assign name "icnaming.ark" to user "user1"
    And last assign name status is "Ok"
    When admin assign name "icnaming.ark" to user "user2"
    Then last assign name status is "AlreadyAssigned"
    And registrar get_details "icnaming.ark" result is
      | key        | value        |
      | owner      | user1        |
      | name       | icnaming.ark |
      | expired_at | 1            |
      | created_at | 0            |
    Then User quota status should be as below
      | user                      | quota_type1 | quota_type2 | value |
      | registrar_control_gateway | LenGte      | 1           | 9     |

  Scenario: Assign a name without quota result in fail
    Given admin assign name "icnaming.ark" to user "user1"
    And last assign name status is "FailFromRegistrar"

  Scenario: Assign a reserved name success and transfer by admin to another user
    Given Update quota as follow operations
      | operation | user                      | quota_type1 | quota_type2 | value |
      | add       | registrar_control_gateway | LenGte      | 1           | 10    |
    And admin assign name "icnaming.ark" to user "user1"
    When User "main" transfer name "icnaming.ark" to user "user3"
    Then last transfer_by_admin status is "Ok"
    And registrar get_details "icnaming.ark" result is
      | key        | value        |
      | owner      | user3        |
      | name       | icnaming.ark |
      | expired_at | 1            |
      | created_at | 0            |

  Scenario: Failed to transfer_by_admin if name is not assigned
    When User "main" transfer name "icnaming.ark" to user "user3"
    Then last transfer_by_admin status is "Registration is not found"

  Scenario: Failed to transfer_by_admin if is not admin
    Given Update quota as follow operations
      | operation | user                      | quota_type1 | quota_type2 | value |
      | add       | registrar_control_gateway | LenGte      | 1           | 10    |
    And admin assign name "icnaming.ark" to user "user1"
    When User "user1" transfer name "icnaming.ark" to user "user3"
    Then last transfer_by_admin status is "Unauthorized, please login first"