$(BUILD_DIR)/libcommon.rlib: $(call rwildcard,src/common/,*.rs) $(BUILD_DIR)/libcore.rlib $(BUILD_DIR)/libsupport.rlib
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_DIR) src/common/lib.rs
