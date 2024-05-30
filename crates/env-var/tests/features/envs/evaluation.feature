Feature: Flag evaluation

# This test suite contains scenarios to test the flag evaluation API.

  Background:
    Given a provider is registered

  Scenario: Resolves boolean value
    When a boolean flag with key "boolean-flag" is evaluated with default value "false"
    Then the resolved boolean value should be "true"

  Scenario: Resolves string value
    When a string flag with key "string-flag" is evaluated with default value "bye"
    Then the resolved string value should be "hi"

  Scenario: Resolves integer value
    When an integer flag with key "integer-flag" is evaluated with default value 1
    Then the resolved integer value should be 10

  Scenario: Resolves float value
    When a float flag with key "float-flag" is evaluated with default value 0.1
    Then the resolved float value should be 0.5 

  Scenario: Flag not found
    When a non-existent string flag with key "missing-flag" is evaluated with details and a default value "uh-oh"
    Then the reason should indicate an error and the error code should indicate a missing flag with "FLAG_NOT_FOUND"

# Not supported
#  Scenario: Resolves object value
#    When an object flag with key "object-flag" is evaluated with a null default value
#    Then the resolved object value should be contain fields "showImages", "title", and "imagesPerPage", with values "true", "Check out these pics!" and 100, respectively         # detailed evaluation
