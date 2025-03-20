# ================================================ #
#      _   _            ____              _        #
#     | \ | |          |  _ \            | |       #
#     |  \| | ___  ___ | |_) | ___   ___ | |_      #
#     | . ` |/ _ \/ _ \|  _ < / _ \ / _ \| __|     #
#     | |\  |  __/ (_) | |_) | (_) | (_) | |_      #
#     |_| \_|\___|\___/|____/ \___/ \___/ \__|     #
#                                                  #
# ================================================ #                              

include tools/shared.mk

# ================================================
#  D E P E N D E N C I E S                       #
# ================================================

include $(SRC_DIR)/u-boot/Makefile
include $(SRC_DIR)/wasm_oss/Makefile
include $(SRC_DIR)/proxyclient/Makefile

# ================================================
#  T A R G E T S                                 #
# ================================================

.PHONY: u-boot rust

u-boot: u-boot-amend u-boot-patches wasm_oss_dist u-boot-aarch64-run-host

rust: wasm_oss_dist

py: proxyclient_run

# ================================================
#  "Makefiles are the ultimate abstraction layer"
#               - Every Seasoned Developer
# ================================================