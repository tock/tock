APPS += app2

$(BUILD_DIR)/apps/app2.elf: $(call rwildcard,$(SRC_DIR)apps/app2/,*.c) $(BUILD_DIR)/arch.o $(APP_LIBC)
	@mkdir -p $(BUILD_DIR)/apps
	@echo "Building $@"
	@$(CC) $(LDFLAGS) $(CFLAGS_APPS) -g -Os -T $(SRC_DIR)apps/app2/loader.ld -o $@ -ffreestanding -nostdlib --specs=nosys.specs --specs=nano.specs $^

$(BUILD_DIR)/apps/app2.bin: $(BUILD_DIR)/apps/app2.elf
	@echo "Extracting binary $@"
	@$(OBJCOPY) --gap-fill 0xff -O binary $< $@ 

$(BUILD_DIR)/apps/app2.bin.o: $(BUILD_DIR)/apps/app2.bin
	@echo "Linking $@"
	@$(LD) -r -b binary -o $@ $<
	@$(OBJCOPY) --rename-section .data=.app.2 $@

