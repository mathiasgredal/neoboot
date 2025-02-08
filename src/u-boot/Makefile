# ================================================
#  U-Boot Configuration                          #
# ================================================

-include ../../tools/shared.mk

U_BOOT_VERSION      				?= v2025.01
U_BOOT_ARCHIVE 						?= https://gitlab.com/u-boot/u-boot/-/archive/$(U_BOOT_VERSION)/u-boot-$(U_BOOT_VERSION).tar.gz

U_BOOT_DIR    						?= $(SRC_DIR)/u-boot
U_BOOT_PATCH_DIR    				?= $(U_BOOT_DIR)/patches
U_BOOT_VENDOR_DIR   				?= $(VENDOR_DIR)/u-boot

DOCKER_RUN	  						?= $(DOCKER) run -it --rm --privileged -v $(DIST_DIR)/wasm_oss:/tftpboot
DOCKER_BUILD 	   					?= $(DOCKER) build --net host
QEMU_OPTS 							?= -m 1g -nographic -netdev user,id=net0,tftp=/tftpboot -device e1000,netdev=net0

U_BOOT_TAG_BASE         			?= $(PROJECT_NAME)/u-boot
U_BOOT_ARM_IMAGE_TAG      			?= $(U_BOOT_TAG_BASE):arm-${VERSION}
U_BOOT_X86_64_IMAGE_TAG 			?= $(U_BOOT_TAG_BASE):x86_64-${VERSION}
U_BOOT_AARCH64_IMAGE_TAG      		?= $(U_BOOT_TAG_BASE):aarch64-${VERSION}
U_BOOT_COMPILE_COMMANDS_IMAGE_TAG 	?= $(U_BOOT_TAG_BASE)-compile-commands:dev

# ================================================
#  U-Boot Lifecycle                              #
# ================================================

.PHONY: u-boot-vendor u-boot-patches u-boot-ide u-boot-redis-up u-boot-redis-down u-boot-amend

u-boot-vendor: vendor/u-boot ## Vendor u-boot source code and apply patches
vendor/u-boot:
	@printf "$(COLOR_BLUE)🚀 Vendoring U-Boot $(COLOR_BOLD)$(U_BOOT_VERSION)$(COLOR_RESET)$(COLOR_BLUE)...$(COLOR_RESET)\n"
	@rm -rf $(U_BOOT_VENDOR_DIR)
	@mkdir -p $(U_BOOT_VENDOR_DIR)
	@printf "$(COLOR_CYAN)📥 Downloading U-Boot $(COLOR_BOLD)$(U_BOOT_VERSION)$(COLOR_RESET)$(COLOR_CYAN) to '$(shell readlink -f ${U_BOOT_VENDOR_DIR})'...$(COLOR_RESET)\n"
	@curl -L "$(U_BOOT_ARCHIVE)" | tar -x --strip-components=1 -C $(U_BOOT_VENDOR_DIR)
	@printf "$(COLOR_CYAN)🔨 Initializing git repository...$(COLOR_RESET)\n"
	@cd $(U_BOOT_VENDOR_DIR) && \
	git init && \
	git add . && \
	git commit -m "Initial commit" && \
	git tag root && \
	printf "$(COLOR_CYAN)🔧 Applying patches from $(COLOR_BOLD)$(U_BOOT_PATCH_DIR)$(COLOR_RESET)$(COLOR_CYAN)...$(COLOR_RESET)\n" && \
	git am $(realpath $(U_BOOT_PATCH_DIR))/*.patch && \
	printf "$(COLOR_GREEN)✅ U-Boot vendoring complete!$(COLOR_RESET)\n"

u-boot-patches: ## Generate patches from u-boot vendor directory
	@printf "$(COLOR_BLUE)🚀 Generating patches from U-Boot vendor directory...$(COLOR_RESET)\n"
	@rm -rf $(U_BOOT_PATCH_DIR)/*.patch
	@mkdir -p $(U_BOOT_PATCH_DIR)
	@printf "$(COLOR_CYAN)🔧 Extracting patches to $(COLOR_BOLD)$(U_BOOT_PATCH_DIR)$(COLOR_RESET)$(COLOR_CYAN)...$(COLOR_RESET)\n"
	@git -C $(U_BOOT_VENDOR_DIR) format-patch --output-directory $(realpath $(U_BOOT_PATCH_DIR)) root..HEAD
	@printf "$(COLOR_GREEN)✅ Patch generation complete!$(COLOR_RESET)\n"

u-boot-ide: u-boot-patches u-boot-vendor ## Make u-boot compile-commands.json for IDE
	@printf "$(COLOR_BLUE)🚀 Generating compile-commands.json for U-Boot...$(COLOR_RESET)\n"
	@$(DOCKER) build -f $(U_BOOT_DIR)/Dockerfile --target compile-commands -t $(U_BOOT_COMPILE_COMMANDS_TAG) \
		--build-arg U_BOOT_ARCHIVE="$(U_BOOT_ARCHIVE)" \
		--build-arg DEFCONFIG=qemu_arm64_lwip_defconfig \
		$(U_BOOT_DIR) && \
	$(DOCKER) run -it --rm --privileged $(U_BOOT_COMPILE_COMMANDS_TAG) \
		cat /u-boot/compile_commands.json | \
		sed -e 's#-nostdinc##g' \
			-e 's#-flto=4##g' \
			-e 's#-isystem /usr/lib/gcc/[^ ]*/include##g' \
            -e 's#/u-boot#$(shell realpath $(U_BOOT_VENDOR_DIR))#g' \
			> $(U_BOOT_VENDOR_DIR)/compile_commands.json

u-boot-redis-up: ## Start redis cache for u-boot
	$(DOCKER) run -d --rm --name u-boot-redis-cache -p 6379:6379 redis:alpine

u-boot-redis-down: ## Stop redis cache for u-boot
	$(DOCKER) stop u-boot-redis-cache

u-boot-amend: ## Amend HEAD commit in u-boot vendor directory
	@printf "$(COLOR_BLUE)🚀 Amending U-Boot vendor directory...$(COLOR_RESET)\n"
	@cd $(U_BOOT_VENDOR_DIR) && \
	git add . && \
	git commit --amend --no-edit && \
	printf "$(COLOR_GREEN)✅ U-Boot vendor directory amended!$(COLOR_RESET)\n"

# ================================================
#  U-Boot AARCH64                                #
# ================================================

.PHONY: u-boot-aarch64-build u-boot-aarch64-dist u-boot-aarch64-run

u-boot-aarch64-build:
	$(DOCKER_BUILD) --target emulator -t $(U_BOOT_AARCH64_IMAGE_TAG) \
		--build-arg U_BOOT_ARCHIVE="$(U_BOOT_ARCHIVE)" \
		--build-arg CROSS_COMPILE=aarch64-linux-gnu- \
		--build-arg DEFCONFIG=qemu_arm64_wasm_defconfig \
		$(U_BOOT_DIR)

u-boot-aarch64-dist: u-boot-aarch64-build
	@mkdir -p $(DIST_DIR)/u-boot/aarch64
	@$(DOCKER) run -it --rm --privileged -v $(realpath .)/dist:/dist $(U_BOOT_AARCH64_IMAGE_TAG) \
		cp -r /u-boot/. $(DIST_DIR)/u-boot/aarch64

u-boot-aarch64-run: u-boot-aarch64-build
	$(DOCKER_RUN) $(U_BOOT_AARCH64_IMAGE_TAG) \
		qemu-system-aarch64 -machine virt -cpu cortex-a57 ${QEMU_OPTS} -bios /u-boot/u-boot.bin

# ================================================
#  U-Boot ARM                                    #
# ================================================

.PHONY: u-boot-arm-build u-boot-arm-dist u-boot-arm-run

u-boot-arm-build:
	$(DOCKER_BUILD) --target emulator -t $(U_BOOT_ARM_IMAGE_TAG) \
		--build-arg U_BOOT_ARCHIVE="$(U_BOOT_ARCHIVE)" \
		--build-arg CROSS_COMPILE=arm-linux-gnueabihf- \
		--build-arg DEFCONFIG=qemu_arm_defconfig \
		$(U_BOOT_DIR)

u-boot-arm-dist: u-boot-arm-build	
	@mkdir -p $(DIST_DIR)/u-boot/arm
	@$(DOCKER) run -it --rm --privileged -v $(realpath .)/dist:/dist $(U_BOOT_ARM_IMAGE_TAG) \
		cp -r /u-boot/. $(DIST_DIR)/u-boot/arm

u-boot-arm-run: u-boot-arm-build
	$(DOCKER_RUN) $(U_BOOT_ARM_IMAGE_TAG) \
		qemu-system-arm -machine virt ${QEMU_OPTS} -bios /u-boot/u-boot.bin

# ================================================
#  U-Boot x86_64                                 #
# ================================================

.PHONY: u-boot-x86_64-build u-boot-x86_64-dist u-boot-x86_64-run

u-boot-x86_64-build:
	$(DOCKER_BUILD) --target emulator -t $(U_BOOT_X86_64_IMAGE_TAG) \
		--build-arg U_BOOT_ARCHIVE="$(U_BOOT_ARCHIVE)" \
		--build-arg CROSS_COMPILE=x86_64-linux-gnu- \
		--build-arg DEFCONFIG=qemu-x86_wasm_defconfig \
		$(U_BOOT_DIR)

u-boot-x86_64-dist: u-boot-x86_64-build
	@mkdir -p $(DIST_DIR)/u-boot/x86_64
	@$(DOCKER) run -it --rm --privileged -v $(realpath .)/dist:/dist $(U_BOOT_X86_64_IMAGE_TAG) \
		cp -r /u-boot/. $(DIST_DIR)/u-boot/x86_64

u-boot-x86_64-run: u-boot-x86_64-build
	$(DOCKER_RUN) $(U_BOOT_X86_64_IMAGE_TAG) \
		qemu-system-i386 ${QEMU_OPTS} -machine pc -bios /u-boot/u-boot.rom
