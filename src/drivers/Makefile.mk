$(BUILD_DIR)/libdrivers.rlib: $(call rwildcard,src/drivers/,*.rs) $(BUILD_DIR)/libcore.rlib $(BUILD_DIR)/libhil.rlib
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -F unsafe-code --out-dir $(BUILD_DIR) src/drivers/lib.rs
