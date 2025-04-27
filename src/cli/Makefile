# ================================================
#  CLI Configuration                             #
# ================================================

-include ../../tools/shared.mk

CLI_DIR					?= $(SRC_DIR)/cli
CLI_BUILD_DIR  			?= $(BUILD_DIR)/cli
CLI_BUILD_EXECUTABLE	?= $(CLI_BUILD_DIR)/neoboot

# ================================================
#  CLI Targets	                                 #
# ================================================

.PHONY: 

cli_build:
	@go build -mod=mod -o $(CLI_BUILD_EXECUTABLE) $(CLI_DIR)/main.go

cli_run: cli_build
	@$(CLI_BUILD_EXECUTABLE)