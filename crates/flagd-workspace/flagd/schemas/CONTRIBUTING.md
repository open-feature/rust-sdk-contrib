## Guidelines

Welcome!

Please ensure you have read the [code of conduct](https://github.com/open-feature/.github/blob/main/CODE_OF_CONDUCT.md) before contributing.  
TLDR: be respectful.


## Developing

### flagd JSON schema

The flagd JSON schema defines the json structure used within flagd for flag configurations.   
To contribute to the schema, make changes in `json/flags.yaml`, these changes should then be propagated to `json/flags.json` by running `make gen-schema-json`.  
Unit testing for the schema is managed through [json/test/negative](./json/test/negative) (in which all json files must fail validation against the schema) and [json/test/positive](./json/test/positive) (in which all json files must pass).
