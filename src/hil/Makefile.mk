$(BUILD_DIR)/libhil.rlib: $(call rwildcard,src/hil/,*.rs) $(BUILD_DIR)/libcore.rlib $(BUILD_DIR)/libprocess.rlib
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_DIR) src/hil/lib.rs
