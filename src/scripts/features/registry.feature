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