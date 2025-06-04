package parser

import (
	"os"
	"strings"

	"github.com/mathiasgredal/neoboot/src/cli/utils"
	"github.com/santhosh-tekuri/jsonschema/v5"
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

	// Validate the bootfile
	compiler := jsonschema.NewCompiler()
	if err := compiler.AddResource("schema.json", strings.NewReader(utils.BootfileSpec)); err != nil {
		return nil, err
	}
	schema, err := compiler.Compile("schema.json")
	if err != nil {
		return nil, err
	}
	if err := schema.Validate(steps); err != nil {
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
