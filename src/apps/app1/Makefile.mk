#APPS += $(BUILD_APP_DIR)/libapp1.rlib

$(BUILD_APP_DIR)/app1.o: $(call rwildcard,$(SRC_DIR)apps/app1/,*.rs) $(BUILD_DIR)/libcore.rlib $(BUILD_DIR)/libsupport.rlib | $(BUILD_APP_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -C llvm-args="-llc-relocation-model=pic" -C lto --emit obj -o $@ $(SRC_DIR)apps/app1/main.rs

$(BUILD_APP_DIR)/app1.elf: $(BUILD_APP_DIR)/app1.o $(BUILD_DIR)/arch.o
	@echo "Building $@"
	$(CC) $(LDFLAGS) -fpic -mpic-register=r6 $^ -o $@ -ffreestanding -nostdlib -lc -lgcc
