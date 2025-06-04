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
)

// GetDockerClient returns a docker client from the environment
func GetDockerClient() (*client.Client, error) {
	apiClient, err := client.NewClientWithOpts(client.FromEnv)
	if err != nil {
		return nil, fmt.Errorf("failed to create docker client: %w", err)
	}
	return apiClient, nil
}

// MakeTar creates a tar archive from a directory
func MakeTar(src string, buf io.Writer) (*tar.Writer, error) {
	tw := tar.NewWriter(buf)

	// Raise error if src is not a directory
	if !IsDir(src) {
		return nil, fmt.Errorf("src is not a directory: %s", src)
	}

	// walk through every file in the folder
	filepath.Walk(src, func(file string, fi os.FileInfo, err error) error {
		// Make the file path relative to the base directory, by removing the base directory from the file path
		relativePath := strings.TrimPrefix(file, src)

		// Generate tar header
		header, err := tar.FileInfoHeader(fi, relativePath)
		if err != nil {
			return err
		}

		// Must provide real name
		header.Name = filepath.ToSlash(relativePath)

		// Write header
		if err := tw.WriteHeader(header); err != nil {
			return err
		}

		// If not a dir, write file content
		if !fi.IsDir() {
			data, err := os.Open(file)
			if err != nil {
				return err
			}
			if _, err := io.Copy(tw, data); err != nil {
				return err
			}
		}
		return nil
	})

	return tw, nil
}

// FindFirstLayer returns the first layer of a tar archive of a docker image export
func FindFirstLayer(tar_raw io.Reader) (io.Reader, error) {
	// Create a tee reader to write into a byte buffer, and use the reader to create a tar.Reader
	buf := bytes.NewBuffer(nil)
	tr := io.TeeReader(tar_raw, buf)
	tar_reader := tar.NewReader(tr)

	// Capture the first layer name
	layerName := ""
	for {
		header, err := tar_reader.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return nil, fmt.Errorf("failed to read tar: %w", err)
		}

		if header.Name == "manifest.json" {
			manifest, err := io.ReadAll(tar_reader)
			if err != nil {
				return nil, fmt.Errorf("failed to read manifest: %w", err)
			}

			// Parse the manifest json(note this is not the oci manifest)
			var manifestJson []map[string]any
			if err := json.Unmarshal(manifest, &manifestJson); err != nil {
				return nil, fmt.Errorf("failed to unmarshal manifest: %w", err)
			}
			layerName = manifestJson[0]["Layers"].([]any)[0].(string)
			break
		}
	}

	// Check if the layer name is in the tar
	if layerName == "" {
		return nil, fmt.Errorf("failed to find layer")
	}

	// Find the layer in the tar, using the buffer
	buf_reader := bufio.NewReader(buf)
	tar_reader = tar.NewReader(buf_reader)
	for {
		header, err := tar_reader.Next()
		if err == io.EOF {
			break
		}
		if header.Name == layerName {
			return io.LimitReader(buf_reader, header.Size), nil
		}
	}

	return nil, fmt.Errorf("failed to find layer")
}

func WriteTarIntoTar(tw *tar.Writer, tr *tar.Reader, targetDirectory string) error {
	for {
		header, err := tr.Next()
		if err == io.EOF {
			fmt.Println("EOF")
			break
		}
		if err != nil {
			fmt.Println("Error reading tar")
			return err
		}
		fmt.Printf("File: %s\n", header.Name)

		data, err := io.ReadAll(tr)
		if err != nil {
			fmt.Println("Error reading file")
			return err
		}

		if err := WriteFileToTar(tw, filepath.Join(targetDirectory, header.Name), data); err != nil {
			fmt.Println("Error writing file")
			return err
		}
	}
	return nil
}

// WriteFileToTar writes a file to a tar archive
func WriteFileToTar(tw *tar.Writer, file string, data []byte) error {
	fmt.Printf("Writing file: %s\n", file)
	header := &tar.Header{
		Name: file,
		Size: int64(len(data)),
	}

	if err := tw.WriteHeader(header); err != nil {
		return err
	}

	if _, err := tw.Write(data); err != nil {
		return err
	}
	return nil
}

func BuildImage(client *client.Client, buildContext io.Reader, dockerfile string, target string, args map[string]*string) (string, error) {
	ctx := context.Background()
	response, err := client.ImageBuild(ctx, buildContext, types.ImageBuildOptions{
		Dockerfile: dockerfile,
		Target:     target,
		Squash:     true,
		BuildArgs:  args,
	})
	if err != nil {
		return "", err
	}

	// Read the response body one line at a time
	imageID := ""
	scanner := bufio.NewScanner(response.Body)
	for scanner.Scan() {
		line := scanner.Text()

		// Marshal the line into a json object
		var message map[string]any
		if err := json.Unmarshal([]byte(line), &message); err != nil {
			return "", err
		}

		// If the message is a stream, then print it
		if message["stream"] != nil {
			fmt.Printf("Docker Build: %s", message["stream"])
		}

		// If the message is aux, return the ID
		if message["aux"] != nil {
			imageID = message["aux"].(map[string]any)["ID"].(string)
		}
	}
	response.Body.Close()

	if imageID == "" {
		return "", fmt.Errorf("failed to build image")
	}

	return imageID, nil
}

func GetImageTar(client *client.Client, imageID string) (io.Reader, error) {
	ctx := context.Background()
	response, err := client.ImageSave(ctx, []string{imageID})
	if err != nil {
		return nil, err
	}

	return response, nil
}

// This file contains utility functions for Docker operations, specifically building images, pulling images and pushing images.

// apiClient, err := client.NewClientWithOpts(client.FromEnv)
// if err != nil {
// 	panic(err)
// }
// defer apiClient.Close()

// ctx := context.Background()
// info, err := apiClient.Info(ctx)
// serialized_info, _ := json.MarshalIndent(info, "", "  ")
// fmt.Printf("%s\n", string(serialized_info))

// containers, err := apiClient.ContainerList(context.Background(), container.ListOptions{All: true})
// if err != nil {
// 	panic(err)
// }

// for _, ctr := range containers {
// 	fmt.Printf("%s %s (status: %s)\n", ctr.ID, ctr.Image, ctr.Status)
// }
// return
