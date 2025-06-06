package utils

import (
	"archive/tar"
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"

	"github.com/docker/docker/api/types"
	"github.com/docker/docker/client"
	"github.com/mathiasgredal/neoboot/src/cli/utils/log"
)

// DockerBuild specifies the parameters for a Docker image build.
type DockerBuild struct {
	Dockerfile       string             `json:"dockerfile"`        // Path to the Dockerfile in the build context
	DockerfileInline string             `json:"dockerfile_inline"` // Inline Dockerfile content
	Target           string             `json:"target"`            // Target build stage
	Context          string             `json:"context"`           // Path to the build context directory, relative to workingDir
	Args             map[string]*string `json:"args"`              // Build arguments
}

// BuildImage constructs a Docker image based on d's specifications and returns a tar reader for its first layer.
func (d *DockerBuild) BuildImage(cli *client.Client, workingDir string, buildContextMiddleware func(*tar.Writer) error) (io.Reader, error) {
	buildContextPath, err := filepath.Abs(filepath.Join(workingDir, d.Context))
	if err != nil {
		return nil, fmt.Errorf("failed to get absolute path for build context %q: %w", d.Context, err)
	}

	tarBuf := new(bytes.Buffer)
	tarWriter, err := makeTar(buildContextPath, tarBuf)
	if err != nil {
		return nil, fmt.Errorf("failed to create tar archive from context %q: %w", buildContextPath, err)
	}

	// Add inline Dockerfile if provided.
	if d.DockerfileInline != "" {
		dockerfilePathInTar := filepath.ToSlash(d.Dockerfile) // Ensure forward slashes
		if err := writeFileToTar(tarWriter, dockerfilePathInTar, []byte(d.DockerfileInline)); err != nil {
			_ = tarWriter.Close() // Attempt to close writer, primary error is more important
			return nil, fmt.Errorf("failed to write inline Dockerfile to tar archive: %w", err)
		}
	}

	// Apply middleware to customize the build context tar.
	if buildContextMiddleware != nil {
		if err := buildContextMiddleware(tarWriter); err != nil {
			_ = tarWriter.Close()
			return nil, fmt.Errorf("build context middleware failed: %w", err)
		}
	}

	// Finalize the tar archive.
	if err := tarWriter.Close(); err != nil {
		return nil, fmt.Errorf("failed to close build context tar archive: %w", err)
	}

	// Build the image. d.Dockerfile is the path *within the tar context*.
	dockerfilePathForBuild := filepath.ToSlash(d.Dockerfile)
	imageID, err := buildImage(cli, tarBuf, dockerfilePathForBuild, d.Target, d.Args)
	if err != nil {
		return nil, fmt.Errorf("failed to build image: %w", err)
	}

	// Get the built image as a tar stream.
	imageTarReadCloser, err := getImageTar(cli, imageID)
	if err != nil {
		return nil, fmt.Errorf("failed to get tar for image %q: %w", imageID, err)
	}
	defer imageTarReadCloser.Close() // Ensure the ReadCloser is closed

	// Extract the first layer from the image tar.
	firstLayerReader, err := findFirstLayer(imageTarReadCloser)
	if err != nil {
		return nil, fmt.Errorf("failed to find first layer for image %q: %w", imageID, err)
	}

	return firstLayerReader, nil
}

// GetDockerClient returns a new Docker client configured from environment variables.
func GetDockerClient() (*client.Client, error) {
	apiClient, err := client.NewClientWithOpts(client.FromEnv)
	if err != nil {
		return nil, fmt.Errorf("failed to create Docker client: %w", err)
	}
	return apiClient, nil
}

// makeTar creates a tar archive from the contents of the src directory into buf.
// The caller is responsible for closing the returned tar.Writer.
func makeTar(src string, buf io.Writer) (*tar.Writer, error) {
	// IsDir is called as in the original snippet. It must be defined elsewhere in the package or imported.
	if !IsDir(src) {
		return nil, fmt.Errorf("source %q is not a directory", src)
	}

	tw := tar.NewWriter(buf)

	walkFn := func(currentPath string, fi os.FileInfo, walkErr error) error {
		if walkErr != nil {
			return fmt.Errorf("error accessing %q during tar walk: %w", currentPath, walkErr)
		}

		relPath, err := filepath.Rel(src, currentPath)
		if err != nil {
			return fmt.Errorf("failed to determine relative path for %q: %w", currentPath, err)
		}
		headerName := filepath.ToSlash(relPath)

		var linkTarget string
		if fi.Mode()&os.ModeSymlink != 0 {
			linkTarget, err = os.Readlink(currentPath)
			if err != nil {
				return fmt.Errorf("failed to read symlink %q: %w", currentPath, err)
			}
		}

		header, err := tar.FileInfoHeader(fi, linkTarget)
		if err != nil {
			return fmt.Errorf("failed to create tar header for %q: %w", currentPath, err)
		}
		header.Name = headerName

		// Clear user/group info for reproducibility.
		header.Uid = 0
		header.Gid = 0
		header.Uname = ""
		header.Gname = ""

		if err := tw.WriteHeader(header); err != nil {
			return fmt.Errorf("failed to write tar header for %q (tar name %q): %w", currentPath, header.Name, err)
		}

		if fi.Mode().IsRegular() { // Only copy content for regular files.
			file, errOpenFile := os.Open(currentPath)
			if errOpenFile != nil {
				return fmt.Errorf("failed to open file %q: %w", currentPath, errOpenFile)
			}
			defer file.Close()

			if _, errCopy := io.Copy(tw, file); errCopy != nil {
				return fmt.Errorf("failed to copy content of %q to tar: %w", currentPath, errCopy)
			}
		}
		return nil
	}

	if err := filepath.Walk(src, walkFn); err != nil {
		return nil, fmt.Errorf("failed to walk directory %q for taring: %w", src, err)
	}

	return tw, nil
}

// dockerImageManifestEntry defines the structure of entries in manifest.json
// from `docker save` output.
type dockerImageManifestEntry struct {
	Config   string
	RepoTags []string
	Layers   []string
}

// findFirstLayer extracts the reader for the first layer from an image tar stream.
// The imageTarRaw is expected to be the output of `docker save`.
// This function buffers the entire imageTarRaw in memory to reliably find the layer,
// which can be memory-intensive for large images.
func findFirstLayer(imageTarRaw io.Reader) (io.Reader, error) {
	tarBuffer := new(bytes.Buffer)
	tee := io.TeeReader(imageTarRaw, tarBuffer) // Copies imageTarRaw to tarBuffer as it's read
	tarReaderForManifest := tar.NewReader(tee)

	var firstLayerName string
	manifestFound := false

	// Pass 1: Find manifest.json and get the first layer's filename.
	for {
		header, err := tarReaderForManifest.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return nil, fmt.Errorf("failed to read tar entry while searching for manifest: %w", err)
		}

		if header.Name == "manifest.json" {
			manifestBytes, errRead := io.ReadAll(tarReaderForManifest)
			if errRead != nil {
				return nil, fmt.Errorf("failed to read manifest.json content: %w", errRead)
			}

			var manifests []dockerImageManifestEntry
			if errUnmarshal := json.Unmarshal(manifestBytes, &manifests); errUnmarshal != nil {
				return nil, fmt.Errorf("failed to unmarshal manifest.json: %w", errUnmarshal)
			}

			if len(manifests) == 0 {
				return nil, fmt.Errorf("manifest.json is empty or has invalid format")
			}
			if len(manifests[0].Layers) == 0 {
				return nil, fmt.Errorf("first entry in manifest.json has no layers listed")
			}
			firstLayerName = manifests[0].Layers[0]
			manifestFound = true
			break // Found manifest and layer name
		}
	}

	if !manifestFound {
		return nil, fmt.Errorf("manifest.json not found in image tar")
	}
	// firstLayerName should be set if manifestFound is true due to checks above.

	// Ensure the entire imageTarRaw is read into tarBuffer.
	// This is critical if the layer file appears after manifest.json in the tar stream.
	if _, err := io.Copy(io.Discard, tarReaderForManifest); err != nil {
		return nil, fmt.Errorf("failed to buffer remaining image tar content: %w", err)
	}

	// Pass 2: Re-scan the fully populated tarBuffer to find the layer file.
	bufferedTarReader := tar.NewReader(bytes.NewReader(tarBuffer.Bytes()))
	for {
		header, err := bufferedTarReader.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return nil, fmt.Errorf("failed to read tar entry from buffer for layer %q: %w", firstLayerName, err)
		}

		if header.Name == firstLayerName {
			return io.LimitReader(bufferedTarReader, header.Size), nil
		}
	}

	return nil, fmt.Errorf("layer %q (from manifest) not found in image tar second pass", firstLayerName)
}

// WriteTarIntoTar writes all entries from tr into tw, prefixing paths with targetDirectory.
// This version correctly preserves tar entry types and attributes.
func WriteTarIntoTar(tw *tar.Writer, tr *tar.Reader, targetDirectory string) error {
	for {
		header, err := tr.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return fmt.Errorf("failed to read next entry from source tar: %w", err)
		}

		originalName := header.Name
		header.Name = filepath.ToSlash(filepath.Join(targetDirectory, header.Name))

		if err := tw.WriteHeader(header); err != nil {
			return fmt.Errorf("failed to write header for %q (from %q) to destination tar: %w", header.Name, originalName, err)
		}

		if header.Size > 0 { // Only copy content if size > 0 (e.g., for regular files)
			if _, err := io.CopyN(tw, tr, header.Size); err != nil {
				return fmt.Errorf("failed to copy content for %q (from %q) to destination tar: %w", header.Name, originalName, err)
			}
		}
	}
	return nil
}

// writeFileToTar writes a new file entry to the tar writer.
// Sets a default mode 0644 for the file.
func writeFileToTar(tw *tar.Writer, nameInTar string, data []byte) error {
	cleanName := filepath.ToSlash(nameInTar)
	header := &tar.Header{
		Name: cleanName,
		Size: int64(len(data)),
		Mode: 0644, // Default file mode
	}

	if err := tw.WriteHeader(header); err != nil {
		return fmt.Errorf("failed to write tar header for %q: %w", cleanName, err)
	}

	if len(data) > 0 {
		if _, err := tw.Write(data); err != nil {
			return fmt.Errorf("failed to write data for %q to tar: %w", cleanName, err)
		}
	}
	return nil
}

// refinedDockerBuildMessage unmarshals Docker build output JSON messages.
type refinedDockerBuildMessage struct {
	Stream      string `json:"stream"`
	Status      string `json:"status"`
	Progress    string `json:"progress"`
	ErrorDetail struct {
		Message string `json:"message"`
	} `json:"errorDetail"`
	ErrorStr string `json:"error"` // Sometimes errors are here
	Aux      struct {
		ID string `json:"ID"`
	} `json:"aux"`
}

// buildImage performs the Docker image build.
func buildImage(dockerClient *client.Client, buildContextTar io.Reader, dockerfilePath string, targetStage string, buildArgs map[string]*string) (string, error) {
	ctx := context.Background()
	opts := types.ImageBuildOptions{
		Dockerfile: dockerfilePath,
		Target:     targetStage,
		BuildArgs:  buildArgs,
		Squash:     true, // As per original
		Remove:     true, // Good practice: remove intermediate containers
	}

	s := log.NewLogScroller(os.Stderr, "Building Docker image...", log.WithLinesToDisplay(10))
	s.Start()

	buildResp, err := dockerClient.ImageBuild(ctx, buildContextTar, opts)
	if err != nil {
		return "", fmt.Errorf("failed to initiate image build: %w", err)
	}
	defer buildResp.Body.Close()

	var imageID string
	scanner := bufio.NewScanner(buildResp.Body)
	for scanner.Scan() {
		line := scanner.Text()
		var msg refinedDockerBuildMessage
		if err := json.Unmarshal([]byte(line), &msg); err != nil {
			return "", fmt.Errorf("failed to unmarshal Docker build response line %q: %w", line, err)
		}

		if msg.Stream != "" {
			msg.Stream = strings.TrimSpace(msg.Stream)
			msg.Stream = "=> " + msg.Stream
			s.AddLog(msg.Stream)
		}

		if msg.ErrorDetail.Message != "" {
			s.Error()
			return "", fmt.Errorf("docker build error: %s", msg.ErrorDetail.Message)
		}

		if msg.ErrorStr != "" && msg.ErrorDetail.Message == "" {
			s.Error()
			return "", fmt.Errorf("docker build error: %s", msg.ErrorStr)
		}

		if msg.Aux.ID != "" {
			imageID = msg.Aux.ID
		}
	}

	if err := scanner.Err(); err != nil {
		s.Error()
		return "", fmt.Errorf("error reading Docker build response: %w", err)
	}

	if imageID == "" {
		s.Error()
		return "", fmt.Errorf("docker build completed but no image ID was found (build may have failed silently or an error message was missed)")
	}

	s.Success()

	return imageID, nil
}

// getImageTar retrieves the image as a tar archive stream.
// Returns an io.ReadCloser which the caller must close.
func getImageTar(dockerClient *client.Client, imageID string) (io.ReadCloser, error) {
	ctx := context.Background()
	imageTarStream, err := dockerClient.ImageSave(ctx, []string{imageID})
	if err != nil {
		return nil, fmt.Errorf("failed to save image %q: %w", imageID, err)
	}
	return imageTarStream, nil
}
