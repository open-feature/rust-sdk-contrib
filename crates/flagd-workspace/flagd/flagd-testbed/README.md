# Flagd Test Harness

This repository contains a Docker image to support the [Gherkin test suites](https://github.com/open-feature/spec/blob/main/specification/appendix-b-gherkin-suites.md) defined in the OpenFeature specification.

## `flagd-testbed` Container

The **`flagd-testbed`** container is a Docker image that includes [**flagd**](https://flagd.dev/) and a testing utility called **launchpad**.
It provides a simple way to run end-to-end (E2E) tests across multiple SDKs.

### Launchpad

**Launchpad** is a control center for managing flagd during tests.
It exposes a REST interface that allows you to:

* Start, stop, or restart flagd
* Dynamically update a single flag, simulating real-time flag changes

This approach centralizes the test orchestration logic, reducing the need to replicate it across SDKs in multiple languages. This simplifies test implementation and allows contributors to focus on testing behavior rather than setup.

#### REST API & Development Docs

Detailed usage and API documentation can be found in the [Launchpad README](./launchpad/README.md).

#### Storing Configurations

Flagd configurations are stored in the [`launchpad/configs`](./launchpad/configs) directory.
Each file corresponds to a named configuration and can be passed as a parameter to the `start` endpoint.

Configurations may include different types of flagd setups and flag files.

### Flag Files

Configured flag definitions reside in the [`flags`](./flags) directory.
Within the Docker image:

* The contents of this directory are mounted under `/rawflags`
* A generated file, `/flags/allFlags.json`, aggregates all flag files **excluding** those prefixed with `selector`

This file is generated when flagd is started and can be used to test all flags in file mode simultaneously.

---

## Gherkin Test Suite

The [`gherkin`](./gherkin) directory contains [*Gherkin*](https://cucumber.io/docs/gherkin/) tests that validate behavior for the configurations defined in the flagd-testbed (see [Flag Files](#flag-files)).
Combined with the appropriate SDK and provider, these tests form a complete integration suite.

### Tagging

Tests are tagged to support selective execution, as not all providers or SDKs support all features. Tags can be applied at both the suite and individual test case level.

Currently supported "provider-tags":

* `@file`
* `@rpc`
* `@in-process`

For example, to run a test in RPC mode, tag the suite or test case with `@rpc`.

Tag-based filtering enables incremental migration and test execution, allowing individual features to be validated progressively rather than all at once.

---

## How to utilize?

This repository should be included as a submodule within your sdk which should be tested.

We recommend:
- to setup renovate to automatically update your submodule, always pointing to version tags and not branches
- to load all gherkin files, and eliminate tests via tag exclusions
- to utilize testcontainers for easier test setup
- to use the version.txt within this repository to load the appropriate docker image

---

### Linting Gherkin Files

You can lint the Gherkin file structure using [gherkin-lint](https://github.com/vsiakka/gherkin-lint).
Requires Node.js v10+.

```bash
npm install
npm run gherkin-lint
```

