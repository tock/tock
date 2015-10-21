$(BUILD_DIR)/libprocess.rlib: $(call rwildcard,$(SRC_DIR)process/,*.rs) $(BUILD_DIR)/libcore.rlib $(BUILD_DIR)/libcommon.rlib
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_DIR) $(SRC_DIR)process/lib.rs
