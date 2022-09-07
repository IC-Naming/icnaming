@resolver
Feature: Resolver import Api

  Background: Registry name and auto resolver
    Given Reinstall registrar related canisters
    And Name "hello.ic" is already taken
    And Name "wonderful.ic" is already taken

  Scenario: Import resolver record, upsert
    When import_record_value
      | name         | operation      | key                                   | value                                                            |
      | hello.ic     | Upsert         | principal.icp                         | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe  |
      | hello.ic     | Upsert         | Unknown                               | Unknown                                                          |
      | hello.ic     | Upsert         | account_id.icp                        | 3445e6cc1bb5397fd89fd1e81090f09541923359bc37fab92c29873b168ba70e |
      | wonderful.ic | InsertOrIgnore | settings.reverse_resolution.principal | xat7x-vbdo7-g6upd-ko36c-wa4f3-2ni3u-476z3-66eyd-hxmi3-mvsgo-mae  |
      | wonderful.ic | Upsert         | account_id.icp                        | 5445e6cc1bb5397fd89fd1e81090f09541923359bc37fab92c29873b168ba70e |
    Then check import_record_value response is ok
    And batch check record_value
      | name         | key                                   | value                                                            |
      | hello.ic     | principal.icp                         | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe  |
      | hello.ic     | Unknown                               | Unknown                                                          |
      | hello.ic     | account_id.icp                        | 3445e6cc1bb5397fd89fd1e81090f09541923359bc37fab92c29873b168ba70e |
      | wonderful.ic | settings.reverse_resolution.principal | xat7x-vbdo7-g6upd-ko36c-wa4f3-2ni3u-476z3-66eyd-hxmi3-mvsgo-mae  |
      | wonderful.ic | account_id.icp                        | 5445e6cc1bb5397fd89fd1e81090f09541923359bc37fab92c29873b168ba70e |

  Scenario: Import resolver record, insert should ignore
    When import_record_value
      | name         | operation      | key                                   | value                                                            |
      | hello.ic     | Upsert         | principal.icp                         | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe  |
      | hello.ic     | InsertOrIgnore | principal.icp                         | xat7x-vbdo7-g6upd-ko36c-wa4f3-2ni3u-476z3-66eyd-hxmi3-mvsgo-mae  |
      | hello.ic     | Upsert         | account_id.icp                        | 3445e6cc1bb5397fd89fd1e81090f09541923359bc37fab92c29873b168ba70e |
      | hello.ic     | InsertOrIgnore | account_id.icp                        | 5445e6cc1bb5397fd89fd1e81090f09541923359bc37fab92c29873b168ba70e |
      | hello.ic     | Upsert         | Unknown                               | Unknown1                                                         |
      | hello.ic     | InsertOrIgnore | Unknown                               | Unknown2                                                         |
      | wonderful.ic | Upsert         | settings.reverse_resolution.principal | xat7x-vbdo7-g6upd-ko36c-wa4f3-2ni3u-476z3-66eyd-hxmi3-mvsgo-mae  |
      | wonderful.ic | InsertOrIgnore | settings.reverse_resolution.principal | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe  |
    Then check import_record_value response is ok
    And batch check record_value
      | name         | key                                   | value                                                            |
      | hello.ic     | principal.icp                         | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe  |
      | hello.ic     | account_id.icp                        | 3445e6cc1bb5397fd89fd1e81090f09541923359bc37fab92c29873b168ba70e |
      | hello.ic     | Unknown                               | Unknown1                                                         |
      | wonderful.ic | settings.reverse_resolution.principal | xat7x-vbdo7-g6upd-ko36c-wa4f3-2ni3u-476z3-66eyd-hxmi3-mvsgo-mae  |

  Scenario: Import resolver record, remove success
    When import_record_value
      | name         | operation      | key                                   | value                                                           |
      | hello.ic     | Upsert         | principal.icp                         | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe |
      | hello.ic     | InsertOrIgnore | principal.icp                         | xat7x-vbdo7-g6upd-ko36c-wa4f3-2ni3u-476z3-66eyd-hxmi3-mvsgo-mae |
      | hello.ic     | Upsert         | Unknown                               | Unknown1                                                        |
      | hello.ic     | Upsert         | Unknown                               | Unknown2                                                        |
      | wonderful.ic | Upsert         | settings.reverse_resolution.principal | xat7x-vbdo7-g6upd-ko36c-wa4f3-2ni3u-476z3-66eyd-hxmi3-mvsgo-mae |
      | wonderful.ic | InsertOrIgnore | settings.reverse_resolution.principal | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe |
      | hello.ic     | Remove         | principal.icp                         |                                                                 |
      | hello.ic     | Remove         | Unknown                               |                                                                 |
      | wonderful.ic | Remove         | settings.reverse_resolution.principal |                                                                 |
    Then check import_record_value response is ok
    And batch check record_value should not in
      | name         | key                                   |
      | hello.ic     | principal.icp                         |
      | hello.ic     | Unknown                               |
      | wonderful.ic | settings.reverse_resolution.principal |


  Scenario: Import resolver record, will not be updated if an error occurs
    Given import_record_value
      | name         | operation      | key                                   | value                                                            |
      | hello.ic     | Upsert         | principal.icp                         | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe  |
      | hello.ic     | Upsert         | Unknown                               | Unknown                                                          |
      | hello.ic     | Upsert         | account_id.icp                        | 3445e6cc1bb5397fd89fd1e81090f09541923359bc37fab92c29873b168ba70e |
      | wonderful.ic | InsertOrIgnore | settings.reverse_resolution.principal | xat7x-vbdo7-g6upd-ko36c-wa4f3-2ni3u-476z3-66eyd-hxmi3-mvsgo-mae  |
      | wonderful.ic | Upsert         | account_id.icp                        | 5445e6cc1bb5397fd89fd1e81090f09541923359bc37fab92c29873b168ba70e |
    When import_record_value
      | name         | operation | key                                   | value                                                            |
      | wonderful.ic | Upsert    | account_id.icp                        | 5445e6cc1bb5397fd89fd1e81090f09541923359bc37fab92c29873b168ba70e |
      | hello.ic     | Upsert    | settings.reverse_resolution.principal | 5445e6cc1bb5397fd89fd1e81090f09541923359bc37fab92c29873b168ba70e |
      | hello.ic     | Upsert    | Unknown                               | Unknown2                                                         |
    Then check import_record_value response is error, expect message contains "invalid resolver value format for"
    And batch check record_value
      | name         | key                                   | value                                                            |
      | hello.ic     | principal.icp                         | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe  |
      | hello.ic     | Unknown                               | Unknown                                                          |
      | hello.ic     | account_id.icp                        | 3445e6cc1bb5397fd89fd1e81090f09541923359bc37fab92c29873b168ba70e |
      | wonderful.ic | settings.reverse_resolution.principal | xat7x-vbdo7-g6upd-ko36c-wa4f3-2ni3u-476z3-66eyd-hxmi3-mvsgo-mae  |
      | wonderful.ic | account_id.icp                        | 5445e6cc1bb5397fd89fd1e81090f09541923359bc37fab92c29873b168ba70e |


  Scenario: Import resolver record, error key too long
    When import_record_value
      | name         | operation      | key                                                               | value                                                            |
      | hello.ic     | Upsert         | Unknown                                                           | Unknown                                                          |
      | hello.ic     | Upsert         | account_id.icp                                                    | 3445e6cc1bb5397fd89fd1e81090f09541923359bc37fab92c29873b168ba70e |
      | wonderful.ic | InsertOrIgnore | settings.reverse_resolution.principal                             | xat7x-vbdo7-g6upd-ko36c-wa4f3-2ni3u-476z3-66eyd-hxmi3-mvsgo-mae  |
      | wonderful.ic | Upsert         | account_id.icp                                                    | 5445e6cc1bb5397fd89fd1e81090f09541923359bc37fab92c29873b168ba70e |
      | hello.ic     | Upsert         | 11111111111111111111111111111111111111111111111111111111111111111 | 2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe  |
    Then check import_record_value response is error, expect message contains "Length of key must be less than 64"

  Scenario: Import resolver record, error value too long
    When import_record_value, value len is "513"
      | name     | operation | key     |
      | hello.ic | Upsert    | Unknown |
    Then check import_record_value response is error, expect message contains "Length of value must be less than 512"
