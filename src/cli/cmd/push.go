package cmd

import (
	"archive/tar"
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"io"
	"os"

	"github.com/mathiasgredal/neoboot/src/cli/build/cache"
	"github.com/mathiasgredal/neoboot/src/cli/utils"
	"github.com/mathiasgredal/neoboot/src/cli/utils/log"
	"github.com/spf13/cobra"
)

func NewPushCommand(cfg *utils.Config) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "push [options] IMAGE[:TAG]",
		Short: "Push a Neoboot image to a registry",
		Args:  cobra.ExactArgs(1),
		RunE:  func(cmd *cobra.Command, args []string) error { return runPush(cmd, args, cfg) },
	}
	return cmd
}

func runPush(cmd *cobra.Command, args []string, cfg *utils.Config) error {
	image := args[0]

	log.Infof("Pushing image %s", image)

	// Instantiate the image cache
	cache, err := cache.NewCache(cfg.Paths.CacheDir)
	if err != nil {
		return err
	}

	// Get the image info
	imageInfo, err := cache.GetImageInfo(image)
	if err != nil {
		return err
	}

	// Create a tar archive of the image
	buf := bytes.NewBuffer(nil)
	tw := tar.NewWriter(buf)

	// Add the oci-layout file
	ociLayout := []byte(`{"imageLayoutVersion": "1.0.0"}`)
	if err := utils.WriteFileToTar(tw, "oci-layout", ociLayout); err != nil {
		return err
	}

	// Marshal the manifest and write it as a blob into the tar archive using the hash of the blob as the filename
	manifestJSON, err := json.Marshal(imageInfo.Manifest)
	if err != nil {
		return err
	}

	// Write the manifest as a blob into the tar archive using the hash of the blob as the filename
	manifestHash := sha256.Sum256(manifestJSON)
	manifestDigest := hex.EncodeToString(manifestHash[:])
	if err := utils.WriteFileToTar(tw, "blobs/sha256/"+manifestDigest, manifestJSON); err != nil {
		return err
	}

	// Write the config as a blob into the tar archive, we don't hash it as we use the digest from the manifest
	configJSON, err := json.Marshal(imageInfo.Config)
	if err != nil {
		return err
	}
	configDigest := imageInfo.Manifest.Config.Digest
	if err := utils.WriteFileToTar(tw, "blobs/sha256/"+configDigest[7:], configJSON); err != nil {
		return err
	}

	// Add the index.json file
	indexJSON, err := json.Marshal(map[string]interface{}{
		"schemaVersion": 2,
		"mediaType":     "application/vnd.oci.image.index.v1+json",
		"manifests": []interface{}{
			map[string]interface{}{
				"mediaType": "application/vnd.oci.image.manifest.v1+json",
				"digest":    "sha256:" + manifestDigest,
				"size":      len(manifestJSON),
				"annotations": map[string]interface{}{
					"org.opencontainers.image.ref.name": image,
				},
			},
		},
	})
	if err != nil {
		return err
	}

	if err := utils.WriteFileToTar(tw, "index.json", indexJSON); err != nil {
		return err
	}

	// Write all the layers
	for _, layer := range imageInfo.Manifest.Layers {
		layerData, err := cache.ReadLayer(layer.Digest)
		if err != nil {
			return err
		}
		defer layerData.Close()

		layerBytes, err := io.ReadAll(layerData)
		if err != nil {
			return err
		}

		if err := utils.WriteFileToTar(tw, "blobs/sha256/"+layer.Digest[7:], layerBytes); err != nil {
			return err
		}
	}

	// Close the tar writer
	if err := tw.Close(); err != nil {
		return err
	}

	// For now just write the tar to a file
	if err := os.WriteFile("image.tar", buf.Bytes(), 0644); err != nil {
		return err
	}
	// Import the image into the docker daemon

	// Push the image to the destination

	// Clean up the imported image from the docker daemon

	return nil
}

// Layout
// oci-layout
// 	{
// 		"imageLayoutVersion": "1.0.0"
// 	}
// /index.json
// 	{
// 		"schemaVersion": 2,
// 		"mediaType": "application/vnd.oci.image.index.v1+json",
// 		"manifests": [
// 			{
// 				"mediaType": "application/vnd.oci.image.manifest.v1+json",
// 				"digest": "sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
// 				"size": 123456,
// 				"annotations": {
// 					"org.opencontainers.image.ref.name": "my-image:latest"
// 				}
// 			}
// 		]
// 	}
// /blobs/sha256/<digest>
