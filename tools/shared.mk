# ================================================
#  Shared Configuration                          #
# ================================================

# Project Metadata
PROJECT_NAME    ?= neoboot
VERSION         ?= 0.0.1

# Configuration
ROOT_DIR 			:= $(realpath $(dir $(abspath $(lastword $(MAKEFILE_LIST))))..)
SRC_DIR         	?= $(ROOT_DIR)/src
BUILD_DIR       	?= $(ROOT_DIR)/build
DIST_DIR 			?= $(ROOT_DIR)/dist
VENDOR_DIR			?= $(ROOT_DIR)/vendor
VENV_DIR			?= $(BUILD_DIR)/venv

# OCI Detection
ifeq (, $(shell which podman))
	DOCKER ?= $(shell which docker)
else
	DOCKER ?= $(shell which podman)
endif

# ANSI Escape Codes
COLOR_RESET = \033[0m
COLOR_BOLD  = \033[1m
COLOR_GREEN = \033[32m
COLOR_BLUE  = \033[34m
COLOR_CYAN  = \033[36m
COLOR_WHITE = \033[37m

# Cargo Configuration
.EXPORT_ALL_VARIABLES:
CARGO_TARGET_DIR=$(BUILD_DIR)/cargo

# ================================================
#  Shared Targets	                             #
# ================================================

$(BUILD_DIR):
	@mkdir -p $(BUILD_DIR)

$(DIST_DIR):
	@mkdir -p $(DIST_DIR)

$(VENV_DIR): $(BUILD_DIR)
	@printf "$(COLOR_BLUE)🚀 Creating venv...$(COLOR_RESET)\n"
	@rm -rf venv && \
	if command -v uv >/dev/null 2>&1; then \
		uv venv $(VENV_DIR) && \
		VIRTUAL_ENV=$(VENV_DIR) uv pip install -e '.[dev]' && \
		printf "$(COLOR_GREEN)✅ venv created with uv$(COLOR_RESET)\n"; \
	else \
		python3 -m venv $(VENV_DIR) && \
		$(VENV_DIR)/bin/pip install -e '.[dev]' && \
		printf "$(COLOR_GREEN)✅ venv created with pip$(COLOR_RESET)\n"; \
	fi && \
	printf "$(COLOR_BLUE)🚀 Building protobufs for Python...$(COLOR_RESET)\n" && \
	rm -rf $(SRC_DIR)/proto_py/* && \
	$(VENV_DIR)/bin/python -m grpc_tools.protoc \
		--proto_path=$(SRC_DIR)/proto \
		--python_out=$(SRC_DIR)/proto_py \
		--pyi_out=$(SRC_DIR)/proto_py \
		$(SRC_DIR)/proto/*.proto && \
	touch $(SRC_DIR)/proto_py/__init__.py && \
	printf "$(COLOR_GREEN)✅ Python protobufs built$(COLOR_RESET)\n"

.PHONY: clean
clean: ## Remove build artifacts
	@printf "$(COLOR_GREEN)🧹 Cleaning...$(COLOR_RESET)\n"
	rm -rf $(BUILD_DIR) $(DIST_DIR)


.PHONY: help
help: ## Show this help message
	@printf "\n$(COLOR_BOLD)$(COLOR_WHITE)Usage: make [target] [VARIABLE=value]$(COLOR_RESET)\n\n"
	@printf "$(COLOR_BOLD)Available targets:$(COLOR_RESET)\n"
	@awk 'BEGIN {FS = ":.*?## "}; /^[a-zA-Z_-]+:.*?## .*$$/ {printf "  $(COLOR_GREEN)%-20s$(COLOR_RESET) %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Default target if none specified
.DEFAULT_GOAL := help