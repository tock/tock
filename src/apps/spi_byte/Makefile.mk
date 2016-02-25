$(BUILD_PLATFORM_APP_DIR)/spi_byte.elf: $(call rwildcard,$(SRC_DIR)apps/spi_byte/,*.c) $(BUILD_PLATFORM_DIR)/arch.o $(BUILD_PLATFORM_APP_DIR)/firestorm.o $(BUILD_PLATFORM_APP_DIR)/tock.o $(BUILD_PLATFORM_APP_DIR)/crt1.o $(BUILD_PLATFORM_APP_DIR)/sys.o $(APP_LIBC)
	@mkdir -p $(BUILD_PLATFORM_APP_DIR)
	@echo "Building $@"
	@$(CC) $(LDFLAGS) $(CFLAGS_APPS) -g -Os -T $(SRC_DIR)apps/spi_byte/loader.ld -o $@ -ffreestanding -nostdlib $^

$(BUILD_PLATFORM_APP_DIR)/spi_byte.bin: $(BUILD_PLATFORM_APP_DIR)/spi_byte.elf
	@echo "Extracting binary $@"
	@$(OBJCOPY) --gap-fill 0xff -O binary $< $@

$(BUILD_PLATFORM_APP_DIR)/spi_byte.bin.o: $(BUILD_PLATFORM_APP_DIR)/spi_byte.bin
	@echo "Linking $@"
	@$(LD) -r -b binary -o $@ $<
	@$(OBJCOPY) --rename-section .data=.app.2 $@

