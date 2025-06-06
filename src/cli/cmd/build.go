package cmd

import (
	"fmt"
	"os"

	"github.com/distribution/reference"
	"github.com/mathiasgredal/neoboot/src/cli/build/builder"
	"github.com/mathiasgredal/neoboot/src/cli/build/cache"
	"github.com/mathiasgredal/neoboot/src/cli/build/parser"
	"github.com/mathiasgredal/neoboot/src/cli/utils"
	"github.com/mathiasgredal/neoboot/src/cli/utils/log"
	"github.com/spf13/cobra"
)

func NewBuildCommand(cfg *utils.Config) *cobra.Command {
	cmd := &cobra.Command{
		Use:     "build [options] [CONTEXT]",
		Short:   "Build a Neoboot image",
		Args:    cobra.ExactArgs(1),
		RunE:    func(cmd *cobra.Command, args []string) error { return runBuild(cmd, args, cfg) },
		Example: `neoboot build -t my-image:latest .`,
	}

	cmd.Flags().StringP("tag", "t", "", "tagged name to apply to the built image")
	cmd.MarkFlagRequired("tag")
	return cmd
}

func runBuild(cmd *cobra.Command, args []string, cfg *utils.Config) error {
	tag, err := cmd.Flags().GetString("tag")
	if err != nil {
		return fmt.Errorf("failed to get tag: %w", err)
	}

	// Normalize the tag
	namedTag, err := reference.ParseNormalizedNamed(tag)
	if err != nil {
		return fmt.Errorf("failed to parse tag: %w", err)
	}
	tag = reference.TagNameOnly(namedTag).String()
	path := args[0]
	log.Infof("Building image with tag '%s' from path '%s'", tag, path)

	// Check if the path is a directory
	info, err := os.Stat(path)
	if err != nil {
		return fmt.Errorf("failed to stat path: %w", err)
	}
	if !info.IsDir() {
		return fmt.Errorf("path %s is not a directory", path)
	}

	// Check if the bootfile exists
	bootfilePath := path + "/Bootfile.yml"
	if _, err := os.Stat(bootfilePath); os.IsNotExist(err) {
		return fmt.Errorf("bootfile not found at %s", bootfilePath)
	}
	bootfile, err := parser.Parse(bootfilePath)
	if err != nil {
		return fmt.Errorf("failed to parse bootfile: %w", err)
	}

	// Check if the bootfile is empty
	if len(bootfile) == 0 {
		return fmt.Errorf("bootfile is empty")
	}

	// Instantiate the image cache
	cache, err := cache.NewCache(cfg.Paths.CacheDir)
	if err != nil {
		return fmt.Errorf("failed to create cache: %w", err)
	}

	// Instantiate the builder
	builder, err := builder.NewBuilder(cache, path, tag)
	if err != nil {
		return fmt.Errorf("failed to create builder: %w", err)
	}

	if err := builder.Build(bootfile, *cfg); err != nil {
		return fmt.Errorf("failed to build image: %w", err)
	}

	return nil
}
