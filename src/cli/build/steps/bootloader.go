package steps

import (
	"archive/tar"
	"encoding/json"
	"fmt"

	"github.com/mathiasgredal/neoboot/src/cli/build/cache"
	"github.com/mathiasgredal/neoboot/src/cli/build/oci"
	"github.com/mathiasgredal/neoboot/src/cli/utils"
)

// Handle the BOOTLOADER command
// TODO:
// - Support caching
// - Support build args
// - Support build_wasm
// - Support build_dockerfile
// - Support build_dockerfile_inline
// - Support different sources for the bootloader
// - Support var substitution in the bootloader build args

type BootloaderBuildArgs struct {
	Type          string      `json:"type"`
	Selector      string      `json:"selector"`
	Version       string      `json:"version"`
	From          string      `json:"from"`
	FromLocal     string      `json:"from_local"`
	BuildRaw      any         `json:"build"`
	Build         DockerBuild `json:"-"`
	Location      string      `json:"location"`
	FromWasm      string      `json:"from_wasm"`
	FromLocalWasm string      `json:"from_local_wasm"`
	BuildWasmRaw  any         `json:"build_wasm"`
	BuildWasm     DockerBuild `json:"-"`
	LocationWasm  string      `json:"location_wasm"`
}

func HandleBootloader(cache *cache.Cache, manifest *oci.Manifest, args any) error {
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
	var wasmResource *tar.Reader
	switch getBuildWasmType(buildArgs) {
	case "from":
		return fmt.Errorf("from wasm not implemented yet")
	case "from_local":
		return fmt.Errorf("from local wasm not implemented yet")
	case "location":
		return fmt.Errorf("location wasm not implemented yet")
	case "build":
		wasmResource, err = buildArgs.BuildWasm.BuildImage(client, buildArgs.BuildWasm.Context, func(tw *tar.Writer) error {
			return nil
		})
		if err != nil {
			return fmt.Errorf("failed to build wasm: %w", err)
		}
	default:
		return fmt.Errorf("unknown build wasm type: %s", getBuildWasmType(buildArgs))
	}

	// Switch on the type of build for bootloader(from, from_local, build, location)
	var bootloaderResource *tar.Reader
	switch getBuildType(buildArgs) {
	case "from":
		return fmt.Errorf("from bootloader not implemented yet")
	case "from_local":
		return fmt.Errorf("from local bootloader not implemented yet")
	case "location":
		return fmt.Errorf("location bootloader not implemented yet")
	case "build":
		bootloaderResource, err = buildArgs.Build.BuildImage(client, buildArgs.Build.Context, func(tw *tar.Writer) error {
			return utils.WriteTarIntoTar(tw, wasmResource, "/wasm")
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

	fmt.Printf("Added layer to cache: %s\n", digest)

	// Add the layer to the manifest
	manifest.Layers = append(manifest.Layers, oci.Descriptor{
		Digest: digest,
		Size:   size,
	})

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
		buildArgs.Build = DockerBuild{
			Context: buildArgs.BuildRaw.(string),
		}
	case DockerBuild:
		buildArgs.Build = buildArgs.BuildRaw.(DockerBuild)
	default:
		return BootloaderBuildArgs{}, fmt.Errorf("invalid build type")
	}

	switch buildArgs.BuildWasmRaw.(type) {
	case string:
		buildArgs.BuildWasm = DockerBuild{
			Context: buildArgs.BuildWasmRaw.(string),
		}
	case DockerBuild:
		buildArgs.BuildWasm = buildArgs.BuildWasmRaw.(DockerBuild)
	default:
		return BootloaderBuildArgs{}, fmt.Errorf("invalid build wasm type")
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

// // Switch on the type of build for bootloader(from, from_local, build, location)
// // Add the wasm to the context of the bootloader build

// // Make the bootload build args context directory absolute and relative to the builder context directory
// buildContext, err := filepath.Abs(filepath.Join(b.context.Dir, buildArgs.Context))
// if err != nil {
// 	return fmt.Errorf("failed to get absolute path: %w", err)
// }
// buildArgs.Context = buildContext

// fmt.Printf("Building bootloader with context: %s\n", buildArgs.Context)

// // Create a tar archive, with the context directory and a Dockerfile
// buf := bytes.NewBuffer(nil)
// tw, err := utils.MakeTar(b.context.Dir, buildArgs.Context, buf)
// if err != nil {
// 	return fmt.Errorf("failed to create tar archive: %w", err)
// }

// // Write the Dockerfile to the tar archive
// // TODO: Check if buildargs is nil
// dockerfile := fmt.Sprintf("FROM %s as builder\n COPY ./bootloader/patches ./yeet\nFROM scratch as dist\nCOPY --from=builder /yeet /yeet\nCOPY --from=builder /yeet /yeet2\nCOPY --from=builder /yeet /yeet3\n", buildArgs.Builder)
// if err := utils.WriteFileToTar(tw, "Dockerfile", []byte(dockerfile)); err != nil {
// 	return fmt.Errorf("failed to write Dockerfile to tar archive: %w", err)
// }

// // Close the tar archive
// if err := tw.Close(); err != nil {
// 	return fmt.Errorf("failed to close tar archive: %w", err)
// }

// client, err := utils.GetDockerClient()
// if err != nil {
// 	return fmt.Errorf("failed to create docker client: %w", err)
// }
// imageID, err := utils.BuildImage(client, buf)
// if err != nil {
// 	return fmt.Errorf("failed to build image: %w", err)
// }

// fmt.Printf("Built image: %s\n", imageID)

// image_tar_raw, err := utils.GetImageTar(client, imageID)
// if err != nil {
// 	return fmt.Errorf("failed to get image tar: %w", err)
// }

// // Go through the tar and find the manifest.json, and identify the layer
// layer, err := FindFirstLayer(image_tar_raw)
// if err != nil {
// 	return fmt.Errorf("failed to find manifest: %w", err)
// }
