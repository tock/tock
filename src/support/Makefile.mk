$(BUILD_DIR)/libsupport.rlib: $(call rwildcard,$(SRC_DIR)support/,*.rs) $(BUILD_DIR)/libcore.rlib
