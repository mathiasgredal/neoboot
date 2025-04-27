package main

import (
	log "github.com/sirupsen/logrus"
)

func main() {
	// Set the log level to debug
	log.SetLevel(log.DebugLevel)

	// Print hello world
	log.Info("Hello")
}
