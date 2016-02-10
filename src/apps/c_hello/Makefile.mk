$(BUILD_APP_DIR)/c_hello.elf: $(call rwildcard,$(SRC_DIR)apps/c_hello/,*.c) $(BUILD_APP_DIR)/firestorm.o $(BUILD_APP_DIR)/tock.o $(BUILD_APP_DIR)/crt1.o $(BUILD_APP_DIR)/sys.o $(BUILD_APP_DIR)/arch.o $(APP_LIBC)
	@echo "Building $@"
	@$(CC) $(LDFLAGS) $(CFLAGS_APPS) -g -Os -T $(SRC_DIR)apps/c_hello/loader.ld -o $@ -ffreestanding -nostdlib $^

$(BUILD_APP_DIR)/c_hello.bin: $(BUILD_APP_DIR)/c_hello.elf
	@echo "Extracting binary $@"
	@$(OBJCOPY) --gap-fill 0xff -O binary $< $@ 

$(BUILD_APP_DIR)/c_hello.bin.o: $(BUILD_APP_DIR)/c_hello.bin
	@echo "Linking $@"
	@$(LD) -r -b binary -o $@ $<
	@$(OBJCOPY) --rename-section .data=.app.2 $@

