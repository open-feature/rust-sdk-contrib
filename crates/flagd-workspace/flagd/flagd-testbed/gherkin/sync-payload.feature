@in-process @sync-payload
Feature: Disabled Sync Metadata

  Background:
    Given a syncpayload flagd provider

  Scenario: Use enriched context
    Given a String-flag with key "flagd-context-aware" and a default value "not"
    When the flag was evaluated with details
    Then the resolved details value should be "INTERNAL"

  @grace
  Scenario: Use enriched context on connection error for IN-PROCESS
    Given a String-flag with key "flagd-context-aware" and a default value "not"
    And a stale event handler
    And a ready event handler
    When the flag was evaluated with details
    Then the resolved details value should be "INTERNAL"
    When the connection is lost for 6s
    And a stale event was fired
    When the flag was evaluated with details
    Then the resolved details value should be "INTERNAL"
    When a ready event was fired

  Scenario: Resolve value independent of context
    Given a Boolean-flag with key "boolean-flag" and a default value "false"
    When the flag was evaluated with details
    Then the resolved details value should be "true"
