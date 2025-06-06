package utils

import (
	"os"
	"path/filepath"

	"github.com/BurntSushi/toml"
	"github.com/mathiasgredal/neoboot/src/cli/utils/log"
)

// General holds general settings.
type General struct {
	Telemetry bool `toml:"telemetry"`
	Debug     bool `toml:"debug"`
}

// Server holds server connection settings.
type Server struct {
	Endpoint string `toml:"endpoint"`
	APIKey   string `toml:"api_key"`
}

// Paths holds various path settings.
type Paths struct {
	CacheDir string `toml:"cache_dir"`
	LogDir   string `toml:"log_dir"`
}

// Config is the root configuration structure.
type Config struct {
	General General `toml:"general"`
	Server  Server  `toml:"server"`
	Paths   Paths   `toml:"paths"`
}

func DefaultConfig() *Config {
	return &Config{
		General: General{
			Telemetry: false,
			Debug:     false,
		},
		Server: Server{
			Endpoint: "https://api.neoboot.dev/v1",
			APIKey:   "",
		},
		Paths: Paths{
			CacheDir: "~/.local/share/neoboot/cache",
			LogDir:   "~/.local/share/neoboot/logs",
		},
	}
}

// LoadConfig loads the configuration from the provided path.
// If no path is provided, it checks the NEBOOT_CONFIG environment variable,
// then defaults to ~/.config/neoboot.conf and /etc/neoboot.conf.
func LoadConfig(path string) (*Config, error) {
	// Use provided path or environment variable override.
	if path == "" {
		path = os.Getenv("NEBOOT_CONFIG")
	}
	if path == "" {
		home, err := os.UserHomeDir()
		if err != nil {
			return nil, err
		}
		userPath := filepath.Join(home, ".config", "neoboot.conf")
		if fileExists(userPath) {
			path = userPath
		} else if fileExists("/etc/neoboot.conf") {
			path = "/etc/neoboot.conf"
		} else {
			log.Debug("No configuration file found. Using default settings.")
			return DefaultConfig(), nil
		}
	}

	var cfg Config
	if _, err := toml.DecodeFile(path, &cfg); err != nil {
		return nil, err
	}
	return &cfg, nil
}

func fileExists(filename string) bool {
	info, err := os.Stat(filename)
	if err != nil {
		return false
	}
	return !info.IsDir()
}
