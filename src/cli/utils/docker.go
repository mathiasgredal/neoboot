package utils

import (
	"archive/tar"
	"bufio"
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

func GetDockerClient() (*client.Client, error) {
	apiClient, err := client.NewClientWithOpts(client.FromEnv)
	if err != nil {
		return nil, fmt.Errorf("failed to create docker client: %w", err)
	}
	return apiClient, nil
}

func IsDir(path string) bool {
	info, err := os.Stat(path)
	if os.IsNotExist(err) {
		return false
	}
	return info.IsDir()
}

func MakeTar(base string, src string, buf io.Writer) (*tar.Writer, error) {
	tw := tar.NewWriter(buf)

	// Raise error if src is not a directory
	if !IsDir(src) {
		return nil, fmt.Errorf("src is not a directory: %s", src)
	}

	// walk through every file in the folder
	filepath.Walk(src, func(file string, fi os.FileInfo, err error) error {
		fmt.Printf("Walking file: %s\n", file)
		fmt.Printf("Base: %s\n", base)
		// Make the file path relative to the base directory, by removing the base directory from the file path
		relativePath := strings.TrimPrefix(file, base)
		fmt.Printf("Relative path: %s\n", relativePath)

		// generate tar header
		header, err := tar.FileInfoHeader(fi, relativePath)
		if err != nil {
			return err
		}

		// must provide real name
		// (see https://golang.org/src/archive/tar/common.go?#L626)
		header.Name = filepath.ToSlash(relativePath)

		// write header
		if err := tw.WriteHeader(header); err != nil {
			return err
		}
		// if not a dir, write file content
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

func WriteFileToTar(tw *tar.Writer, file string, data []byte) error {
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

func BuildImage(client *client.Client, buildContext io.Reader) (string, error) {
	ctx := context.Background()
	response, err := client.ImageBuild(ctx, buildContext, types.ImageBuildOptions{
		Dockerfile: "Dockerfile",
		Target:     "dist",
		Squash:     true,
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

	return imageID, nil
}

func GetImageTar(client *client.Client, imageID string) (io.Reader, error) {
	ctx := context.Background()
	response, err := client.ImageSave(ctx, []string{imageID})
	if err != nil {
		return nil, err
	}

	// Write the response to a file
	file, err := os.Create("image.tar")
	if err != nil {
		return nil, err
	}
	defer file.Close()
	io.Copy(file, response)

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
