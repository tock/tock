$(BUILD_DIR)/apps/c_hello.elf: $(call rwildcard,$(SRC_DIR)apps/c_hello/,*.c) $(BUILD_DIR)/arch.o $(BUILD_DIR)/apps/firestorm.o $(BUILD_DIR)/apps/tock.o $(APP_LIBC)
	@mkdir -p $(BUILD_DIR)/apps
	@echo "Building $@"
	@$(CC) $(LDFLAGS) $(CFLAGS_APPS) -g -Os -T $(SRC_DIR)apps/c_hello/loader.ld -o $@ -ffreestanding -nostdlib $^

$(BUILD_DIR)/apps/c_hello.bin: $(BUILD_DIR)/apps/c_hello.elf
	@echo "Extracting binary $@"
	@$(OBJCOPY) --gap-fill 0xff -O binary $< $@ 

$(BUILD_DIR)/apps/c_hello.bin.o: $(BUILD_DIR)/apps/c_hello.bin
	@echo "Linking $@"
	@$(LD) -r -b binary -o $@ $<
	@$(OBJCOPY) --rename-section .data=.app.2 $@

