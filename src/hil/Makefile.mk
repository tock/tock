$(BUILD_PLATFORM_DIR)/libhil.rlib: $(call rwildcard,$(SRC_DIR)hil/,*.rs) $(BUILD_PLATFORM_DIR)/libcore.rlib $(BUILD_PLATFORM_DIR)/libprocess.rlib $(BUILD_PLATFORM_DIR)/libcommon.rlib | $(BUILD_PLATFORM_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_PLATFORM_DIR) $(SRC_DIR)hil/lib.rs
