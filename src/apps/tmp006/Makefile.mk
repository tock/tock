$(BUILD_APP_DIR)/tmp006.elf: $(call rwildcard,$(SRC_DIR)apps/tmp006/,*.c) $(BUILD_APP_DIR)/firestorm.o $(BUILD_APP_DIR)/tock.o $(BUILD_APP_DIR)/crt1.o $(BUILD_APP_DIR)/sys.o $(BUILD_APP_DIR)/arch.o $(BUILD_APP_DIR)/tmp006.o $(APP_LIBC)
	@echo "Building $@"
	@$(CC) $(LDFLAGS) $(CFLAGS_APPS) -g -Os -T $(SRC_DIR)apps/tmp006/loader.ld -o $@ -ffreestanding -nostdlib -Wl,-Map=$(BUILD_APP_DIR)/app.Map $^
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > $(BUILD_APP_DIR)/app.lst

$(BUILD_APP_DIR)/tmp006.bin: $(BUILD_APP_DIR)/tmp006.elf
	@echo "Extracting binary $@"
	@$(OBJCOPY) --gap-fill 0xff -O binary $< $@ 

$(BUILD_APP_DIR)/tmp006.bin.o: $(BUILD_APP_DIR)/tmp006.bin
	@echo "Linking $@"
	@$(LD) -r -b binary -o $@ $<
	@$(OBJCOPY) --rename-section .data=.app.2 $@

