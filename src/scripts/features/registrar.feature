@registrar
Feature: Query Api

  Background:
    Given Reinstall registrar related canisters

  Scenario Outline: Test state
    Given Check availability of "<name>"
    Given Check result of "<name>" is '<status>'
    Examples: Rainbow colours
      | name                                                                            | status                                                                       |
      | hello.icp                                                                       | Ok                                                                           |
      | 012345678901234567890123456789012345678901234567890123456789012345678912345.icp | name is invalid, reason: "second level name must be less than 64 characters" |
      | www.hello.icp                                                                   | name is invalid, reason: "it must be second level name"                      |
      | icp                                                                             | name is invalid, reason: "it must be second level name"                      |
      | hello.com                                                                       | name is invalid, reason: "top level of name must be icp"                     |
      | hel!lo.icp                                                                      | name is invalid, reason: "name must be alphanumeric or -"                    |
      | hello .icp                                                                      | name is invalid, reason: "name must be alphanumeric or -"                    |
      | 你好.icp                                                                          | name is invalid, reason: "name must be alphanumeric or -"                    |
      | icp.icp                                                                         | Registration has been taken                                                  |

  Scenario: Check availability of a name which is already taken
    Given Name "hello.icp" is already taken
    When Check availability of "hello.icp"
    Then Check result of "hello.icp" is 'Registration has been taken'

  Scenario: Get details of a name
    When Name "hello.icp" is already taken
    And get_owner result "hello.icp" is the same as "main" identity
    Then get_name_expires "hello.icp" result is about in "1" years
    And get_details "hello.icp" result is
      | key        | value     |
      | owner      | main      |
      | name       | hello.icp |
      | expired_at | 1         |
      | created_at | 0         |
