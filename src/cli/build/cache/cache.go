package cache

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"

	"github.com/distribution/reference"
	"github.com/gofrs/flock"
	"github.com/mathiasgredal/neoboot/src/cli/build/oci"
	"github.com/mathiasgredal/neoboot/src/cli/utils/log"
)

type Layer struct {
	ID       string `json:"id"`
	CacheID  string `json:"cache_id"`
	Size     int64  `json:"size"`
	Created  string `json:"created"`
	LastRead string `json:"last_read"`
}

type LayersMetadata struct {
	Layers []Layer `json:"layers"`
}

type Cache struct {
	Dir           string
	layersDir     string
	manifestsDir  string
	layersFile    string
	lockFile      string
	flock         *flock.Flock
	metadata      LayersMetadata
	metadataMutex sync.RWMutex
}

func NewCache(dir string) (*Cache, error) {
	if strings.HasPrefix(dir, "~/") {
		home, err := os.UserHomeDir()
		if err != nil {
			return nil, fmt.Errorf("failed to get user home directory: %w", err)
		}
		dir = filepath.Join(home, dir[2:])
	}

	// Ensure base cache directory exists
	if err := os.MkdirAll(dir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create cache directory %s: %w", dir, err)
	}

	layersDir := filepath.Join(dir, "layers")
	manifestsDir := filepath.Join(dir, "manifests")

	// Create subdirectories for layers and manifests
	if err := os.MkdirAll(layersDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create layers subdirectory %s: %w", layersDir, err)
	}
	if err := os.MkdirAll(manifestsDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create manifests subdirectory %s: %w", manifestsDir, err)
	}

	c := &Cache{
		Dir:           dir,
		layersDir:     layersDir,
		manifestsDir:  manifestsDir,
		layersFile:    filepath.Join(layersDir, "layers.json"),
		lockFile:      filepath.Join(layersDir, "layers.lock"),
		metadataMutex: sync.RWMutex{},
	}
	c.flock = flock.New(c.lockFile)

	// Load initial metadata
	if err := c.loadMetadata(); err != nil {
		return nil, fmt.Errorf("failed to initialize or load cache metadata: %w", err)
	}

	log.Debugf("Using cache directory: %s, layers metadata file: %s", dir, c.layersFile)

	return c, nil
}

// loadMetadata reads layers.json from disk into memory.
// It acquires a file lock to ensure consistent reads.
func (c *Cache) loadMetadata() error {
	if err := c.flock.Lock(); err != nil {
		return fmt.Errorf("failed to acquire lock for %s: %w", c.lockFile, err)
	}
	defer func() {
		if err := c.flock.Unlock(); err != nil {
			log.Errorf("Failed to unlock %s: %v", c.lockFile, err)
		}
	}()

	c.metadataMutex.Lock() // Lock for writing to c.metadata
	defer c.metadataMutex.Unlock()

	data, err := os.ReadFile(c.layersFile)
	if err != nil {
		if os.IsNotExist(err) {
			log.Warnf("layers.json not found at %s, initializing new metadata structure.", c.layersFile)
			c.metadata.Layers = []Layer{}
			return nil
		}
		return fmt.Errorf("failed to read %s: %w", c.layersFile, err)
	}

	if len(data) == 0 {
		log.Infof("layers.json at %s is empty, initializing new metadata structure.", c.layersFile)
		c.metadata.Layers = []Layer{}
		return nil
	}

	if err := json.Unmarshal(data, &c.metadata); err != nil {
		log.Warnf("Failed to unmarshal %s: %v. Initializing with empty metadata.", c.layersFile, err)
		c.metadata.Layers = []Layer{}
		// Depending on strictness, could return: fmt.Errorf("corrupt metadata file %s: %w", c.layersFile, err)
		return nil
	}

	if c.metadata.Layers == nil { // Handle JSON like {"layers": null}
		c.metadata.Layers = []Layer{}
	}

	log.Debugf("Loaded %d layer metadata entries from %s", len(c.metadata.Layers), c.layersFile)
	return nil
}

// saveMetadata writes the in-memory layers.json to disk.
// It acquires a file lock to prevent concurrent writes from other processes.
func (c *Cache) saveMetadata() error {
	if err := c.flock.Lock(); err != nil {
		return fmt.Errorf("failed to acquire lock for %s: %w", c.lockFile, err)
	}
	defer func() {
		if err := c.flock.Unlock(); err != nil {
			log.Errorf("Failed to unlock %s: %v", c.lockFile, err)
		}
	}()

	c.metadataMutex.RLock() // Use RLock as we are only reading c.metadata for marshalling
	jsonData, err := json.MarshalIndent(c.metadata, "", "  ")
	c.metadataMutex.RUnlock()

	if err != nil {
		return fmt.Errorf("failed to marshal layers metadata: %w", err)
	}

	tempLayersFilePath := c.layersFile + ".tmp"
	if err := os.WriteFile(tempLayersFilePath, jsonData, 0644); err != nil {
		return fmt.Errorf("failed to write temporary metadata to %s: %w", tempLayersFilePath, err)
	}

	if err := os.Rename(tempLayersFilePath, c.layersFile); err != nil {
		if removeErr := os.Remove(tempLayersFilePath); removeErr != nil {
			log.Warnf("Failed to remove temporary metadata file %s after rename error: %v", tempLayersFilePath, removeErr)
		}
		return fmt.Errorf("failed to rename temporary metadata %s to %s: %w", tempLayersFilePath, c.layersFile, err)
	}

	return nil
}

// Write a blob to the cache, update layers.json, and return the digest.
func (c *Cache) Write(blobReader io.Reader) (string, int64, error) {
	tempBlobFile, err := os.CreateTemp(c.layersDir, "tmp-blob-*")
	if err != nil {
		return "", 0, fmt.Errorf("failed to create temp blob file in %s: %w", c.layersDir, err)
	}
	tempBlobPath := tempBlobFile.Name()

	// Defer ensures the temp file is closed and removed if not successfully processed (renamed or explicitly deleted).
	tempFileHandled := false
	defer func() {
		// Close can be called multiple times; subsequent calls on os.File return error.
		_ = tempBlobFile.Close()
		if !tempFileHandled {
			if err := os.Remove(tempBlobPath); err != nil && !os.IsNotExist(err) {
				log.Warnf("Deferred removal of temporary blob file %s failed: %v", tempBlobPath, err)
			}
		}
	}()

	hasher := sha256.New()
	teeReader := io.TeeReader(blobReader, hasher)

	size, err := io.Copy(tempBlobFile, teeReader)
	if err != nil {
		return "", 0, fmt.Errorf("failed to copy blob content to %s: %w", tempBlobPath, err)
	}

	// Content is in tempBlobFile. Close it before further operations like rename or stat.
	if err := tempBlobFile.Close(); err != nil {
		// If close fails here, the defer will also try to close it.
		return "", 0, fmt.Errorf("failed to close temp blob file %s after writing: %w", tempBlobPath, err)
	}

	digestSum := hasher.Sum(nil)
	hexDigest := hex.EncodeToString(digestSum)
	layerID := fmt.Sprintf("sha256-%s", hexDigest) // Used as Layer.ID, Layer.CacheID

	// Filename for the blob uses a dash instead of colon for better filesystem compatibility.
	finalBlobPath := filepath.Join(c.layersDir, layerID)

	// Check if blob already exists.
	if _, statErr := os.Stat(finalBlobPath); statErr == nil {
		log.Infof("Layer blob %s (file: %s) already exists. Discarding temp file %s.", layerID, finalBlobPath, tempBlobPath)
		// Temp file is redundant. Remove it explicitly.
		if err := os.Remove(tempBlobPath); err != nil && !os.IsNotExist(err) {
			log.Warnf("Error removing already existing temporary blob file %s: %v", tempBlobPath, err)
		}
		tempFileHandled = true
	} else if os.IsNotExist(statErr) {
		// Blob does not exist, rename temp file to its final path.
		if err := os.Rename(tempBlobPath, finalBlobPath); err != nil {
			// If rename fails, tempBlobPath still exists; defer will attempt cleanup.
			return "", 0, fmt.Errorf("failed to rename temp blob %s to %s: %w", tempBlobPath, finalBlobPath, err)
		}
		tempFileHandled = true // Renamed, so tempBlobPath is gone.
		log.Tracef("Stored layer blob %s to %s", layerID, finalBlobPath)
	} else {
		// Some other error occurred during stat. Defer will attempt cleanup of tempBlobPath.
		return "", 0, fmt.Errorf("failed to stat final blob path %s: %w", finalBlobPath, statErr)
	}

	// Update layers.json metadata
	now := time.Now().UTC().Format(time.RFC3339Nano)

	// Lock the metadata for writing
	log.Tracef("Locking metadata for writing")
	c.metadataMutex.Lock()

	// Ensure Layers map is initialized (it should be by loadMetadata, but defensive check).
	if c.metadata.Layers == nil {
		c.metadata.Layers = []Layer{}
	}

	// Find if layer exists in the metadata
	layerIndex := -1
	for i, layer := range c.metadata.Layers {
		if layer.ID == layerID {
			layerIndex = i
			break
		}
	}

	// If layer exists, update the metadata
	if layerIndex != -1 {
		c.metadata.Layers[layerIndex].LastRead = now
		if c.metadata.Layers[layerIndex].Size != size && size != 0 { // Size might be 0 for an empty blob
			log.Warnf("Size mismatch for existing layer %s: metadata %d, new %d. Keeping original metadata size.", layerID, c.metadata.Layers[layerIndex].Size, size)
		}
		c.metadataMutex.Unlock()
		return layerID, size, nil
	} else {
		newLayer := Layer{
			ID:       layerID,
			CacheID:  layerID, // TODO: Change this to the cache ID
			Size:     size,
			Created:  now,
			LastRead: now,
		}
		c.metadata.Layers = append(c.metadata.Layers, newLayer)
	}

	c.metadataMutex.Unlock()

	if err := c.saveMetadata(); err != nil {
		// Blob operation successful, but metadata save failed. Cache might be inconsistent.
		return "", 0, fmt.Errorf("blob %s processed, but failed to save metadata: %w", layerID, err)
	}

	return layerID, size, nil
}

func (c *Cache) ReadLayer(layerID string) (io.ReadCloser, error) {
	// Try to be smart about the layer ID, such as handling sha256- prefix and if the seperator is a : instead of a -
	if !strings.HasPrefix(layerID, "sha256") {
		layerID = "sha256-" + layerID
	}
	layerID = strings.ReplaceAll(layerID, ":", "-")
	layerPath := filepath.Join(c.layersDir, layerID)
	return os.Open(layerPath)
}

func (c *Cache) WriteManifest(name string, manifest *oci.Manifest) error {
	manifestJSON, err := json.Marshal(manifest)
	if err != nil {
		return fmt.Errorf("failed to marshal manifest: %w", err)
	}

	named, err := reference.ParseNormalizedNamed(name)
	if err != nil {
		return fmt.Errorf("invalid tag format: %s", name)
	}

	// Write the manifest to the cache, using this folder structure:
	manifestDir := filepath.Join(c.manifestsDir, reference.Domain(named), reference.Path(named))
	if err := os.MkdirAll(manifestDir, 0755); err != nil {
		return fmt.Errorf("failed to create manifest image directory %s: %w", manifestDir, err)
	}
	return os.WriteFile(filepath.Join(manifestDir, named.(reference.Tagged).Tag()), manifestJSON, 0644)
}

func (c *Cache) ListImages() ([]string, error) {
	var images []string

	walkFn := func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return fmt.Errorf("failed to walk manifest directory %s: %w", path, err)
		}

		if info.IsDir() {
			return nil
		}

		// Subtract the manifests directory from the path and remove the leading slash
		path = strings.TrimPrefix(path, c.manifestsDir)
		path = strings.TrimPrefix(path, string(os.PathSeparator))

		// Extract the path and tag from the path
		pathParts := strings.Split(path, string(os.PathSeparator))
		if len(pathParts) < 2 {
			return fmt.Errorf("invalid manifest path %s", path)
		}

		imagePath := pathParts[0 : len(pathParts)-1]
		tag := pathParts[len(pathParts)-1]

		images = append(images, fmt.Sprintf("%s:%s", strings.Join(imagePath, "/"), tag))

		return nil
	}

	if err := filepath.Walk(c.manifestsDir, walkFn); err != nil {
		return nil, err
	}

	return images, nil
}

type ImageInfo struct {
	Name        string
	Tag         string
	ShortDigest string
	Size        int64
	Created     time.Time
	Config      oci.Config
	Manifest    oci.Manifest
}

func (c *Cache) GetImageInfo(name string) (*ImageInfo, error) {
	// Read the image manifest by parsing the name
	namedImage, err := reference.ParseNormalizedNamed(name)
	if err != nil {
		return nil, fmt.Errorf("failed to parse image name: %s, %w", name, err)
	}

	// Ensure the name has a tag
	namedImage = reference.TagNameOnly(namedImage)

	// Get the repository and image name
	repository := reference.Domain(namedImage)
	image := reference.Path(namedImage)
	tag := namedImage.(reference.Tagged).Tag()

	// Read the manifest
	manifestPath := filepath.Join(c.manifestsDir, repository, image, tag)
	manifestJSON, err := os.ReadFile(manifestPath)
	if err != nil {
		return nil, fmt.Errorf("failed to read manifest: %w", err)
	}

	var manifest oci.Manifest
	if err := json.Unmarshal(manifestJSON, &manifest); err != nil {
		return nil, fmt.Errorf("failed to unmarshal manifest: %w", err)
	}

	// Get the config
	configJSON, err := c.ReadLayer(manifest.Config.Digest[7:])
	if err != nil {
		return nil, fmt.Errorf("failed to read config: %w", err)
	}
	defer configJSON.Close()

	var config oci.Config
	if err := json.NewDecoder(configJSON).Decode(&config); err != nil {
		return nil, fmt.Errorf("failed to unmarshal config: %w", err)
	}

	// Get the created time
	created := time.Time{}
	if config.Created != "" {
		created, err = time.Parse(time.RFC3339Nano, config.Created)
		if err != nil {
			return nil, fmt.Errorf("failed to parse created time: %w", err)
		}
	}

	// Sum the size of all the layers and the config
	var size int64
	for _, layer := range manifest.Layers {
		size += layer.Size
	}
	size += manifest.Config.Size

	// Get the image info
	imageInfo := &ImageInfo{
		Name:        repository + "/" + image,
		Tag:         tag,
		ShortDigest: manifest.Config.Digest[7:19],
		Size:        size,
		Created:     created,
		Config:      config,
		Manifest:    manifest,
	}
	return imageInfo, nil
}
