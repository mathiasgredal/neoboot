package cmd

import (
	"github.com/mathiasgredal/neoboot/src/cli/utils"
	log "github.com/sirupsen/logrus"
	"github.com/spf13/cobra"
)

func NewPushCommand(cfg *utils.Config) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "push [options] IMAGE [DESTINATION]",
		Short: "Push a Neoboot image to a registry",
		Args:  cobra.ExactArgs(2),
		RunE:  func(cmd *cobra.Command, args []string) error { return runPush(cmd, args, cfg) },
	}
	return cmd
}

func runPush(cmd *cobra.Command, args []string, cfg *utils.Config) error {
	image := args[0]
	destination := args[1]

	log.Infof("Pushing image %s to %s", image, destination)

	// Create a tar archive of the image

	// Import the image into the docker daemon

	// Push the image to the destination

	// Clean up the imported image from the docker daemon

	return nil
}
