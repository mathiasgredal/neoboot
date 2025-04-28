package main

import (
	"encoding/json"
	"os"

	"github.com/mathiasgredal/neoboot/src/cli/cmd"
	"github.com/mathiasgredal/neoboot/src/cli/utils"
	log "github.com/sirupsen/logrus"
	"github.com/spf13/cobra"
)

func main() {
	// Initialize root command
	rootCmd := &cobra.Command{
		Use:   "neoboot",
		Short: "Neoboot - A modern bootloader management tool", // Updated description
	}

	// Configure logging settings
	log.SetFormatter(&log.TextFormatter{
		DisableColors: false,
		FullTimestamp: true,
	})
	log.SetLevel(log.DebugLevel)

	// Set up configuration
	var configPath string
	rootCmd.PersistentFlags().StringVarP(&configPath, "config", "c", "", "config file path")

	cfg, err := utils.LoadConfig(configPath)
	if err != nil {
		log.Errorf("Error loading config: %v", err)
		os.Exit(1)
	}
	serialized_cfg, _ := json.MarshalIndent(cfg, "", "  ")
	log.Debugf("Using configuration: %s", string(serialized_cfg))

	// Add subcommands
	rootCmd.AddCommand(cmd.NewBuildCommand(cfg))

	// Execute root command
	if err := rootCmd.Execute(); err != nil {
		os.Exit(1)
	}
}
