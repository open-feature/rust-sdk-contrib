package flagd

import (
	"encoding/json"
	"os"
	"reflect"
	"testing"
)

// Test JSON merging
func TestCombineJSONFiles(t *testing.T) {
	// Setup: Create temp input directory
	testInputDir := "./test_rawflags"
	os.MkdirAll(testInputDir, os.ModePerm)
	defer os.RemoveAll(testInputDir) // Cleanup after test

	// Create sample JSON files
	json1 := `{"flags": {"feature1": {"state": "on"}}}`
	json2 := `{"flags": {"feature2": {"state": "off"}}}`
	os.WriteFile(testInputDir+"/file1.json", []byte(json1), 0644)
	os.WriteFile(testInputDir+"/file2.json", []byte(json2), 0644)

	// Run function
	err := CombineJSONFiles(testInputDir)
	if err != nil {
		t.Fatalf("CombineJSONFiles failed: %v", err)
	}

	// Verify output
	expected := `{"flags":{"feature1":{"state":"on"},"feature2":{"state":"off"}}}`
	data, _ := os.ReadFile(OutputFile)

	var v1, v2 interface{}

	json.Unmarshal([]byte(expected), &v1)
	json.Unmarshal(data, &v2)
	if !reflect.DeepEqual(v1, v2) {
		t.Errorf("Expected %s, got %s", expected, string(data))
	}
}
