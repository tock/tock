$(BUILD_PLATFORM_DIR)/libcommon.rlib: $(call rwildcard,$(SRC_DIR)common/src/,*.rs) $(BUILD_PLATFORM_DIR)/libcore.rlib | $(BUILD_PLATFORM_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_PLATFORM_DIR) $(SRC_DIR)common/src/lib.rs

