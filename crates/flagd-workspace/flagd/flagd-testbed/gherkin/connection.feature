Feature: flagd provider disconnect and reconnect functionality

  @rpc @in-process @file
  Scenario Outline: Connection
    Given a <name> flagd provider
    And a Boolean-flag with key "boolean-flag" and a default value "false"
    When the flag was evaluated with details
    Then the resolved details value should be "true"

    Scenarios: Stable
      | name   |
      | stable |
    @customCert
    Scenarios: Certificates
      | name |
      | ssl  |
    @unixsocket @os.linux
    Scenarios: Unixsocket
      | name   |
      | socket |

  @rpc @in-process @file @reconnect
  # This test suite tests the reconnection functionality of flagd providers
  Scenario Outline: Provider reconnection
    Given a <name> flagd provider
    And a ready event handler
    And a error event handler
    When a ready event was fired
    When the connection is lost for 5s
    Then the error event handler should have been executed
    Then the ready event handler should have been executed

    Scenarios: Stable
      | name   |
      | stable |
    @customCert
    Scenarios: Certificates
      | name |
      | ssl  |
  # unix sockets and reconnects is a strange topic and not as easily handled as like tcp reconnects
  #  @unixsocket @os.linux
  #  Scenarios: Unixsocket
  #    | name   |
  #    | socket |

  @rpc @in-process @file @unavailable
  Scenario: Provider unavailable
    Given an option "deadlineMs" of type "Integer" with value "1000"
    And a unavailable flagd provider
    And a error event handler
    Then the error event handler should have been executed within 3000ms

  @rpc @in-process @file @forbidden
  # This test ensures that a forbidden response from flagd results in a fatal client state
  Scenario: Provider forbidden
    Given an option "fatalStatusCodes" of type "StringList" with value "PERMISSION_DENIED"
    And a forbidden flagd provider
    And a error event handler
    Then the error event handler should have been executed within 5000ms
    And the client is in fatal state

  @targetURI @rpc
  Scenario: Connection via TargetUri rpc
    Given an option "targetUri" of type "String" with value "envoy://localhost:<port>/rpc.service"
    And a stable flagd provider
    And a Boolean-flag with key "boolean-flag" and a default value "false"
    When the flag was evaluated with details
    Then the resolved details value should be "true"

  @targetURI @in-process
  Scenario: Connection via TargetUri in-process
    Given an option "targetUri" of type "String" with value "envoy://localhost:<port>/sync.service"
    And a stable flagd provider
    And a Boolean-flag with key "boolean-flag" and a default value "false"
    When the flag was evaluated with details
    Then the resolved details value should be "true"

  @targetURI @in-process @reconnect
  Scenario: Re-Connection via TargetUri in-process
    Given an option "targetUri" of type "String" with value "envoy://localhost:<port>/sync.service"
    And a stable flagd provider
    And a ready event handler
    And a error event handler
    When a ready event was fired
    When the connection is lost for 3s
    Then the error event handler should have been executed
    Then the ready event handler should have been executed

  @targetURI @in-process @reconnect @grace
  Scenario: Re-Connection via TargetUri in-process with grace period
    Given an option "targetUri" of type "String" with value "envoy://localhost:<port>/sync.service"
    And a stable flagd provider
    And a ready event handler
    And a stale event handler
    And a error event handler
    When a ready event was fired
    When the connection is lost for 3s
    Then the stale event handler should have been executed
    Then the error event handler should have been executed
    Then the ready event handler should have been executed
