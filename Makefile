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
include $(SRC_DIR)/linux/Makefile
include $(SRC_DIR)/cli/Makefile

# ================================================
#  T A R G E T S                                 #
# ================================================

.PHONY: u-boot rust

u-boot: u-boot-amend u-boot-patches wasm_oss_dist u-boot-aarch64-run-host

linux: linux-amend linux-patches linux-aarch64-dist

rust: wasm_oss_dist

pyb: proxyclient_boot

pyc: wasm_oss_dist proxyclient_chain

# ================================================
#  "Makefiles are the ultimate abstraction layer"
#               - Every Seasoned Developer
# ================================================