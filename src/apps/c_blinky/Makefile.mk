$(BUILD_DIR)/apps/c_blinky.elf: $(call rwildcard,$(SRC_DIR)apps/c_blinky/,*.c) $(BUILD_DIR)/arch.o $(BUILD_DIR)/apps/firestorm.o $(BUILD_DIR)/apps/tock.o $(BUILD_DIR)/apps/crt1.o $(BUILD_DIR)/apps/sys.o $(APP_LIBC)
	@mkdir -p $(BUILD_DIR)/apps
	@echo "Building $@"
	@$(CC) $(LDFLAGS) $(CFLAGS_APPS) -g -Os -T $(SRC_DIR)apps/c_blinky/loader.ld -o $@ -ffreestanding -nostdlib $^

$(BUILD_DIR)/apps/c_blinky.bin: $(BUILD_DIR)/apps/c_blinky.elf
	@echo "Extracting binary $@"
	@$(OBJCOPY) --gap-fill 0xff -O binary $< $@

$(BUILD_DIR)/apps/c_blinky.bin.o: $(BUILD_DIR)/apps/c_blinky.bin
	@echo "Linking $@"
	@$(LD) -r -b binary -o $@ $<
	@$(OBJCOPY) --rename-section .data=.app.2 $@

