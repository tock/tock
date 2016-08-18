$(BUILD_PLATFORM_DIR)/libdrivers.rlib: $(call rwildcard,$(SRC_DIR)drivers/src/,*.rs) $(BUILD_PLATFORM_DIR)/libcore.rlib $(BUILD_PLATFORM_DIR)/libhil.rlib $(BUILD_PLATFORM_DIR)/libcommon.rlib $(BUILD_PLATFORM_DIR)/libmain.rlib | $(BUILD_PLATFORM_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -F unsafe-code --out-dir $(BUILD_PLATFORM_DIR) $(SRC_DIR)drivers/src/lib.rs
