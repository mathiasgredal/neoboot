package build

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

	"github.com/gofrs/flock"
	log "github.com/sirupsen/logrus"
)

type Layer struct {
	ID       string `json:"id"`
	CacheID  string `json:"cache_id"` // ID for the content in cache (e.g., sha256:...)
	Digest   string `json:"digest"`   // OCI content digest (often same as CacheID)
	Size     int64  `json:"size"`
	Created  string `json:"created"`   // RFC3339Nano format
	LastRead string `json:"last_read"` // RFC3339Nano format
}

// LayersMetadata defines the structure of layers.json
type LayersMetadata struct {
	Layers map[string]Layer `json:"layers"`
}

type Cache struct {
	Dir           string
	layersDir     string
	manifestsDir  string
	layersFile    string // path to layers.json
	lockFile      string // path to layers.lock for FS-level lock on layers.json modification
	flock         *flock.Flock
	metadata      LayersMetadata
	metadataMutex sync.RWMutex // Mutex for in-memory metadata access
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

	log.Infof("Using cache directory: %s, layers metadata file: %s", dir, c.layersFile)

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
			log.Infof("layers.json not found at %s, initializing new metadata structure.", c.layersFile)
			c.metadata.Layers = make(map[string]Layer)
			return nil
		}
		return fmt.Errorf("failed to read %s: %w", c.layersFile, err)
	}

	if len(data) == 0 {
		log.Infof("layers.json at %s is empty, initializing new metadata structure.", c.layersFile)
		c.metadata.Layers = make(map[string]Layer)
		return nil
	}

	if err := json.Unmarshal(data, &c.metadata); err != nil {
		log.Warnf("Failed to unmarshal %s: %v. Initializing with empty metadata.", c.layersFile, err)
		c.metadata.Layers = make(map[string]Layer)
		// Depending on strictness, could return: fmt.Errorf("corrupt metadata file %s: %w", c.layersFile, err)
		return nil
	}

	if c.metadata.Layers == nil { // Handle JSON like {"layers": null}
		c.metadata.Layers = make(map[string]Layer)
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

	log.Debugf("Successfully saved layers metadata to %s", c.layersFile)
	return nil
}

// Write a blob to the cache, update layers.json, and return the digest (CacheID).
func (c *Cache) Write(blobReader io.Reader) (string, error) {
	tempBlobFile, err := os.CreateTemp(c.layersDir, "tmp-blob-*")
	if err != nil {
		return "", fmt.Errorf("failed to create temp blob file in %s: %w", c.layersDir, err)
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
		return "", fmt.Errorf("failed to copy blob content to %s: %w", tempBlobPath, err)
	}

	// Content is in tempBlobFile. Close it before further operations like rename or stat.
	if err := tempBlobFile.Close(); err != nil {
		// If close fails here, the defer will also try to close it.
		return "", fmt.Errorf("failed to close temp blob file %s after writing: %w", tempBlobPath, err)
	}

	digestSum := hasher.Sum(nil)
	hexDigest := hex.EncodeToString(digestSum)
	layerID := fmt.Sprintf("sha256:%s", hexDigest) // Used as Layer.ID, Layer.CacheID, Layer.Digest

	// Filename for the blob uses a dash instead of colon for better filesystem compatibility.
	blobFileName := "sha256-" + hexDigest
	finalBlobPath := filepath.Join(c.layersDir, blobFileName)

	// Check if blob already exists.
	if _, statErr := os.Stat(finalBlobPath); statErr == nil {
		log.Infof("Layer blob %s (file: %s) already exists. Discarding temp file %s.", layerID, blobFileName, tempBlobPath)
		// Temp file is redundant. Remove it explicitly.
		if err := os.Remove(tempBlobPath); err != nil && !os.IsNotExist(err) {
			log.Warnf("Error removing already existing temporary blob file %s: %v", tempBlobPath, err)
		}
		tempFileHandled = true
	} else if os.IsNotExist(statErr) {
		// Blob does not exist, rename temp file to its final path.
		if err := os.Rename(tempBlobPath, finalBlobPath); err != nil {
			// If rename fails, tempBlobPath still exists; defer will attempt cleanup.
			return "", fmt.Errorf("failed to rename temp blob %s to %s: %w", tempBlobPath, finalBlobPath, err)
		}
		tempFileHandled = true // Renamed, so tempBlobPath is gone.
		log.Infof("Stored layer blob %s to %s", layerID, finalBlobPath)
	} else {
		// Some other error occurred during stat. Defer will attempt cleanup of tempBlobPath.
		return "", fmt.Errorf("failed to stat final blob path %s: %w", finalBlobPath, statErr)
	}

	// Update layers.json metadata
	now := time.Now().UTC().Format(time.RFC3339Nano)

	c.metadataMutex.Lock()
	// Ensure Layers map is initialized (it should be by loadMetadata, but defensive check).
	if c.metadata.Layers == nil {
		c.metadata.Layers = make(map[string]Layer)
	}

	existingLayer, found := c.metadata.Layers[layerID]
	if found {
		existingLayer.LastRead = now
		if existingLayer.Size != size && size != 0 { // Size might be 0 for an empty blob
			log.Warnf("Size mismatch for existing layer %s: metadata %d, new %d. Keeping original metadata size.", layerID, existingLayer.Size, size)
		}
		c.metadata.Layers[layerID] = existingLayer
	} else {
		newLayer := Layer{
			ID:       layerID,
			CacheID:  layerID,
			Digest:   layerID,
			Size:     size,
			Created:  now,
			LastRead: now,
		}
		c.metadata.Layers[layerID] = newLayer
	}
	c.metadataMutex.Unlock()

	if err := c.saveMetadata(); err != nil {
		// Blob operation successful, but metadata save failed. Cache might be inconsistent.
		return "", fmt.Errorf("blob %s processed, but failed to save metadata: %w", layerID, err)
	}

	return layerID, nil
}
