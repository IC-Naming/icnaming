@resolver
Feature: Resolver import Api

  Background:
    Given Reinstall registrar related canisters
    And Name "hello.ic" is already taken
    And Name "wonderful.ic" is already taken


  Scenario: Query default resolver values
    Then get_record_value "hello.ic" should be as below
      | name     | opt            | key           | value                                                           |
      | hello.ic | insert         | principal.icp | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe |
      | hello.ic | insertOrIgnore | principal.icp | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe |
      | hello.ic | delete         | principal.icp | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe |
