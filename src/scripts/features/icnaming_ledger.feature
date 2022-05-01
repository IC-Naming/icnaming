@icnaming_ledger
Feature: ICNaming Ledger

  Background:
    Given Reinstall canisters
      | name            |
      | icnaming_ledger |
      | ledger          |

  Scenario: Withdraw ICP Success
    Given ICNaming Ledger RECEIVE_SUBACCOUNT have balance to "123 icp"
    And ICNaming Ledger REFUND_SUBACCOUNT have balance to "456 icp"
    When Withdraw ICP from ICNaming Ledger RECEIVE_SUBACCOUNT with "122 icp"
    And Withdraw ICP from ICNaming Ledger REFUND_SUBACCOUNT with "455 icp"
    Then ICP Receiver account balance is "577 icp"
    And ICNaming Ledger RECEIVE_SUBACCOUNT balance is "0.9999 icp"
    And ICNaming Ledger REFUND_SUBACCOUNT balance is "0.9999 icp"
