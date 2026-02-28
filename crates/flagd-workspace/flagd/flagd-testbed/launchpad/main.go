package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"openfeature.com/flagd-testbed/launchpad/handlers"
	"openfeature.com/flagd-testbed/launchpad/pkg"
)

func main() {
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM, syscall.SIGINT)
	defer stop()

	if err := flagd.CombineJSONFiles(flagd.InputDir); err != nil {
		log.Fatalf("Error during initial JSON combination: %v", err)
	}

	if err := flagd.StartFileWatcher(); err != nil {
		log.Fatalf("Error starting file watcher: %v", err)
	}

	http.HandleFunc("/start", handlers.StartFlagdHandler)
	http.HandleFunc("/restart", handlers.RestartHandler)
	http.HandleFunc("/stop", handlers.StopFlagdHandler)
	http.HandleFunc("/change", handlers.ChangeHandler)

	server := &http.Server{Addr: ":8080"}

	go func() {
		<-ctx.Done()
		log.Println("Shutting down...")
		timeout, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		err := server.Shutdown(timeout)
		if err != nil {
			fmt.Println("could not stop server", err)
		}
	}()

	err := flagd.StartFlagd("default")
	if err != nil {
		fmt.Printf("Failed to start flagd: %v\n", err)
		os.Exit(1)
	}

	log.Println("Server running on port 8080...")
	if err := server.ListenAndServe(); err != http.ErrServerClosed {
		log.Fatalf("Server error: %v", err)
	}

	if err := os.Remove(flagd.OutputFile); err != nil {
		fmt.Printf("Failed to remove output file: %v\n", err)
	}
}
