# Flagd Testbed Launchpad Application

Launchpad is a lightweight HTTP server built in Go that controls a `flagd` binary and provides endpoints to manage its lifecycle and configuration. The application also allows toggling a flag's `defaultVariant` dynamically and saves the updated configuration to a file.

Additionally, launchpad will write the whole configuration as one combined JSON file into the "flags" directory with the name "allFlags.json".
This file can be utilized for File provider tests, instead of implementing a json manipulation in all languages.
Mount the folder of the docker image to a local directory, and it will generate the file into this folder.

## Features

- **Start and Stop `flagd`:** 
  Use `/start` and `/stop` endpoints to manage the `flagd` process.
  
- **Dynamic Configuration Toggling:**
  The `/change` endpoint toggles the `defaultVariant` for a specific flag and saves the change to the configuration file.

## Endpoints

### 1. `/start`
- **Method:** `POST`
- **Description:** Starts the `flagd` binary with the specified configuration.
- **Query Parameters:**
  - `config` (optional): Name of the configuration to load. Defaults to `"default"`.
- **Example:**
  ```bash
  curl -X POST http://localhost:8080/start?config=my-config
  ```

### 2. `/stop`
- **Method:** `POST`
- **Description:** Stops the running `flagd` binary.
- **Example:**
  ```bash
  curl -X POST http://localhost:8080/stop
  ```

### 3. `/change`
- **Method:** `POST`
- **Description:** Toggles the `defaultVariant` for the flag `changing-flag` between `"foo"` and `"bar"` and saves the updated configuration to the file `changing-flag.json`.
- **Example:**
  ```bash
  curl -X POST http://localhost:8080/change

### 3. `/restart`
- **Method:** `POST`
- **Description:** restarts the running `flagd` binary. 
- **Query Parameters:**
    - `seconds` (optional): Time between stop and start. Defaults to `"5"`.

- **Example:**
  ```bash
  curl -X POST http://localhost:8080/restart?seconds=5 ```

## Configuration Files

The application relies on JSON configuration files to manage flags for `flagd`. The configuration files are stored locally, and the `/change` endpoint modifies the file `changing-flag.json`.

### Example Configuration (`changing-flag.json`)

```json
{
  "flags": {
    "changing-flag": {
      "state": "ENABLED",
      "variants": {
        "foo": "foo",
        "bar": "bar"
      },
      "defaultVariant": "foo"
    }
  }
}
```

## Running the Application

1. **Build and Run the Application:**
   ```bash
   go run main.go
   ```

2. **Start the `flagd` Binary:**
   ```bash
   curl -X POST http://localhost:8080/start?config=default
   ```

3. **Stop the `flagd` Binary:**
   ```bash
   curl -X POST http://localhost:8080/stop
   ```

4. **Toggle the Default Variant:**
   ```bash
   curl -X POST http://localhost:8080/change
   ```


## Development Notes

- Ensure that `flagd` is available in the application directory or adjust the path accordingly.
- Modify the `changing-flag.json` file if additional flags or configurations are required.
