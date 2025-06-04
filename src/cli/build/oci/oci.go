package oci

type MediaType string

const (
	MediaTypeImageManifest MediaType = "application/vnd.oci.image.manifest.v1+json"
	MediaTypeImageConfig   MediaType = "application/vnd.oci.image.config.v1+json"
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

type Config struct {
	Created string      `json:"created"`
	Author  string      `json:"author"`
	OS      string      `json:"os"`
	Config  ImageConfig `json:"config"`
	Rootfs  ImageRootfs `json:"rootfs"`
}

func NewManifest() *Manifest {
	return &Manifest{
		SchemaVersion: 2,
		MediaType:     MediaTypeImageManifest,
	}
}

// func (m *Manifest) AddConfig(cache *Cache, config Config) error {
// 	config.Rootfs.DiffIDs = []string{}

// 	// Serialize the config to JSON
// 	configJSON, err := json.Marshal(config)
// 	if err != nil {
// 		return err
// 	}

// 	// Write the config to the cache
// 	digest, size, err := cache.Write(bytes.NewReader(configJSON))
// 	if err != nil {
// 		return err
// 	}

// 	// Add the config to the manifest
// 	m.Config = Descriptor{
// 		MediaType: MediaTypeImageConfig,
// 		Digest:    digest,
// 		Size:      size,
// 	}

// 	return nil
// }
