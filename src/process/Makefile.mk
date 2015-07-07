$(BUILD_DIR)/libprocess.rlib: $(call rwildcard,src/process/,*.rs) $(BUILD_DIR)/libcore.rlib $(BUILD_DIR)/libcommon.rlib
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_DIR) src/process/lib.rs
