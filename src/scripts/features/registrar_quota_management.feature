@registrar
Feature: Quota Management

  Background:
    Given Reinstall canisters
      | name      |
      | registrar |

  Scenario: Update user quota
    When Update quota as follow operations
      | operation | user  | quota_type1 | quota_type2 | value |
      | add       | user1 | LenGte      | 3           | 10    |
      | add       | user1 | LenGte      | 4           | 10    |
      | add       | user1 | LenEq       | 5           | 10    |
      | add       | user2 | LenGte      | 3           | 10    |
      | sub       | user1 | LenGte      | 3           | 4     |
    Then User quota status should be as below
      | user  | quota_type1 | quota_type2 | value |
      | user1 | LenGte      | 3           | 6     |
      | user1 | LenGte      | 4           | 10    |
      | user1 | LenEq       | 5           | 10    |
      | user2 | LenGte      | 3           | 10    |

  Scenario: Import quota
    When admin import quota file "20220223_astrox_me_event.zlib"
    Then Last quota import status "true"
    And User quota status should be as below
      | user                        | quota_type1 | quota_type2 | value |
      | 64l4r-aaaaa-aaaah-aaklq-cai | LenGte      | 5           | 300   |
      | 64l4r-aaaaa-aaaah-aaklq-cai | LenGte      | 6           | 400   |
      | 64l4r-aaaaa-aaaah-aaklq-cai | LenGte      | 7           | 1600  |
      | 64l4r-aaaaa-aaaah-aaklq-cai | LenGte      | 8           | 660   |

  Scenario: Import quota duplicated
    When admin import quota file "20220223_astrox_me_event.zlib"
    And admin import quota file "20220223_astrox_me_event.zlib"
    Then Last quota import status "false"
    Then User quota status should be as below
      | user                        | quota_type1 | quota_type2 | value |
      | 64l4r-aaaaa-aaaah-aaklq-cai | LenGte      | 5           | 300   |
      | 64l4r-aaaaa-aaaah-aaklq-cai | LenGte      | 6           | 400   |
      | 64l4r-aaaaa-aaaah-aaklq-cai | LenGte      | 7           | 1600  |
      | 64l4r-aaaaa-aaaah-aaklq-cai | LenGte      | 8           | 660   |

  Scenario: Import large file
    When admin import quota file "20220223_twitter_retweet.zlib"
    Then Last quota import status "true"
    Then User quota status should be as below
      | user                                                            | quota_type1 | quota_type2 | value |
      | qhl7u-fzmv5-2znja-t4zlh-lcde7-rbvbo-fqtlu-izncn-6f6gm-6gg52-vae | LenGte      | 7           | 2     |
      | hnja5-kwd4y-azcdv-fzsd2-rounm-myhqo-cedgb-ne5e6-jpk7q-ymkec-7ae | LenGte      | 7           | 1     |

  Scenario: Import illegal data
    When admin import quota file "illegal.zlib"
    Then Last quota import status "false"
