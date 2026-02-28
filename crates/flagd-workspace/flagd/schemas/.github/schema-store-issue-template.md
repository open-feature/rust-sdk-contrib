---
title: "chore: update flagd and schema store"
---

# Update Schema Store JSON Schema

[Schema store repo](https://github.com/SchemaStore/schemastore)

Extend the catalog entry found in `src/api/json/catalog.json` to include the new version:

```diff
{
    "name": "flagd flag configuration",
    "description": "Flag configuration for the OpenFeature spec compliant flagd provider",
    "fileMatch": [
    "flagd.json",
    "flagd.yaml",
    "flagd.yml",
    "*.flagd.json",
    "*.flagd.yaml",
    "*.flagd.yml"
    ],
    "url": "https://flagd.dev/schema/v0/flags.json",
    "versions": {
    "0.1.1": "https://raw.githubusercontent.com/open-feature/schemas/json/json-schema-v0.1.1/json/flagd-definitions.json",
+   "X.X.X": "https://raw.githubusercontent.com/open-feature/schemas/json/json-schema-vX.X.X/json/flagd-definitions.json"
}
```

# Update flagd.dev JSON Schema

[flagd.dev](https://flagd.dev/) hosts the canonical schema at: https://flagd.dev/schema/vX/flags.json (where X is the major version number).
See the [Makefile in flagd](https://github.com/open-feature/flagd/blob/main/Makefile) for instructions on how to update this version.
