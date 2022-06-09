@favorites
Feature: Favorites Api

  Background:
    Given Reinstall canisters
      | name      |
      | favorites |

  Scenario: Add a favorite
    When User "user1" add some favorites
      | name      |
      | hello.ic |
    And User "user2" add some favorites
      | name       |
      | hello1.ic |
    Then User "user1" should see the favorites
      | name      |
      | hello.ic |
    And User "user2" should see the favorites
      | name       |
      | hello1.ic |

  Scenario: Delete a favorite
    Given User "user1" add some favorites
      | name      |
      | hello.ic |
      | icp.ic   |
    And User "user1" should see the favorites
      | name      |
      | hello.ic |
      | icp.ic   |
    When User "user1" delete a favorite "hello.ic"
    Then User "user1" should see the favorites
      | name    |
      | icp.ic |
