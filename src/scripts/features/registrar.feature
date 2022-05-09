@registrar
Feature: Query Api

  Background:
    Given Reinstall registrar related canisters

  Scenario Outline: Test state
    Given Check availability of "<name>"
    Given Check result of "<name>" is '<status>'
    Examples: Rainbow colours
      | name                                                                            | status                                                                       |
      | hello.ark                                                                       | Ok                                                                           |
      | 012345678901234567890123456789012345678901234567890123456789012345678912345.ark | name is invalid, reason: "second level name must be less than 64 characters" |
      | www.hello.ark                                                                   | name is invalid, reason: "it must be second level name"                      |
      | icp                                                                             | name is invalid, reason: "it must be second level name"                      |
      | hello.com                                                                       | name is invalid, reason: "top level of name must be ark"                     |
      | hel!lo.ark                                                                      | name is invalid, reason: "name must be alphanumeric or -"                    |
      | hello .ark                                                                      | name is invalid, reason: "name must be alphanumeric or -"                    |
      | 你好.ark                                                                         | name is invalid, reason: "name must be alphanumeric or -"                   |
      | icp.ark                                                                         | Registration has been taken                                                  |

  Scenario: Check availability of a name which is already taken
    Given Name "hello.ark" is already taken
    When Check availability of "hello.ark"
    Then Check result of "hello.ark" is 'Registration has been taken'

  Scenario: Get details of a name
    When Name "hello.ark" is already taken
    And get_owner result "hello.ark" is the same as "main" identity
    Then get_name_expires "hello.ark" result is about in "1" years
    And registrar get_details "hello.ark" result is
      | key        | value     |
      | owner      | main      |
      | name       | hello.ark |
      | expired_at | 1         |
      | created_at | 0         |
