package parser

import (
	"fmt"
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
	var steps any
	if err := yaml.Unmarshal(data, &steps); err != nil {
		return nil, err
	}

	// Convert all keys to strings
	convertedSteps, err := convertToStringKeysRecursive(steps, "")
	if err != nil {
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
	if err := schema.Validate(convertedSteps); err != nil {
		return nil, err
	}

	var parsed []Step
	for _, step := range convertedSteps.([]interface{}) {
		for cmd, args := range step.(map[string]interface{}) {
			parsed = append(parsed, Step{Command: cmd, Args: args})
		}
	}
	return parsed, nil
}

func convertToStringKeysRecursive(value interface{}, keyPrefix string) (interface{}, error) {
	if mapping, ok := value.(map[interface{}]interface{}); ok {
		dict := make(map[string]interface{})
		for key, entry := range mapping {
			str, ok := key.(string)
			if !ok {
				return nil, formatInvalidKeyError(keyPrefix, key)
			}
			var newKeyPrefix string
			if keyPrefix == "" {
				newKeyPrefix = str
			} else {
				newKeyPrefix = fmt.Sprintf("%s.%s", keyPrefix, str)
			}
			convertedEntry, err := convertToStringKeysRecursive(entry, newKeyPrefix)
			if err != nil {
				return nil, err
			}
			dict[str] = convertedEntry
		}
		return dict, nil
	}
	if list, ok := value.([]interface{}); ok {
		var convertedList []interface{}
		for index, entry := range list {
			newKeyPrefix := fmt.Sprintf("%s[%d]", keyPrefix, index)
			convertedEntry, err := convertToStringKeysRecursive(entry, newKeyPrefix)
			if err != nil {
				return nil, err
			}
			convertedList = append(convertedList, convertedEntry)
		}
		return convertedList, nil
	}
	return value, nil
}

func formatInvalidKeyError(keyPrefix string, key interface{}) error {
	var location string
	if keyPrefix == "" {
		location = "at top level"
	} else {
		location = fmt.Sprintf("in %s", keyPrefix)
	}
	return fmt.Errorf("Non-string key %s: %#v", location, key)
}
