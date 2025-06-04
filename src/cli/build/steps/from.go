package steps

import (
	"fmt"

	"github.com/mathiasgredal/neoboot/src/cli/build/context"
)

func HandleFrom(ctx *context.Context, args any) error {
	fromStr, ok := args.(string)
	if !ok {
		return fmt.Errorf("FROM requires an unnamed string argument")
	}

	fromStr = ctx.Substitute(fromStr)

	// Handle special scratch case
	if fromStr == "scratch" {
		return nil
	}

	// TODO: Implement external image handling and tar ball from "file://" and "http://" and "docker://"
	return fmt.Errorf("From with external image not implemented yet")
}
