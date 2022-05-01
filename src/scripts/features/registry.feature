@registry
Feature: Registry Api

  Background:
    Given Reinstall registrar related canisters
    And Name "hello.icp" is already taken

  Scenario: It is impossible to create a new registry from other principal but registrar
    When I call set_subdomain_owner to add a second level name
    Then set_subdomain_owner result in status "permission deny"

  Scenario: Query default registry values
    Then get_resolver "hello.icp" should be the public resolver
    And get_owner "hello.icp" should be "main"
    And get_ttl "hello.icp" should be "600"
    And registry get_details "hello.icp" should be as below
      | key      | value     |
      | name     | hello.icp |
      | owner    | main      |
      | resolver | public    |
      | ttl      | 600       |

  Scenario: Update registry values
    When I update registry "hello.icp" with values
      | key      | value                       |
      | resolver | qjdve-lqaaa-aaaaa-aaaeq-cai |
      | ttl      | 600                         |
    Then registry get_details "hello.icp" should be as below
      | key      | value                       |
      | name     | hello.icp                   |
      | owner    | main                        |
      | resolver | qjdve-lqaaa-aaaaa-aaaeq-cai |
      | ttl      | 600                         |

  Scenario: Update registry resolver
    When I update registry "hello.icp" resolver to "qjdve-lqaaa-aaaaa-aaaeq-cai"
    Then registry get_details "hello.icp" should be as below
      | key      | value                       |
      | name     | hello.icp                   |
      | owner    | main                        |
      | resolver | qjdve-lqaaa-aaaaa-aaaeq-cai |
      | ttl      | 600                         |

  Scenario: Set owner to another principal successfully
    Given Some users already have some quotas
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 10    |
    And User "user1" register name "nice-name.icp" with quote "LenGte(3)"
    And registry get_details "nice-name.icp" should be as below
      | key      | value         |
      | name     | nice-name.icp |
      | owner    | user1         |
      | resolver | public        |
      | ttl      | 600           |
    When User "user1" set registry owner for "nice-name.icp" to "user2"
    Then registry get_details "nice-name.icp" should be as below
      | key      | value         |
      | name     | nice-name.icp |
      | owner    | user2         |
      | resolver | public        |
      | ttl      | 600           |

  Scenario: Fail to update owner if the name is not owned by the user
    Given Some users already have some quotas
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 10    |
    And User "user1" register name "nice-name.icp" with quote "LenGte(3)"
    When User "user2" set registry owner for "nice-name.icp" to "user3"
    Then registry get_details "nice-name.icp" should be as below
      | key      | value         |
      | name     | nice-name.icp |
      | owner    | user1         |
      | resolver | public        |
      | ttl      | 600           |
