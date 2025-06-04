package oci

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
	manifest.Layers = append(manifest.Layers, Descriptor{
		MediaType: mediaType,
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
