# ================================================
#  Linux Configuration                          #
# ================================================

-include ../../tools/shared.mk

LINUX_VERSION      					?= 6.13.7
LINUX_ARCHIVE 						?= https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-$(LINUX_VERSION).tar.gz

LINUX_DIR    						?= $(SRC_DIR)/linux
LINUX_PATCH_DIR    					?= $(LINUX_DIR)/patches
LINUX_VENDOR_DIR   					?= $(VENDOR_DIR)/linux

LINUX_DOCKER_RUN	  				?= $(DOCKER) run -it --rm --privileged -v $(DIST_DIR)/wasm_oss:/tftpboot -p 8080:8080
LINUX_DOCKER_BUILD 	   				?= $(DOCKER) build --net host
# QEMU_SHARED_OPTS 					?= -m 1g -nographic
# QEMU_DOCKER_OPTS 					?= $(QEMU_SHARED_OPTS) -netdev user,id=net0,tftp=/tftpboot,hostfwd=tcp::8080-:8080
# QEMU_HOST_OPTS 						?= $(QEMU_SHARED_OPTS) -netdev user,id=net0,tftp=$(DIST_DIR)/wasm_oss,hostfwd=tcp::8080-:8080

LINUX_TAG_BASE         				?= $(PROJECT_NAME)/linux
LINUX_IMAGE_TAG      				?= $(LINUX_TAG_BASE):${VERSION}
# U_BOOT_ARM_IMAGE_TAG      			?= $(U_BOOT_TAG_BASE):arm-${VERSION}
# U_BOOT_X86_64_IMAGE_TAG 			?= $(U_BOOT_TAG_BASE):x86_64-${VERSION}
LINUX_AARCH64_IMAGE_TAG      		?= $(LINUX_TAG_BASE):aarch64-${VERSION}
LINUX_COMPILE_COMMANDS_IMAGE_TAG 	?= $(LINUX_TAG_BASE)-compile-commands:dev

# ================================================
#  Linux Lifecycle                              #
# ================================================

.PHONY: linux-vendor linux-patches linux-ide

linux-vendor: vendor/linux ## Vendor linux source code and apply patches
vendor/linux:
	@printf "$(COLOR_BLUE)🚀 Vendoring Linux $(COLOR_BOLD)$(LINUX_VERSION)$(COLOR_RESET)$(COLOR_BLUE)...$(COLOR_RESET)\n"
	@rm -rf $(LINUX_VENDOR_DIR)
	@mkdir -p $(LINUX_VENDOR_DIR)
	@printf "$(COLOR_CYAN)📥 Downloading Linux $(COLOR_BOLD)$(LINUX_VERSION)$(COLOR_RESET)$(COLOR_CYAN) to '$(shell readlink -f ${LINUX_VENDOR_DIR})'...$(COLOR_RESET)\n"
	@curl -L "$(LINUX_ARCHIVE)" | tar -x --strip-components=1 -C $(LINUX_VENDOR_DIR)
	@printf "$(COLOR_CYAN)🔨 Initializing git repository...$(COLOR_RESET)\n"
	@cd $(LINUX_VENDOR_DIR) && \
	git init && \
	git add . && \
	git commit -m "Initial commit" && \
	git tag root && \
	printf "$(COLOR_CYAN)🔧 Applying patches from $(COLOR_BOLD)$(LINUX_PATCH_DIR)$(COLOR_RESET)$(COLOR_CYAN)...$(COLOR_RESET)\n" && \
	git am $(realpath $(LINUX_PATCH_DIR))/*.patch && \
	printf "$(COLOR_GREEN)✅ Linux vendoring complete!$(COLOR_RESET)\n"

linux-patches: ## Generate patches from linux vendor directory
	@printf "$(COLOR_BLUE)🚀 Generating patches from Linux vendor directory...$(COLOR_RESET)\n"
	@rm -rf $(LINUX_PATCH_DIR)/*.patch
	@mkdir -p $(LINUX_PATCH_DIR)
	@printf "$(COLOR_CYAN)🔧 Extracting patches to $(COLOR_BOLD)$(LINUX_PATCH_DIR)$(COLOR_RESET)$(COLOR_CYAN)...$(COLOR_RESET)\n"
	@git -C $(LINUX_VENDOR_DIR) format-patch --output-directory $(realpath $(LINUX_PATCH_DIR)) root..HEAD
	@printf "$(COLOR_GREEN)✅ Patch generation complete!$(COLOR_RESET)\n"

linux-amend: ## Amend HEAD commit in linux vendor directory
	@printf "$(COLOR_BLUE)🚀 Amending Linux vendor directory...$(COLOR_RESET)\n"
	@cd $(LINUX_VENDOR_DIR) && \
	git add . && \
	git commit --amend --no-edit && \
	printf "$(COLOR_GREEN)✅ Linux vendor directory amended!$(COLOR_RESET)\n"


linux-ide: linux-patches linux-vendor ## Make linux compile-commands.json for IDE
	@printf "$(COLOR_BLUE)🚀 Generating compile-commands.json for Linux...$(COLOR_RESET)\n"
	@$(DOCKER) build -f $(LINUX_DIR)/Dockerfile --target compile-commands -t $(LINUX_COMPILE_COMMANDS_IMAGE_TAG) \
		--build-arg LINUX_ARCHIVE="$(LINUX_ARCHIVE)" \
		--build-arg ARCH=arm64 \
		--build-arg CROSS_COMPILE=aarch64-linux-gnu- \
		--build-arg DEFCONFIG=defconfig \
		$(LINUX_DIR) && \
	$(DOCKER) run -it --rm --privileged $(LINUX_COMPILE_COMMANDS_IMAGE_TAG) \
		cat /linux/compile_commands.json | \
		sed -e 's#-nostdinc##g' \
			-e 's#-flto=4##g' \
			-e 's#-mabi=lp64##g' \
			-e 's#-isystem /usr/lib/gcc/[^ ]*/include##g' \
            -e 's#/linux#$(shell realpath $(LINUX_VENDOR_DIR))#g' \
			> $(LINUX_VENDOR_DIR)/compile_commands.json

# ================================================
#  Linux AARCH64                                #
# ================================================

.PHONY: linux-aarch64-build linux-aarch64-dist

linux-aarch64-build:
	$(LINUX_DOCKER_BUILD) --target dist -t $(LINUX_AARCH64_IMAGE_TAG) \
		--build-arg LINUX_ARCHIVE="$(LINUX_ARCHIVE)" \
		--build-arg ARCH=arm64 \
		--build-arg CROSS_COMPILE=aarch64-linux-gnu- \
		--build-arg DEFCONFIG=neoboot_defconfig \
		$(LINUX_DIR)

linux-aarch64-dist: linux-aarch64-build $(DIST_DIR)
	@mkdir -p $(DIST_DIR)/linux/aarch64
	@$(DOCKER) run -it --rm --privileged -v $(DIST_DIR):/dist $(LINUX_AARCH64_IMAGE_TAG) \
		cp -r /linux/. /dist/linux/aarch64
