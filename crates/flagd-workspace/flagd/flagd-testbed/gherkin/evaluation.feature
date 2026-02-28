@rpc @in-process @file
Feature: flagd evaluations

  # This test suite contains scenarios to test flagd providers.
  # It's associated with the flags configured in flags.
  # It should be used in conjunction with the suites supplied by the OpenFeature specification.

  Background:
    Given an option "cache" of type "CacheType" with value "disabled"
    And a stable flagd provider

  Scenario Outline: Resolve values
    Given a <type>-flag with key "<key>" and a default value "<default>"
    When the flag was evaluated with details
    Then the resolved details value should be "<resolved_value>"

    Examples:
      | key          | type    | default | resolved_value |
      | boolean-flag | Boolean | false   | true           |
      | string-flag  | String  | bye     | hi             |
      | integer-flag | Integer | 1       | 10             |
      | float-flag   | Float   | 0.1     | 0.5            |

  Scenario Outline: Resolves zero value
    Given a <type>-flag with key "<key>" and a default value "<default>"
    When the flag was evaluated with details
    Then the resolved details value should be "<resolved_value>"
    And the reason should be "STATIC"

    Examples:
      | key               | type    | default | resolved_value |
      | boolean-zero-flag | Boolean | true    | false          |
      | string-zero-flag  | String  | hi      |                |
      | integer-zero-flag | Integer | 1       | 0              |
      | float-zero-flag   | Float   | 0.1     | 0.0            |

  @targeting
  Scenario Outline: Resolves zero value with targeting
    Given a <type>-flag with key "<key>" and a default value "<default>"
    And a context containing a key "email", with type "String" and with value "ballmer@macrosoft.com"
    When the flag was evaluated with details
    Then the resolved details value should be "<resolved_value>"
    And the reason should be "TARGETING_MATCH"

    Examples:
      | key                        | type    | default | resolved_value |
      | boolean-targeted-zero-flag | Boolean | true    | false          |
      | string-targeted-zero-flag  | String  | hi      |                |
      | integer-targeted-zero-flag | Integer | 1       | 0              |
      | float-targeted-zero-flag   | Float   | 0.1     | 0.0            |

  @targeting
  Scenario Outline: Resolves zero value with targeting using default
    Given a <type>-flag with key "<key>" and a default value "<default>"
    And a context containing a key "email", with type "String" and with value "ballmer@none.com"
    When the flag was evaluated with details
    Then the resolved details value should be "<resolved_value>"
    And the reason should be "DEFAULT"

    Examples:
      | key                        | type    | default | resolved_value |
      | boolean-targeted-zero-flag | Boolean | true    | false          |
      | string-targeted-zero-flag  | String  | hi      |                |
      | integer-targeted-zero-flag | Integer | 1       | 0              |
      | float-targeted-zero-flag   | Float   | 0.1     | 0.0            |

  @no-default
  Scenario Outline: Resolves flag with no defaultValue correctly
    Given a <type>-flag with key "<key>" and a default value "<default>"
    And a context containing a key "email", with type "String" and with value "<email>"
    When the flag was evaluated with details
    Then the resolved details value should be "<resolved_value>"
    And the reason should be "<reason>"
    And the error-code should be "<error_code>"

    # For now, no defaultValue is resolved as FLAG_NOT_FOUND to result in a code default.
    # This may be handled more gracefully in the future.
    Examples:
      | key                                         | type    | email              | default  | resolved_value | reason          | error_code     |
      | null-default-flag                           | Boolean |                    | true     | true           | ERROR           | FLAG_NOT_FOUND |
      | null-default-flag                           | Boolean |                    | false    | false          | ERROR           | FLAG_NOT_FOUND |
      | undefined-default-flag                      | Integer |                    |      100 |            100 | ERROR           | FLAG_NOT_FOUND |
      | no-default-flag-null-targeting-variant      | String  | wozniak@orange.com | Inventor | Inventor       | ERROR           | FLAG_NOT_FOUND |
      | no-default-flag-null-targeting-variant      | String  | wozniak@orange.com | Founder  | Founder        | ERROR           | FLAG_NOT_FOUND |
      | no-default-flag-null-targeting-variant      | String  | jobs@orange.com    | CEO      | CEO            | TARGETING_MATCH |                |
      | no-default-flag-undefined-targeting-variant | String  | wozniak@orange.com | Retired  | Retired        | ERROR           | FLAG_NOT_FOUND |
