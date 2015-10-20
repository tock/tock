APPS += $(BUILD_DIR)/apps/libapp2.rlib

$(BUILD_DIR)/apps/app2.o: $(call rwildcard,src/apps/app2/,*.rs) $(BUILD_DIR)/libcore.rlib $(BUILD_DIR)/libsupport.rlib
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -C lto --emit obj -o $@ src/apps/app2/main.rs

$(BUILD_DIR)/apps/app2.elf: $(call rwildcard,src/apps/app2/,*.c) $(BUILD_DIR)/arch.o src/apps/app2/libc.a
	@echo "Building $@"
	$(CC) $(LDFLAGS) $(CFLAGS_APPS) -g -Os -T src/apps/app2/loader.ld -o $@ -ffreestanding -nostdlib --specs=nosys.specs --specs=nano.specs $^ src/apps/app2/libc.a

$(BUILD_DIR)/apps/app2.bin: $(BUILD_DIR)/apps/app2.elf
	@echo "Extracting binary $@"
	$(OBJCOPY) --gap-fill 0xff -O binary $< $@ 

$(BUILD_DIR)/apps/app2.bin.o: $(BUILD_DIR)/apps/app2.bin
	@echo "Linking $@"
	$(LD) -r -b binary -o $@ $<
	$(OBJCOPY) --rename-section .data=.app.2 $@
