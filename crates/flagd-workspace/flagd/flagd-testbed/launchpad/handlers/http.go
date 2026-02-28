package handlers

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"openfeature.com/flagd-testbed/launchpad/pkg"
	"os"
	"strconv"
	"sync"
	"time"
)

// Response struct to standardize API responses
type Response struct {
	Status  string `json:"status"`
	Message string `json:"message"`
}

// StartFlagdHandler starts the `flagd` process
func StartFlagdHandler(w http.ResponseWriter, r *http.Request) {
	config := r.URL.Query().Get("config")

	if err := flagd.StartFlagd(config); err != nil {
		respondWithJSON(w, http.StatusInternalServerError, "error", fmt.Sprintf("Failed to start flagd: %v", err))
		return
	}
	respondWithJSON(w, http.StatusOK, "success", "flagd started successfully")
}

// RestartHandler stops and starts `flagd`
func RestartHandler(w http.ResponseWriter, r *http.Request) {
	secondsStr := r.URL.Query().Get("seconds")
	if secondsStr == "" {
		secondsStr = "5"
	}

	seconds, err := strconv.Atoi(secondsStr)
	if err != nil || seconds < 0 {
		respondWithJSON(w, http.StatusBadRequest, "error", "'seconds' must be a non-negative integer")
		return
	}

	flagd.RestartFlagd(seconds)
	respondWithJSON(w, http.StatusOK, "success", fmt.Sprintf("flagd will restart in %d seconds", seconds))
}

// StopFlagdHandler stops `flagd`
func StopFlagdHandler(w http.ResponseWriter, r *http.Request) {
	if err := flagd.StopFlagd(); err != nil {
		respondWithJSON(w, http.StatusInternalServerError, "error", fmt.Sprintf("Failed to stop flagd: %v", err))
		return
	}
	respondWithJSON(w, http.StatusOK, "success", "flagd stopped successfully")
}

type FlagConfig struct {
	Flags map[string]struct {
		State          string            `json:"state"`
		Variants       map[string]string `json:"variants"`
		DefaultVariant string            `json:"defaultVariant"`
	} `json:"flags"`
}

var mu sync.Mutex // Mutex to ensure thread-safe file operations

// ChangeHandler triggers JSON file merging and notifies `flagd`
func ChangeHandler(w http.ResponseWriter, r *http.Request) {
	mu.Lock()
	defer mu.Unlock()

	// Path to the configuration file
	configFile := "rawflags/changing-flag.json"

	// Read the existing file
	data, err := os.ReadFile(configFile)
	if err != nil {
		http.Error(w, fmt.Sprintf("Failed to read file: %v", err), http.StatusInternalServerError)
		return
	}

	// Parse the JSON into the FlagConfig struct
	var config FlagConfig
	if err := json.Unmarshal(data, &config); err != nil {
		http.Error(w, fmt.Sprintf("Failed to parse JSON: %v", err), http.StatusInternalServerError)
		return
	}

	// Find the "changing-flag" and toggle the default variant
	flag, exists := config.Flags["changing-flag"]
	if !exists {
		http.Error(w, "Flag 'changing-flag' not found in the configuration", http.StatusNotFound)
		return
	}

	// Toggle the defaultVariant between "foo" and "bar"
	if flag.DefaultVariant == "foo" {
		flag.DefaultVariant = "bar"
	} else {
		flag.DefaultVariant = "foo"
	}

	// Save the updated flag back to the configuration
	config.Flags["changing-flag"] = flag
	// Serialize the updated configuration back to JSON
	updatedData, err := json.MarshalIndent(config, "", "  ")
	if err != nil {
		http.Error(w, fmt.Sprintf("Failed to serialize updated JSON: %v", err), http.StatusInternalServerError)
		return
	}

	// the file watcher should be triggered instantly. If not, we add a timeout to prevent a hanging test
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	// wait for the filewatcher to register an update and write the new json file
	flagUpdateWait := sync.WaitGroup{}
	flagUpdateWait.Add(1)
	flagd.RegisterWaitForNextChangingFlagUpdate(&flagUpdateWait)
	go func() {
		flagUpdateWait.Wait()
		cancel()
	}()

	// Write the updated JSON back to the file
	if err := os.WriteFile(configFile, updatedData, 0644); err != nil {
		http.Error(w, fmt.Sprintf("Failed to write updated file: %v", err), http.StatusInternalServerError)
		return
	}

	select {
	case <-ctx.Done():
		if errors.Is(ctx.Err(), context.DeadlineExceeded) {
			http.Error(w, fmt.Sprintf("Flags were not updated in time: %v", ctx.Err()), http.StatusInternalServerError)
		} else {
			respondWithJSON(w, http.StatusOK, "success", fmt.Sprintf("Default variant successfully changed to '%s'\n", flag.DefaultVariant))
		}
	}
}

// Utility function to send JSON responses
func respondWithJSON(w http.ResponseWriter, statusCode int, status, message string) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(statusCode)

	response := Response{
		Status:  status,
		Message: message,
	}

	json.NewEncoder(w).Encode(response)
}
