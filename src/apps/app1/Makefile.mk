APPS += $(BUILD_DIR)/apps/libapp1.rlib

$(BUILD_DIR)/apps/app1.o: $(call rwildcard,src/apps/app1/,*.rs) $(BUILD_DIR)/libcore.rlib $(BUILD_DIR)/libsupport.rlib
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -C llvm-args="-llc-relocation-model=pic" -C lto --emit obj -o $@ src/apps/app1/main.rs

$(BUILD_DIR)/apps/app1.elf: $(BUILD_DIR)/apps/app1.o $(BUILD_DIR)/arch.o
	@echo "Building $@"
	$(CC) $(LDFLAGS) -fpic -mpic-register=r6 $^ -o $@ -ffreestanding -nostdlib -lc -lgcc
