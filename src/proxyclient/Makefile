# ================================================
#  WASM_OSS Configuration                        #
# ================================================

-include ../../tools/shared.mk

PROXYCLIENT_DIR	?= $(SRC_DIR)/proxyclient

# ================================================
#  PROXYCLIENT Targets	                             #
# ================================================

.PHONY: proxyclient_run

proxyclient_run: $(VENV_DIR)
	@$(VENV_DIR)/bin/python -m proxyclient

