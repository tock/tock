$(BUILD_DIR)/libdrivers.rlib: $(call rwildcard,$(SRC_DIR)drivers/,*.rs) $(BUILD_DIR)/libcore.rlib $(BUILD_DIR)/libhil.rlib $(BUILD_DIR)/libcommon.rlib
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_DIR) $(SRC_DIR)drivers/lib.rs
#	@$(RUSTC) $(RUSTC_FLAGS) -F unsafe-code --out-dir $(BUILD_DIR) $(SRC_DIR)drivers/lib.rs
