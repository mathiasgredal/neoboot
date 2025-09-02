package builder

import (
	"bytes"
	"encoding/json"
	"fmt"
	"path/filepath"
	"time"

	"github.com/mathiasgredal/neoboot/src/cli/build/cache"
	"github.com/mathiasgredal/neoboot/src/cli/build/context"
	"github.com/mathiasgredal/neoboot/src/cli/build/oci"
	"github.com/mathiasgredal/neoboot/src/cli/build/parser"
	"github.com/mathiasgredal/neoboot/src/cli/build/steps"
	"github.com/mathiasgredal/neoboot/src/cli/utils"
)

type Builder struct {
	context  *context.Context
	cache    *cache.Cache
	manifest *oci.Manifest
	config   *oci.Config
}

func NewBuilder(cache *cache.Cache, dir string, tag string) (*Builder, error) {
	absDir, err := filepath.Abs(dir)
	if err != nil {
		return nil, fmt.Errorf("failed to get absolute path: %w", err)
	}

	return &Builder{
		context: context.NewContext(absDir, tag),
		cache:   cache,
		manifest: &oci.Manifest{
			SchemaVersion: 2,
			MediaType:     oci.MediaTypeImageManifest,
			Config: oci.Descriptor{
				MediaType: oci.MediaTypeImageConfig,
				Digest:    "",
				Size:      0,
			},
			Layers: []oci.Descriptor{},
		},
		config: &oci.Config{
			Created: time.Now().Format(time.RFC3339),
			Author:  "neoboot",
			OS:      "linux",
			Config: oci.ImageConfig{
				State: "bootfile_state",
			},
			Rootfs: oci.ImageRootfs{
				Type:    "layers",
				DiffIDs: []string{},
			},
		}}, nil
}

func (b *Builder) Build(buildSteps []parser.Step, cfg utils.Config) error {
	for _, step := range buildSteps {
		switch step.Command {
		case "ARG":
			if err := steps.HandleArg(b.context, step.Args); err != nil {
				return err
			}
		case "FROM":
			if err := steps.HandleFrom(b.context, step.Args); err != nil {
				return err
			}
		case "VERSION":
			if err := steps.HandleVersion(b.context, step.Args); err != nil {
				return err
			}
		case "BOOTLOADER":
			if err := steps.HandleBootloader(b.context, b.cache, b.manifest, b.config, step.Args); err != nil {
				return err
			}
		// Add cases for other commands
		default:
			return fmt.Errorf("unknown command: %s", step.Command)
		}
	}

	// Marshal the image config and write it to the cache
	imageConfigJSON, err := json.Marshal(b.config)
	if err != nil {
		return fmt.Errorf("failed to marshal image config: %w", err)
	}
	configDigest, size, err := b.cache.Write(bytes.NewReader(imageConfigJSON))
	if err != nil {
		return fmt.Errorf("failed to write image config to cache: %w", err)
	}

	// Update the manifest with the config digest
	b.manifest.Config.Digest = "sha256:" + configDigest[7:]
	b.manifest.Config.Size = size

	// Write the manifest to the cache
	b.cache.WriteManifest(b.context.Tag, b.manifest)

	return nil
}
