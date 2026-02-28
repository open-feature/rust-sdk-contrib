package flagd

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"
	"strings"
	"sync"
)

var (
	mu         sync.Mutex
	InputDir   = "./rawflags"
	outputDir  = "./flags"
	OutputFile = filepath.Join(outputDir, "allFlags.json")
)

func CombineJSONFiles(inputDir string) error {
	mu.Lock()
	defer mu.Unlock()

	files, err := os.ReadDir(inputDir)
	if err != nil {
		return fmt.Errorf("failed to read input directory: %v", err)
	}

	combinedData := make(map[string]interface{})

	for _, file := range files {
		if filepath.Ext(file.Name()) == ".json" && !strings.HasPrefix(file.Name(), "selector-") {
			filePath := filepath.Join(inputDir, file.Name())
			content, err := ioutil.ReadFile(filePath)
			if err != nil {
				return fmt.Errorf("failed to read file %s: %v", file.Name(), err)
			}

			var data map[string]interface{}
			if err := json.Unmarshal(content, &data); err != nil {
				return fmt.Errorf("failed to parse JSON file %s: %v", file.Name(), err)
			}

			combinedData = deepMerge(combinedData, data)
		}
	}

	if err := os.MkdirAll(outputDir, os.ModePerm); err != nil {
		return fmt.Errorf("failed to create output directory: %v", err)
	}

	combinedContent, err := json.MarshalIndent(combinedData, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to serialize combined JSON: %v", err)
	}

	return ioutil.WriteFile(OutputFile, combinedContent, 0644)
}

func deepMerge(dst, src map[string]interface{}) map[string]interface{} {
	for key, srcValue := range src {
		if dstValue, exists := dst[key]; exists {
			if srcMap, ok := srcValue.(map[string]interface{}); ok {
				if dstMap, ok := dstValue.(map[string]interface{}); ok {
					dst[key] = deepMerge(dstMap, srcMap)
					continue
				}
			}
		}
		dst[key] = srcValue
	}
	return dst
}
