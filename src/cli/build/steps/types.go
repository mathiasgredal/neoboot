package steps

import (
	"archive/tar"
	"bytes"
	"path/filepath"

	"github.com/docker/docker/client"
	"github.com/mathiasgredal/neoboot/src/cli/utils"
)

type DockerBuild struct {
	Dockerfile       string             `json:"dockerfile"`
	DockerfileInline string             `json:"dockerfile_inline"`
	Target           string             `json:"target"`
	Context          string             `json:"context"`
	Args             map[string]*string `json:"args"`
}

func (d *DockerBuild) BuildImage(client *client.Client, workingDir string, buildContextMiddleware func(*tar.Writer) error) (*tar.Reader, error) {
	// Get the absolute path of the build context
	buildContext, err := filepath.Abs(filepath.Join(workingDir, d.Context))
	if err != nil {
		return nil, err
	}

	// Create a tar archive of the build context
	buf := bytes.NewBuffer(nil)
	tw, err := utils.MakeTar(buildContext, buf)
	if err != nil {
		return nil, err
	}

	// Add the dockerfile inline to the build context
	if d.DockerfileInline != "" {
		if err := utils.WriteFileToTar(tw, d.Dockerfile, []byte(d.DockerfileInline)); err != nil {
			return nil, err
		}
	}

	// Run the build context middleware, to allow for customizing the build context
	if buildContextMiddleware != nil {
		if err := buildContextMiddleware(tw); err != nil {
			return nil, err
		}
	}

	// Close the tar archive
	if err := tw.Close(); err != nil {
		return nil, err
	}

	// Build the image
	imageID, err := utils.BuildImage(client, buf, d.Dockerfile, d.Target, d.Args)
	if err != nil {
		return nil, err
	}

	// Get the image tar
	imageTarRaw, err := utils.GetImageTar(client, imageID)
	if err != nil {
		return nil, err
	}

	// Find the first layer
	layer, err := utils.FindFirstLayer(imageTarRaw)
	if err != nil {
		return nil, err
	}

	// Return the image tar
	return tar.NewReader(layer), nil
}
