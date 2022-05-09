@favorites
Feature: Favorites Api

  Background:
    Given Reinstall canisters
      | name      |
      | favorites |

  Scenario: Add a favorite
    When User "user1" add some favorites
      | name      |
      | hello.ark |
    And User "user2" add some favorites
      | name       |
      | hello1.ark |
    Then User "user1" should see the favorites
      | name      |
      | hello.ark |
    And User "user2" should see the favorites
      | name       |
      | hello1.ark |

  Scenario: Delete a favorite
    Given User "user1" add some favorites
      | name      |
      | hello.ark |
      | icp.ark   |
    And User "user1" should see the favorites
      | name      |
      | hello.ark |
      | icp.ark   |
    When User "user1" delete a favorite "hello.ark"
    Then User "user1" should see the favorites
      | name    |
      | icp.ark |
