package build

import (
	"os"

	"gopkg.in/yaml.v3"
)

type Step struct {
	Command string
	Args    any
}

func Parse(path string) ([]Step, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	var steps []map[string]any
	if err := yaml.Unmarshal(data, &steps); err != nil {
		return nil, err
	}
	var parsed []Step
	for _, step := range steps {
		for cmd, args := range step {
			parsed = append(parsed, Step{Command: cmd, Args: args})
		}
	}
	return parsed, nil
}
