$(BUILD_PLATFORM_APP_DIR)/spi_buf.elf: $(call rwildcard,$(SRC_DIR)apps/spi_buf/,*.c) $(BUILD_PLATFORM_APP_DIR)/firestorm.o $(BUILD_PLATFORM_APP_DIR)/tock.o $(BUILD_PLATFORM_APP_DIR)/crt1.o $(BUILD_PLATFORM_APP_DIR)/sys.o $(BUILD_PLATFORM_APP_DIR)/arch.o $(APP_LIBC) | $(BUILD_PLATFORM_APP_DIR)
	@echo "Building $@"
	@$(CC) $(CFLAGS_BASE) $(CFLAGS_APPS) -g -Os -T $(SRC_DIR)apps/spi_buf/loader.ld -o $@ -ffreestanding -nostdlib -Wl,-Map=$(BUILD_PLATFORM_APP_DIR)/app.Map $^
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > $(BUILD_PLATFORM_APP_DIR)/app.lst

$(BUILD_PLATFORM_APP_DIR)/spi_buf.bin: $(BUILD_PLATFORM_APP_DIR)/spi_buf.elf
	@echo "Extracting binary $@"
	@$(OBJCOPY) --gap-fill 0xff -O binary $< $@ 

$(BUILD_PLATFORM_APP_DIR)/spi_buf.bin.o: $(BUILD_PLATFORM_APP_DIR)/spi_buf.bin
	@echo "Linking $@"
	@$(LD) -r -b binary -o $@ $<
	@$(OBJCOPY) --rename-section .data=.app.2 $@

