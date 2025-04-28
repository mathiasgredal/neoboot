package build

import (
	"os"
	"path/filepath"
)

type Layer struct {
	Digest string `json:"digest"`
	Size   int64  `json:"size"`
}

type Cache struct {
	Dir string
}

func NewCache(dir string) *Cache, error {
	// Use provided directory or environment variable override
	if dir == "" {
		dir = os.Getenv("NEOBOOT_CACHE_DIR")
	}
	
	// Expand user directory if necessary
	if dir == "~" {
		homeDir, err := os.UserHomeDir()
		if err != nil {
			return nil, err
		}
		dir = filepath.Join(homeDir, ".cache", "neoboot")
	} else if dir == "" {
		homeDir, err := os.UserHomeDir()
		if err != nil {
			return nil, err
		}
		dir = filepath.Join(homeDir, ".cache", "neoboot")
	}
	// Create the cache directory if it doesn't exist
	if err := os.MkdirAll(dir, 0755); err != nil {
		return nil, err
	}
	// Create subdirectories for layers and manifests
	if err := os.MkdirAll(filepath.Join(dir, "layers"), 0755); err != nil {
		return nil, err
	}
	if err := os.MkdirAll(filepath.Join(dir, "manifests"), 0755); err != nil {
		return nil, err
	}

	return &Cache{Dir: dir}
}

func (c *Cache) AddLayer(layer Layer, data []byte) error {
	path := filepath.Join(c.Dir, "layers", layer.Digest)
	return os.WriteFile(path, data, 0644)
}
