package build

import (
	"regexp"
	"strings"
)

type Context struct {
	Vars    map[string]string
	Version string
	Dir     string
}

func NewContext(dir string) *Context {
	return &Context{Vars: make(map[string]string), Dir: dir}
}

func (c *Context) Substitute(s string) string {
	re := regexp.MustCompile(`\$\{?(\w+)\}?`)
	return re.ReplaceAllStringFunc(s, func(m string) string {
		varName := strings.TrimPrefix(m, "${")
		varName = strings.TrimSuffix(varName, "}")
		return c.Vars[varName]
	})
}
