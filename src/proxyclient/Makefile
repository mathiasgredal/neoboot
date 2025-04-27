# ================================================
#  WASM_OSS Configuration                        #
# ================================================

-include ../../tools/shared.mk

PROXYCLIENT_DIR	?= $(SRC_DIR)/proxyclient

# ================================================
#  PROXYCLIENT Targets	                             #
# ================================================

.PHONY: proxyclient_boot proxyclient_chain

proxyclient_boot: $(VENV_DIR)
	@$(VENV_DIR)/bin/python -m proxyclient boot -t aarch64

proxyclient_chain: $(VENV_DIR)
	@$(VENV_DIR)/bin/python -m proxyclient chain