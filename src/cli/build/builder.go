package build

import (
	"fmt"
	"strings"

	"github.com/mathiasgredal/neoboot/src/cli/utils"
)

type Builder struct {
	context *Context
	// Include other fields like cache, OCI manifest
}

func NewBuilder() *Builder {
	return &Builder{context: NewContext()}
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

func (b *Builder) handleBootloader(args any) error {
	// This is the first handler, which actually needs to do something
	// - Parse arguments
	// - Fetch the builder or build the builder image
	// - Create an insitu dockerfile, which has the extra lines appended to build the actual bootloader
	// - Spawn the docker build
	// - Handle the output and turn it into a layer(this will also require configuration management)
	return nil
}
