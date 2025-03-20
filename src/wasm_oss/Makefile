# ================================================
#  WASM_OSS Configuration                        #
# ================================================

-include ../../tools/shared.mk

WASM_OSS_DIR	?= $(SRC_DIR)/wasm_oss

# ================================================
#  WASM_OSS Targets	                             #
# ================================================

.PHONY: wasm_oss_build wasm_oss_dist

wasm_oss_build: $(BUILD_DIR)
	@printf "$(COLOR_BLUE)🚀 Building WASM-OSS...$(COLOR_RESET)\n"
	@cd $(WASM_OSS_DIR) && \
	cargo build --target wasm32-wasip1
	@printf "$(COLOR_GREEN)✅ WASM-OSS build complete!$(COLOR_RESET)\n"

wasm_oss_dist: wasm_oss_build $(DIST_DIR)
	@printf "$(COLOR_BLUE)🚀 Distributing WASM-OSS...$(COLOR_RESET)\n"
	@mkdir -p $(DIST_DIR)/wasm_oss
	@cp $(CARGO_TARGET_DIR)/wasm32-wasip1/debug/wasm_oss.wasm $(DIST_DIR)/wasm_oss/main.wasm
	@printf "$(COLOR_GREEN)✅ WASM-OSS distribution complete!$(COLOR_RESET)\n"