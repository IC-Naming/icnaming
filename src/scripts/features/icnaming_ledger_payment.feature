@payments
Feature: ICNaming Ledger Payment

  Background:
    Given Reinstall all canisters

  Scenario: Payment with different memo
    Given Create a payment with amount "1 icp"
    When Transfer to icnaming ledger account with memo "777", amount "1 icp"
    Then Verify payment
    And Payment status is "NeedMore"
    And Payment received_amount is "0"

  Scenario: Payment with one enough transaction
    Given Create a payment with amount "1 icp"
    When Transfer to icnaming ledger account with memo "1", amount "1 icp"
    Then Verify payment
    And Payment status is "Paid"

  Scenario: Payment with not enough transaction
    Given Create a payment with amount "1 icp"
    When Transfer to icnaming ledger account with memo "1", amount "0.2 icp"
    Then Verify payment
    And Payment status is "NeedMore"
    And Payment received_amount is "0.2 icp"

  Scenario: Payment with multiple transactions
    Given Create a payment with amount "1 icp"
    When Transfer to icnaming ledger account with memo "1", amount "0.2 icp"
    And Transfer to icnaming ledger account with memo "1", amount "0.8 icp"
    Then Verify payment
    And Payment status is "Paid"

  Scenario: Payment with over paid
    Given Create a payment with amount "1 icp"
    When Transfer to icnaming ledger account with memo "1", amount "2 icp"
    Then Verify payment
    And Payment status is "Paid"

  Scenario: Payment with refund
    Given User "user1" balance is set to be "1 icp"
    And ICNaming ledger receiver subaccount balance is set to be "0 icp"
    And ICNaming ledger refund subaccount balance is set to be "1 icp"
    And Create a payment with amount "0.4 icp"
    And User "user1" transfer to icnaming ledger account with memo "1", amount "0.4 icp"
    And Sleep for "5" secs.
    When Refund last payment
    Then Refund response status is "Refunded"
    And ICNaming ledger refund subaccount balance is "0.5999 icp"
    And ICNaming ledger receiver subaccount balance is "0.4 icp"
    And User "user1" balance is "0.9999 icp"
    And Verify payment with "PaymentNotFound" result

  Scenario: Refund payment with insufficient refund balance
    Given User "user1" balance is set to be "1 icp"
    And ICNaming ledger receiver subaccount balance is set to be "0 icp"
    And ICNaming ledger refund subaccount balance is set to be "0.3 icp"
    And Create a payment with amount "400_000"
    And User "user1" transfer to icnaming ledger account with memo "1", amount "0.4 icp"
    And Sleep for "5" secs.
    When Refund last payment
    Then Refund response status is "RefundFailed"
    And ICNaming ledger refund subaccount balance is "0.3 icp"
    And ICNaming ledger receiver subaccount balance is "0.4 icp"
    And User "user1" balance is "0.5999 icp"
    And Verify payment with "Paid" result

  Scenario: Refund payment successfully after top up to refund subaccount
    Given User "user1" balance is set to be "1 icp"
    And ICNaming ledger receiver subaccount balance is set to be "0 icp"
    And ICNaming ledger refund subaccount balance is set to be "0.3 icp"
    And Create a payment with amount "0.4 icp"
    And User "user1" transfer to icnaming ledger account with memo "1", amount "0.4 icp"
    And Sleep for "5" secs.
    And Refund last payment
    And Refund response status is "RefundFailed"
    And ICNaming ledger refund subaccount balance is topped up with "1 icp"
    When Refund last payment
    Then Refund response status is "Refunded"
    And ICNaming ledger refund subaccount balance is "0.8999 icp"
    And ICNaming ledger receiver subaccount balance is "0.4 icp"
    And User "user1" balance is "0.9999 icp"
    And Verify payment with "PaymentNotFound" result