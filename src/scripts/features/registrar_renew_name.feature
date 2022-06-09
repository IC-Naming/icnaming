@registrar
Feature: Renew a name

  Background:
    Given Reinstall registrar related canisters
    And Some users already have some quotas
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 10    |
    And User "user1" balance is set to be "10 icp"


  Scenario: Renew a name successfully
    Given User "user1" register name "7654321.ic" with quote "LenGte(3)"
    And registrar get_details "7654321.ic" result is
      | key        | value       |
      | owner      | user1       |
      | name       | 7654321.ic |
      | expired_at | 1           |
      | created_at | 0           |
    When User "user1" renew name "7654321.ic" for "3" years and pay "3 icp"
    Then Last renew name status is "Ok"
    And registrar get_details "7654321.ic" result is
      | key        | value       |
      | owner      | user1       |
      | name       | 7654321.ic |
      | expired_at | 4           |
      | created_at | 0           |

  Scenario: Renew a name successfully for 5-chars name
    Given User "user1" register name "hello.ic" with quote "LenGte(3)"
    And registrar get_details "hello.ic" result is
      | key        | value     |
      | owner      | user1     |
      | name       | hello.ic |
      | expired_at | 1         |
      | created_at | 0         |
    When User "user1" renew name "hello.ic" for "3" years and pay "7.26 icp"
    Then Last renew name status is "Ok"
    And registrar get_details "hello.ic" result is
      | key        | value     |
      | owner      | user1     |
      | name       | hello.ic |
      | expired_at | 4         |
      | created_at | 0         |

  Scenario: Renew a name success with 95% lower price
    Given User "user1" register name "7654321.ic" with quote "LenGte(3)"
    And registrar get_details "7654321.ic" result is
      | key        | value       |
      | owner      | user1       |
      | name       | 7654321.ic |
      | expired_at | 1           |
      | created_at | 0           |
    When User "user1" renew name "7654321.ic" for "5" years and pay "4.75 icp"
    Then Last renew name status is "Ok"
    And registrar get_details "7654321.ic" result is
      | key        | value       |
      | owner      | user1       |
      | name       | 7654321.ic |
      | expired_at | 6           |
      | created_at | 0           |

  Scenario: Renew a name failed since too many years
    Given User "user1" register name "7654321.ic" with quote "LenGte(3)"
    And registrar get_details "7654321.ic" result is
      | key        | value       |
      | owner      | user1       |
      | name       | 7654321.ic |
      | expired_at | 1           |
      | created_at | 0           |
    When User "user1" renew name "7654321.ic" for "11" years and pay "11 icp"
    Then Last renew name status is "it is not allowed to renew the name more than 10 years"

  Scenario: Renew a name failed since not enough money
    Given User "user1" register name "7654321.ic" with quote "LenGte(3)"
    And registrar get_details "7654321.ic" result is
      | key        | value       |
      | owner      | user1       |
      | name       | 7654321.ic |
      | expired_at | 1           |
      | created_at | 0           |
    When User "user1" renew name "7654321.ic" for "10" years and pay "3 icp"
    Then Last renew name status is "price changed, please refresh and try again"

  Scenario: Renew a name failed since caller is not owner
    Given User "user1" register name "7654321.ic" with quote "LenGte(3)"
    And registrar get_details "7654321.ic" result is
      | key        | value       |
      | owner      | user1       |
      | name       | 7654321.ic |
      | expired_at | 1           |
      | created_at | 0           |
    And User "user2" balance is set to be "10 icp"
    When User "user2" renew name "7654321.ic" for "5" years and pay "5 icp"
    Then Last renew name status is "owner is invalid"
