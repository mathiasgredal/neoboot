package context

import (
	"regexp"
	"strings"
)

type Context struct {
	Vars    map[string]string
	Version string
	Dir     string
	Tag     string
}

func NewContext(dir string, tag string) *Context {
	return &Context{Vars: make(map[string]string), Dir: dir, Tag: tag}
}

func (c *Context) Substitute(s string) string {
	re := regexp.MustCompile(`\$\{?(\w+)\}?`)
	return re.ReplaceAllStringFunc(s, func(m string) string {
		varName := strings.TrimPrefix(m, "${")
		varName = strings.TrimSuffix(varName, "}")
		return c.Vars[varName]
	})
}
