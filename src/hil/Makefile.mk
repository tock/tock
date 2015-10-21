$(BUILD_DIR)/libhil.rlib: $(call rwildcard,$(SRC_DIR)hil/,*.rs) $(BUILD_DIR)/libcore.rlib $(BUILD_DIR)/libprocess.rlib
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_DIR) $(SRC_DIR)hil/lib.rs
