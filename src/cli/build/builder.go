package build

import (
	"bytes"
	"encoding/json"
	"fmt"
	"path/filepath"
	"strings"

	"github.com/mathiasgredal/neoboot/src/cli/utils"
)

type Builder struct {
	context  *Context
	cache    *Cache
	manifest *Manifest
}

func NewBuilder(cache *Cache, dir string) (*Builder, error) {
	absDir, err := filepath.Abs(dir)
	if err != nil {
		return nil, fmt.Errorf("failed to get absolute path: %w", err)
	}

	return &Builder{context: NewContext(absDir), cache: cache}, nil
}

func (b *Builder) Build(steps []Step, cfg utils.Config) error {
	for _, step := range steps {
		switch step.Command {
		case "ARG":
			if err := b.handleArg(step.Args); err != nil {
				return err
			}
		case "FROM":
			if err := b.handleFrom(step.Args); err != nil {
				return err
			}
		case "VERSION":
			if err := b.handleVersion(step.Args); err != nil {
				return err
			}
		case "BOOTLOADER":
			if err := b.handleBootloader(step.Args); err != nil {
				return err
			}
		// Add cases for other commands
		default:
			return fmt.Errorf("unknown command: %s", step.Command)
		}
	}
	return nil
}

func (b *Builder) handleArg(args any) error {
	argStr, ok := args.(string)
	if !ok {
		return fmt.Errorf("ARG requires a string")
	}
	parts := strings.SplitN(argStr, "=", 2)
	if len(parts) != 2 {
		return fmt.Errorf("invalid ARG format")
	}
	b.context.Vars[parts[0]] = parts[1]
	return nil
}

func (b *Builder) handleFrom(args any) error {
	fromStr, ok := args.(string)
	if !ok {
		return fmt.Errorf("FROM requires an unnamed string argument")
	}

	fromStr = b.context.Substitute(fromStr)

	// Handle special scratch case
	if fromStr == "scratch" {
		return nil
	}

	// TODO: Implement external image handling and tar ball from "file://" and "http://" and "docker://"
	return fmt.Errorf("From with external image not implemented yet")
}

func (b *Builder) handleVersion(args any) error {
	versionStr, ok := args.(string)
	if !ok {
		return fmt.Errorf("VERSION requires a string")
	}

	versionStr = b.context.Substitute(versionStr)

	// Set the version in the context
	b.context.Version = versionStr
	return nil
}

type BootloaderBuildArgs struct {
	Type      string            `json:"type"`
	Selector  string            `json:"selector"`
	Version   string            `json:"version"`
	Builder   string            `json:"builder"`
	BuildArgs map[string]string `json:"build_args"`
	Context   string            `json:"context"`
}

func (b *Builder) handleBootloader(args any) error {
	// Parse the arguments
	jsonData, err := json.Marshal(args)
	if err != nil {
		return fmt.Errorf("failed to marshal bootloader args: %w", err)
	}

	buildArgs := BootloaderBuildArgs{}
	if err := json.Unmarshal(jsonData, &buildArgs); err != nil {
		return fmt.Errorf("failed to unmarshal bootloader args: %w", err)
	}

	// Make the bootload build args context directory absolute and relative to the builder context directory
	buildContext, err := filepath.Abs(filepath.Join(b.context.Dir, buildArgs.Context))
	if err != nil {
		return fmt.Errorf("failed to get absolute path: %w", err)
	}
	buildArgs.Context = buildContext

	fmt.Printf("Building bootloader with context: %s\n", buildArgs.Context)

	// Create a tar archive, with the context directory and a Dockerfile
	buf := bytes.NewBuffer(nil)
	tw, err := utils.MakeTar(b.context.Dir, buildArgs.Context, buf)
	if err != nil {
		return fmt.Errorf("failed to create tar archive: %w", err)
	}

	// Write the Dockerfile to the tar archive
	dockerfile := fmt.Sprintf("FROM %s as builder\n COPY ./bootloader/patches ./yeet\nFROM scratch as dist\nCOPY --from=builder /yeet /yeet\nCOPY --from=builder /yeet /yeet2\nCOPY --from=builder /yeet /yeet3\n", buildArgs.Builder)
	utils.WriteFileToTar(tw, "Dockerfile", []byte(dockerfile))

	// Close the tar archive
	if err := tw.Close(); err != nil {
		return fmt.Errorf("failed to close tar archive: %w", err)
	}

	client, err := utils.GetDockerClient()
	if err != nil {
		return fmt.Errorf("failed to create docker client: %w", err)
	}
	imageID, err := utils.BuildImage(client, buf)
	if err != nil {
		return fmt.Errorf("failed to build image: %w", err)
	}

	fmt.Printf("Built image: %s\n", imageID)

	_, err = utils.GetImageTar(client, imageID)
	if err != nil {
		return fmt.Errorf("failed to get image tar: %w", err)
	}

	// Go through the tar and find the manifest.json, and identify the layer

	// Go through the tar and find the layer, and write the tar file to the cache and add it to our manifest

	// If the builder is a docker image, then the first line of the Dockerfile is FROM the builder image
	// If the builder is a path to a dockerfile, then this is the Dockerfile

	// TODO: Fetch the builder or build the builder image

	// - Create an insitu dockerfile, which has the extra lines appended to build the actual bootloader

	// - Spawn the docker build
	// - Handle the output and turn it into a layer(this will also require configuration management)
	return nil
}
