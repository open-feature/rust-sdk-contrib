@contextEnrichment
Feature: Context Enrichment

  Background:
    Given a stable flagd provider

  @in-process @rpc
  Scenario: Use enriched context
    Given a String-flag with key "flagd-context-aware" and a default value "not"
    When the flag was evaluated with details
    Then the resolved details value should be "INTERNAL"

  @grace @in-process
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

  @grace @rpc
  Scenario: Use enriched context on connection error for RPC
    Given a String-flag with key "flagd-context-aware" and a default value "not"
    And a stale event handler
    And a ready event handler
    When the flag was evaluated with details
    Then the resolved details value should be "INTERNAL"
    When the connection is lost for 6s
    And a stale event was fired
    When the flag was evaluated with details
    Then the resolved details value should be "not"
    When a ready event was fired

  @rpc @caching
  Scenario: Use enriched context on RPC connection will not cache the value
    Given a String-flag with key "flagd-context-aware" and a default value "not"
    And a change event handler
    And a ready event handler
    When the flag was modified
    And a change event was fired
    And the flag was evaluated with details
    Then the reason should be "TARGETING_MATCH"
    # ensure that we do not cache a "TARGETING_MATCH", we should only cache evaluation with a "STATIC" reason
    When the flag was evaluated with details
    Then the reason should be "TARGETING_MATCH"
