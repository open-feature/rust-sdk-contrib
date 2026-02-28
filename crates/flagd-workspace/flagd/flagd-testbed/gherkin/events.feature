@rpc @in-process @file @events
Feature: Flagd Provider State Changes

  Background:
    Given a stable flagd provider

  Scenario: Provider events chain ready -> error -> ready
    Given a ready event handler
    And a error event handler
    Then the ready event handler should have been executed
    When the connection is lost for 3s
    Then the error event handler should have been executed
    Then the ready event handler should have been executed

  @grace
  Scenario: Provider events chain ready -> stale -> error -> ready
    Given a ready event handler
    And a error event handler
    And a stale event handler
    Then the ready event handler should have been executed
    When the connection is lost for 3s
    Then the stale event handler should have been executed
    Then the error event handler should have been executed
    Then the ready event handler should have been executed

  @grace
  Scenario: Provider events chain ready -> stale -> ready
    Given a ready event handler
    And a error event handler
    And a stale event handler
    Then the ready event handler should have been executed
    When the connection is lost for 1s
    Then the stale event handler should have been executed
    Then the ready event handler should have been executed

  Scenario: Flag change event
    Given a String-flag with key "changing-flag" and a default value "false"
    And a change event handler
    When the flag was modified
    And a change event was fired
    Then the flag should be part of the event payload
