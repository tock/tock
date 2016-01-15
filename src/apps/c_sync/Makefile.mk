$(BUILD_APP_DIR)/c_sync.elf: $(call rwildcard,$(SRC_DIR)apps/c_sync/,*.c) $(BUILD_DIR)/arch.o $(BUILD_APP_DIR)/firestorm.o $(BUILD_APP_DIR)/tock.o $(BUILD_APP_DIR)/crt1.o $(BUILD_APP_DIR)/sys.o $(APP_LIBC)
	@echo "Building $@"
	@$(CC) $(LDFLAGS) $(CFLAGS_APPS) -g -Os -T $(SRC_DIR)apps/c_sync/loader.ld -o $@ -ffreestanding -nostdlib $^

$(BUILD_APP_DIR)/c_sync.bin: $(BUILD_APP_DIR)/c_sync.elf
	@echo "Extracting binary $@"
	@$(OBJCOPY) --gap-fill 0xff -O binary $< $@ 

$(BUILD_APP_DIR)/c_sync.bin.o: $(BUILD_APP_DIR)/c_sync.bin
	@echo "Linking $@"
	@$(LD) -r -b binary -o $@ $<
	@$(OBJCOPY) --rename-section .data=.app.2 $@

