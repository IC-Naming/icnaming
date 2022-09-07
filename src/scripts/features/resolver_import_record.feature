@resolver
Feature: Resolver import Api

  Background:
    Given Reinstall registrar related canisters
    And Name "hello.ic" is already taken
    And Name "wonderful.ic" is already taken


  Scenario: Import resolver record
    When import_record_value
      | name         | operation      | key                                   | value                                                           |
      | hello.ic     | Upsert         | principal.icp                         | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe |
      | wonderful.ic | InsertOrIgnore | settings.reverse_resolution.principal | xat7x-vbdo7-g6upd-ko36c-wa4f3-2ni3u-476z3-66eyd-hxmi3-mvsgo-mae |
    Then batch check record_value
      | name         | key                                   | value                                                           |
      | hello.ic     | principal.icp                         | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe |
      | wonderful.ic | settings.reverse_resolution.principal | xat7x-vbdo7-g6upd-ko36c-wa4f3-2ni3u-476z3-66eyd-hxmi3-mvsgo-mae |

  Scenario: Import resolver record, insert should ignore
    When import_record_value
      | name     | operation      | key           | value                                                           |
      | hello.ic | Upsert         | principal.icp | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe |
      | hello.ic | InsertOrIgnore | principal.icp | xat7x-vbdo7-g6upd-ko36c-wa4f3-2ni3u-476z3-66eyd-hxmi3-mvsgo-mae |
    Then batch check record_value
      | name     | key           | value                                                           |
      | hello.ic | principal.icp | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe |

  Scenario: Import resolver record, remove success
    When import_record_value
      | name     | operation      | key           | value                                                           |
      | hello.ic | Upsert         | principal.icp | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe |
      | hello.ic | InsertOrIgnore | principal.icp | xat7x-vbdo7-g6upd-ko36c-wa4f3-2ni3u-476z3-66eyd-hxmi3-mvsgo-mae |
      | hello.ic | Remove         | principal.icp | xat7x-vbdo7-g6upd-ko36c-wa4f3-2ni3u-476z3-66eyd-hxmi3-mvsgo-mae |
    Then batch check record_value should not in
      | name     | key           |
      | hello.ic | principal.icp |


  Scenario: Import resolver record, remove primary name by name success
    When import_record_value
      | name     | operation | key                                   | value |
      | hello.ic | Remove    | settings.reverse_resolution.principal |       |
    Then batch check record_value should not in
      | name     | key                                   |
      | hello.ic | settings.reverse_resolution.principal |
