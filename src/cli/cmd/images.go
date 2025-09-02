package cmd

import (
	"fmt"
	"os"
	"text/tabwriter"

	"github.com/mathiasgredal/neoboot/src/cli/build/cache"
	"github.com/mathiasgredal/neoboot/src/cli/utils"
	"github.com/spf13/cobra"
)

func NewImagesCommand(cfg *utils.Config) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "images [options]",
		Short: "List Neoboot images",
		Args:  cobra.NoArgs,
		RunE:  func(cmd *cobra.Command, args []string) error { return runImages(cmd, args, cfg) },
	}
	return cmd
}

func runImages(cmd *cobra.Command, args []string, cfg *utils.Config) error {
	// Instantiate the image cache
	cache, err := cache.NewCache(cfg.Paths.CacheDir)
	if err != nil {
		return fmt.Errorf("failed to create cache: %w", err)
	}

	// List the images
	images, err := cache.ListImages()
	if err != nil {
		return fmt.Errorf("failed to list images: %w", err)
	}

	// Initialize a new tabwriter.
	w := tabwriter.NewWriter(os.Stdout, 0, 0, 3, ' ', 0)

	// Print header row
	fmt.Fprintln(w, "REPOSITORY\tTAG\tIMAGE ID\tCREATED\tSIZE")

	// Print data rows
	for _, img := range images {
		imageInfo, err := cache.GetImageInfo(img)
		if err != nil {
			return fmt.Errorf("failed to get image info: %w", err)
		}
		fmt.Fprintf(w, "%s\t%s\t%s\t%s\t%s\n",
			imageInfo.Name,
			imageInfo.Tag,
			imageInfo.ShortDigest,
			imageInfo.Created.Format("2006-01-02 15:04:05"),
			utils.BytesHumanize(uint64(imageInfo.Size)),
		)
	}

	// Flush writes to the underlying io.Writer (os.Stdout)
	// This is important! Without Flush, you might not see any output.
	if err := w.Flush(); err != nil {
		fmt.Fprintf(os.Stderr, "Error flushing tabwriter: %v\n", err)
		os.Exit(1)
	}

	return nil
}
