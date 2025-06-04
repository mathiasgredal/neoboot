package steps

import (
	"github.com/mathiasgredal/neoboot/src/cli/build/context"

	"fmt"
)

func HandleVersion(ctx *context.Context, args any) error {
	versionStr, ok := args.(string)
	if !ok {
		return fmt.Errorf("VERSION requires a string")
	}

	versionStr = ctx.Substitute(versionStr)

	// Set the version in the context
	ctx.Version = versionStr
	return nil
}
