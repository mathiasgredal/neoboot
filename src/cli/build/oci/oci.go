package oci

import (
	"fmt"
	"strings"
)

type MediaType string

const (
	MediaTypeImageManifest        MediaType = "application/vnd.oci.image.manifest.v1+json"
	MediaTypeImageConfig          MediaType = "application/vnd.oci.image.config.v1+json"
	MediaTypeImageLayerBootloader MediaType = "application/vnd.oci.image.layer.bootloader.v1+json"
)

type Manifest struct {
	SchemaVersion int          `json:"schemaVersion"`
	MediaType     MediaType    `json:"mediaType"`
	Config        Descriptor   `json:"config"`
	Layers        []Descriptor `json:"layers"`
}

type Descriptor struct {
	MediaType MediaType `json:"mediaType"`
	Digest    string    `json:"digest"`
	Size      int64     `json:"size"`
}

type ImageConfig struct {
	State string `json:"bootfile_state"`
}

type ImageRootfs struct {
	Type    string   `json:"type"`
	DiffIDs []string `json:"diff_ids"`
}

type LayerMeta struct {
	MediaType         MediaType `json:"media_type"`
	Selector          *string   `json:"selector"`
	Version           *string   `json:"version"`
	LocationDirective *string   `json:"location"`
}

type Config struct {
	Created   string      `json:"created"`
	Author    string      `json:"author"`
	OS        string      `json:"os"`
	Config    ImageConfig `json:"config"`
	Rootfs    ImageRootfs `json:"rootfs"`
	LayerMeta []LayerMeta `json:"layer_meta"`
}

func AddLayer(manifest *Manifest, config *Config, mediaType MediaType, digest string, size int64, selector *string, version *string, locationDirective *string) {
	fmt.Println("Adding layer with digest", digest)
	// If the digest doesnt start with sha256, prefix it
	if !strings.HasPrefix(digest, "sha256") {
		digest = "sha256:" + digest
	}

	// If the digest contains a dash, replace it with a colon
	if strings.Contains(digest, "-") {
		digest = strings.ReplaceAll(digest, "-", ":")
	}

	fmt.Println("Adding layer with digest", digest)

	// Add the layer to the manifest
	manifest.Layers = append(manifest.Layers, Descriptor{
		MediaType: mediaType, // Only write the vendor specific media type to the meta
		Digest:    digest,
		Size:      size,
	})

	config.Rootfs.DiffIDs = append(config.Rootfs.DiffIDs, digest)

	config.LayerMeta = append(config.LayerMeta, LayerMeta{
		MediaType:         mediaType,
		Selector:          selector,
		Version:           version,
		LocationDirective: locationDirective,
	})
}
