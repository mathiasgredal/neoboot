package steps

import (
	"archive/tar"
	"encoding/json"
	"fmt"
	"io"

	log "github.com/sirupsen/logrus"

	"github.com/mathiasgredal/neoboot/src/cli/build/cache"
	"github.com/mathiasgredal/neoboot/src/cli/build/context"
	"github.com/mathiasgredal/neoboot/src/cli/build/oci"
	"github.com/mathiasgredal/neoboot/src/cli/utils"
)

// Handle the BOOTLOADER command
// TODO:
// - Support caching
// - Support different sources for the bootloader
// - Support var substitution in the bootloader build args
type BootloaderBuildArgs struct {
	Type          string            `json:"type"`
	Selector      string            `json:"selector"`
	Version       string            `json:"version"`
	From          string            `json:"from"`
	FromLocal     string            `json:"from_local"`
	BuildRaw      any               `json:"build"`
	Build         utils.DockerBuild `json:"-"`
	Location      string            `json:"location"`
	FromWasm      string            `json:"from_wasm"`
	FromLocalWasm string            `json:"from_local_wasm"`
	BuildWasmRaw  any               `json:"build_wasm"`
	BuildWasm     utils.DockerBuild `json:"-"`
	LocationWasm  string            `json:"location_wasm"`
}

func HandleBootloader(ctx *context.Context, cache *cache.Cache, manifest *oci.Manifest, config *oci.Config, args any) error {
	// Parse the arguments
	buildArgs, err := parseBootloaderBuildArgs(args)
	if err != nil {
		return fmt.Errorf("failed to parse bootloader args: %w", err)
	}

	// Assert u-boot type, as ipxe is not supported yet
	if buildArgs.Type != "u-boot" {
		return fmt.Errorf("bootloader type %s not supported yet", buildArgs.Type)
	}

	// Get the docker client
	client, err := utils.GetDockerClient()
	if err != nil {
		return fmt.Errorf("failed to create docker client: %w", err)
	}

	// Switch on the type of build for wasm(from, from_local, build, location)
	var wasmResource io.Reader
	switch getBuildWasmType(buildArgs) {
	case "from":
		return fmt.Errorf("from wasm not implemented yet")
	case "from_local":
		return fmt.Errorf("from local wasm not implemented yet")
	case "location":
		return fmt.Errorf("location wasm not implemented yet")
	case "build":
		wasmResource, err = buildArgs.BuildWasm.BuildImage(client, ctx.Dir, nil)
		if err != nil {
			return fmt.Errorf("failed to build wasm: %w", err)
		}
	default:
		return fmt.Errorf("unknown build wasm type: %s", getBuildWasmType(buildArgs))
	}

	// Switch on the type of build for bootloader(from, from_local, build, location)
	var bootloaderResource io.Reader
	switch getBuildType(buildArgs) {
	case "from":
		return fmt.Errorf("from bootloader not implemented yet")
	case "from_local":
		return fmt.Errorf("from local bootloader not implemented yet")
	case "location":
		return fmt.Errorf("location bootloader not implemented yet")
	case "build":
		bootloaderResource, err = buildArgs.Build.BuildImage(client, ctx.Dir, func(tw *tar.Writer) error {
			return utils.WriteTarIntoTar(tw, tar.NewReader(wasmResource), "/wasm")
		})

		if err != nil {
			return fmt.Errorf("failed to build bootloader: %w", err)
		}
	default:
		return fmt.Errorf("unknown build type: %s", getBuildType(buildArgs))
	}

	// Write the layer to the cache
	digest, size, err := cache.Write(bootloaderResource)
	if err != nil {
		return fmt.Errorf("failed to add layer to cache: %w", err)
	}

	log.Infof("Added layer to cache: %s", digest)
	log.Infof("Size: %d", size)

	// Add the layer to the manifest
	oci.AddLayer(manifest, config, oci.MediaTypeImageLayerBootloader, digest, size, &buildArgs.Selector, &buildArgs.Version, nil)

	return nil
}

func parseBootloaderBuildArgs(args any) (BootloaderBuildArgs, error) {
	jsonData, err := json.Marshal(args)
	if err != nil {
		return BootloaderBuildArgs{}, fmt.Errorf("failed to marshal bootloader args: %w", err)
	}

	buildArgs := BootloaderBuildArgs{}
	if err := json.Unmarshal(jsonData, &buildArgs); err != nil {
		return BootloaderBuildArgs{}, fmt.Errorf("failed to unmarshal bootloader args: %w", err)
	}

	switch buildArgs.BuildRaw.(type) {
	case string:
		buildArgs.Build = utils.DockerBuild{
			Context: buildArgs.BuildRaw.(string),
		}
	default:
		jsonData, err := json.Marshal(buildArgs.BuildRaw)
		if err != nil {
			return BootloaderBuildArgs{}, fmt.Errorf("failed to marshal build args: %w", err)
		}
		if err := json.Unmarshal(jsonData, &buildArgs.Build); err != nil {
			return BootloaderBuildArgs{}, fmt.Errorf("failed to unmarshal build args: %w", err)
		}
	}

	switch buildArgs.BuildWasmRaw.(type) {
	case string:
		buildArgs.BuildWasm = utils.DockerBuild{
			Context: buildArgs.BuildWasmRaw.(string),
		}
	default:
		jsonData, err := json.Marshal(buildArgs.BuildWasmRaw)
		if err != nil {
			return BootloaderBuildArgs{}, fmt.Errorf("failed to marshal build wasm args: %w", err)
		}
		if err := json.Unmarshal(jsonData, &buildArgs.BuildWasm); err != nil {
			return BootloaderBuildArgs{}, fmt.Errorf("failed to unmarshal build wasm args: %w", err)
		}
	}

	// Ensure that target is set to dist if not set
	if buildArgs.Build.Target == "" {
		buildArgs.Build.Target = "dist"
	}
	if buildArgs.BuildWasm.Target == "" {
		buildArgs.BuildWasm.Target = "dist"
	}

	// Ensure that dockerfile is set to Dockerfile if not set
	if buildArgs.Build.Dockerfile == "" {
		buildArgs.Build.Dockerfile = "Dockerfile"
	}
	if buildArgs.BuildWasm.Dockerfile == "" {
		buildArgs.BuildWasm.Dockerfile = "Dockerfile"
	}

	return buildArgs, nil
}

func getBuildType(buildArgs BootloaderBuildArgs) string {
	if buildArgs.From != "" {
		return "from"
	}
	if buildArgs.FromLocal != "" {
		return "from_local"
	}
	if buildArgs.Location != "" {
		return "location"
	}
	return "build"
}

func getBuildWasmType(buildArgs BootloaderBuildArgs) string {
	if buildArgs.FromWasm != "" {
		return "from"
	}
	if buildArgs.FromLocalWasm != "" {
		return "from_local"
	}
	if buildArgs.LocationWasm != "" {
		return "location"
	}
	return "build"
}
