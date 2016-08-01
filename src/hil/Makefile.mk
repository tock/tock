$(BUILD_PLATFORM_DIR)/libhil.rlib: $(call rwildcard,$(SRC_DIR)hil/src/,*.rs) $(BUILD_PLATFORM_DIR)/libcore.rlib $(BUILD_PLATFORM_DIR)/libmain.rlib $(BUILD_PLATFORM_DIR)/libcommon.rlib | $(BUILD_PLATFORM_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_PLATFORM_DIR) $(SRC_DIR)hil/src/lib.rs
