package main

import (
	"encoding/json"
	"os"

	"github.com/mathiasgredal/neoboot/src/cli/cmd"
	"github.com/mathiasgredal/neoboot/src/cli/utils"
	"github.com/mathiasgredal/neoboot/src/cli/utils/log"
	"github.com/spf13/cobra"
)

func main() {
	// Initialize root command
	rootCmd := &cobra.Command{
		Use:   "neoboot",
		Short: "Neoboot - A modern bootloader management tool",
	}

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
	rootCmd.AddCommand(cmd.NewImagesCommand(cfg))
	rootCmd.AddCommand(cmd.NewPushCommand(cfg))
	// Add inspect, rm and prune commands for images

	// Execute root command
	if err := rootCmd.Execute(); err != nil {
		os.Exit(1)
	}
}
