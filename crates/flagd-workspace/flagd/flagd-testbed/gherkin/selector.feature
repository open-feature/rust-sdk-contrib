@in-process
Feature: flagd selector

  Background:
    Given an option "selector" of type "String" with value "rawflags/selector-flags.json"
    And a stable flagd provider

  Scenario Outline: Flags not found
    Given a <type>-flag with key "<key>" and a default value "<default>"
    When the flag was evaluated with details
    Then the reason should be "ERROR"

    Examples:
      | key          | type    | default |
      | boolean-flag | Boolean | false   |
      | string-flag  | String  | bye     |
      | integer-flag | Integer | 1       |
      | float-flag   | Float   | 0.1     |

  Scenario: Resolve values
    Given a String-flag with key "selector-flag" and a default value "foo"
    When the flag was evaluated with details
    Then the reason should be "STATIC"
