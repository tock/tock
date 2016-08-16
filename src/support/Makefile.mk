$(BUILD_PLATFORM_DIR)/libsupport.rlib: $(call rwildcard,$(SRC_DIR)support/src/,*.rs) $(BUILD_PLATFORM_DIR)/libcore.rlib
