@favorites
Feature: Favorites Api

  Background:
    Given Reinstall canisters
      | name      |
      | favorites |

  Scenario: Add a favorite
    When User "user1" add some favorites
      | name      |
      | hello.icp |
    And User "user2" add some favorites
      | name       |
      | hello1.icp |
    Then User "user1" should see the favorites
      | name      |
      | hello.icp |
    And User "user2" should see the favorites
      | name       |
      | hello1.icp |

  Scenario: Delete a favorite
    Given User "user1" add some favorites
      | name      |
      | hello.icp |
      | icp.icp   |
    And User "user1" should see the favorites
      | name      |
      | hello.icp |
      | icp.icp   |
    When User "user1" delete a favorite "hello.icp"
    Then User "user1" should see the favorites
      | name    |
      | icp.icp |
