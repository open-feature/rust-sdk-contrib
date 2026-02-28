package flagd

import (
	"bufio"
	"context"
	"errors"
	"fmt"
	"io"
	"os"
	"os/exec"
	"strings"
	"sync"
	"time"
)

var (
	flagdCmd          *exec.Cmd
	flagdLock         sync.Mutex
	Config            = "default"
	restartCancelFunc context.CancelFunc // Stores the cancel function for delayed restarts
)

func ensureStartConditions() {
	if _, err := os.Stat(OutputFile); errors.Is(err, os.ErrNotExist) {
		err := CombineJSONFiles(InputDir)
		if err != nil {
			fmt.Printf("Error combining JSON files on flagd start: %v\n", err)
		}
	}
	ContinueFileWatcher()
}

func deleteCombinedFlagsFile() {
	// if we cannot delete it, we can assume it did not exist in the first place, so we can ignore this error
	_ = os.Remove(OutputFile)
}

func RestartFlagd(seconds int) {
	flagdLock.Lock()
	if restartCancelFunc != nil {
		restartCancelFunc()
		fmt.Println("Previous restart canceled.")
	}

	ctx, cancel := context.WithCancel(context.Background())
	restartCancelFunc = cancel

	PauseFileWatcher()
	deleteCombinedFlagsFile()
	err := stopFlagDWithoutLock()
	if err != nil {
		fmt.Printf("Failed to restart flagd: %v\n", err)
	}
	flagdLock.Unlock()

	go func() {
		fmt.Printf("flagd will restart in %d seconds...\n", seconds)
		select {
		case <-time.After(time.Duration(seconds) * time.Second):
			fmt.Println("Restarting flagd now...")
			if err := StartFlagd(Config); err != nil {
				fmt.Printf("Failed to restart flagd: %v\n", err)
			} else {
				fmt.Println("flagd restarted successfully.")
			}
		case <-ctx.Done():
			fmt.Println("Restart canceled before execution.")
		}
	}()
}

func StartFlagd(config string) error {
	if config == "" {
		config = Config
	} else {
		Config = config
	}

	flagdLock.Lock()
	// Cancel any pending restart attempts
	if restartCancelFunc != nil {
		restartCancelFunc()
		fmt.Println("Pending restart canceled due to manual start.")
		restartCancelFunc = nil
	}

	if err := stopFlagDWithoutLock(); err != nil {
		return err
	}

	ensureStartConditions()

	configPath := fmt.Sprintf("./configs/%s.json", config)

	flagdCmd = exec.Command("./flagd", "start", "--config", configPath)

	stdout, err := flagdCmd.StdoutPipe()
	if err != nil {
		return fmt.Errorf("failed to capture stdout: %v", err)
	}
	stderr, err := flagdCmd.StderrPipe()
	if err != nil {
		return fmt.Errorf("failed to capture stderr: %v", err)
	}

	if err := flagdCmd.Start(); err != nil {
		return fmt.Errorf("failed to start flagd: %v", err)
	}

	flagdLock.Unlock()
	ready := make(chan bool)

	go monitorOutput(stdout, ready, "stdout")
	go monitorOutput(stderr, ready, "stderr")

	select {
	case success := <-ready:
		if success {
			fmt.Println("flagd started successfully.")
			return nil
		}
		return fmt.Errorf("flagd did not start correctly")
	case <-time.After(10 * time.Second):
		err := StopFlagd()
		if err != nil {
			fmt.Println("could not stop flagd", err)
		}
		return fmt.Errorf("flagd start timeout exceeded")
	}
}

func StopFlagd() error {
	flagdLock.Lock()
	defer flagdLock.Unlock()

	// Cancel any pending restart attempts
	if restartCancelFunc != nil {
		restartCancelFunc()
		fmt.Println("Pending restart canceled due to manual start.")
		restartCancelFunc = nil
	}

	err := stopFlagDWithoutLock()
	if err != nil {
		return err
	}

	return nil
}

func stopFlagDWithoutLock() error {
	if flagdCmd != nil && flagdCmd.Process != nil {
		if err := flagdCmd.Process.Kill(); err != nil {
			return fmt.Errorf("failed to stop flagd: %v", err)
		}
		flagdCmd = nil
		fmt.Println("flagd stopped")
	}
	return nil
}

func monitorOutput(pipe io.ReadCloser, ready chan bool, stream string) {
	scanner := bufio.NewScanner(pipe)
	//adjust the capacity to your need (max characters in line)
	const maxCapacity = 512 * 1024
	buf := make([]byte, maxCapacity)
	scanner.Buffer(buf, maxCapacity)
	started := false

	for scanner.Scan() {
		line := scanner.Text()
		fmt.Println("[flagd ", stream, "]:", line)
		if ready != nil && !started && strings.Contains(line, "listening at") {
			ready <- true
			close(ready)
			fmt.Println("flagd started properly found logline with 'listening at'")
			started = true
		}
	}
}
