package steps

import (
	"fmt"
	"strings"

	"github.com/mathiasgredal/neoboot/src/cli/build/context"
)

func HandleArg(ctx *context.Context, args any) error {
	argStr, ok := args.(string)
	if !ok {
		return fmt.Errorf("ARG requires a string")
	}
	parts := strings.SplitN(argStr, "=", 2)
	if len(parts) != 2 {
		return fmt.Errorf("invalid ARG format")
	}
	ctx.Vars[parts[0]] = parts[1]
	return nil
}
