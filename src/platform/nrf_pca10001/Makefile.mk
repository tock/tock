$(BUILD_DIR)/libplatform.rlib: $(call rwildcard,$(SRC_DIR)platform/$(PLATFORM),*.rs) $(BUILD_DIR)/libcore.rlib $(BUILD_DIR)/libhil.rlib $(BUILD_DIR)/lib$(CHIP).rlib $(BUILD_DIR)/libdrivers.rlib
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_DIR) $(SRC_DIR)platform/$(PLATFORM)/lib.rs

$(BUILD_DIR)/main.elf: $(BUILD_DIR)/crt1.o $(BUILD_DIR)/arch.o $(BUILD_DIR)/main.o $(APP_BINS)
	@echo "Linking $@"
	@$(CC) $(LDFLAGS) -T$(LOADER) $^ -o $@ -ffreestanding -nostdlib -lc -lgcc
	@$(TOOLCHAIN)size $@

all: $(BUILD_DIR)/main.elf

.PHONY: rebuild-apps
rebuild-apps: $(BUILD_DIR)/crt1.o $(BUILD_DIR)/arch.o $(BUILD_DIR)/main.o $(APP_BINS)
	@echo "Relinking with APPS=\"$(APPS)\""
	@$(CC) $(LDFLAGS) -T$(LOADER) $^ -o $(BUILD_DIR)/main.elf -ffreestanding -nostdlib -lc -lgcc

# FIXME: implement "program" target using J-Link
#.PHONY: program
#program: $(BUILD_DIR)/main.hex
#	...
