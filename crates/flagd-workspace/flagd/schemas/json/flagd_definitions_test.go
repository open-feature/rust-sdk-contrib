package flagd_definitions_test

import (
	"fmt"
	flagd_definitions "github.com/open-feature/flagd-schemas/json"
	"log"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/santhosh-tekuri/jsonschema/v6"
)

var compiler *jsonschema.Compiler

func init() {
	// Create a new JSON Schema compiler
	compiler = jsonschema.NewCompiler()

	// Add the Flagd Daemon schema
	flagdFile := strings.NewReader(flagd_definitions.FlagdSchema)
	json, _ := jsonschema.UnmarshalJSON(flagdFile)
	if err := compiler.AddResource("https://flagd.dev/schema/v0/flagd.json", json); err != nil {
		log.Fatalf("Failed to add flagd schema: %v", err)
	}

	// Add the Flag schema
	flagsFile := strings.NewReader(flagd_definitions.FlagSchema)
	json, _ = jsonschema.UnmarshalJSON(flagsFile)
	if err := compiler.AddResource("https://flagd.dev/schema/v0/flags.json", json); err != nil {
		log.Fatalf("Failed to add flags schema: %v", err)
	}

	// Add the Targeting schema
	targetingFile := strings.NewReader(flagd_definitions.TargetingSchema)
	json, _ = jsonschema.UnmarshalJSON(targetingFile)
	if err := compiler.AddResource("https://flagd.dev/schema/v0/targeting.json", json); err != nil {
		log.Fatalf("Failed to add targeting schema: %v", err)
	}
}

func TestPositiveFlagdParsing(t *testing.T) {

	if err := walkPath(true, "./test/flagd/positive"); err != nil {
		t.Error(err)
		t.FailNow()
	}
}

func TestNegativeFlagdParsing(t *testing.T) {

	if err := walkPath(true, "./test/flagd/negative"); err != nil {
		t.Error(err)
		t.FailNow()
	}
}

func TestPositiveFlagsParsing(t *testing.T) {

	if err := walkPath(true, "./test/flags/positive"); err != nil {
		t.Error(err)
		t.FailNow()
	}
}

func TestNegativeFlagsParsing(t *testing.T) {
	if err := walkPath(false, "./test/flags/negative"); err != nil {
		t.Error(err)
		t.FailNow()
	}
}

func TestPositiveTargetingParsing(t *testing.T) {
	if err := walkPath(true, "./test/targeting/positive"); err != nil {
		t.Error(err)
		t.FailNow()
	}
}

func TestNegativeTargetingParsing(t *testing.T) {
	if err := walkPath(false, "./test/targeting/negative"); err != nil {
		t.Error(err)
		t.FailNow()
	}
}

func walkPath(shouldPass bool, root string) error {
	return filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		// Only process JSON files
		if !strings.HasSuffix(path, ".json") {
			return nil
		}

		// Open the file
		f, err := os.Open(path)
		if err != nil {
			return fmt.Errorf("failed to open file %s: %v", path, err)
		}
		defer f.Close()

		// Unmarshal the JSON instance
		inst, err := jsonschema.UnmarshalJSON(f)
		if err != nil {
			return fmt.Errorf("failed to unmarshal JSON in file %s: %v", path, err)
		}
		obj, ok := inst.(map[string]interface{})
		if !ok {
			return fmt.Errorf("inst is not a map[string]interface{}")
		}
		var schema *jsonschema.Schema
		if schemaValue, ok := obj["$schema"]; ok {
			// Type-assert the value to a string
			if schemaStr, ok := schemaValue.(string); ok {
				schemaStr = strings.ReplaceAll(schemaStr, "../../..", "https://flagd.dev/schema/v0")
				schema, err = compiler.Compile(schemaStr)
			} else {
				return fmt.Errorf("The value of `$schema` is not a string")
			}
		} else {
			schema, err = compiler.Compile("https://flagd.dev/schema/v0/targeting.json")
		}

		// Validate the instance against the schema
		err = schema.Validate(inst)
		if err == nil && !shouldPass {
			return fmt.Errorf("file %s should have failed validation, but did not", path)
		}
		if err != nil && shouldPass {
			return fmt.Errorf("file %s should not have failed validation, but did: %v", path, err)
		}

		return nil
	})
}

type SchemaObject struct {
	Schema string `json:"$schema"`
}
