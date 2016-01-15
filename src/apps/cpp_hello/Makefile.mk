$(BUILD_APP_DIR)/cpp_hello.elf: $(call rwildcard,$(SRC_DIR)apps/cpp_hello/,*.cc) $(BUILD_DIR)/arch.o $(BUILD_APP_DIR)/firestorm.o $(BUILD_APP_DIR)/tock.o $(BUILD_APP_DIR)/crt1.o $(BUILD_APP_DIR)/sys.o | $(APP_LIBC)
	@echo "Building $@"
	$(CPP) $(LDFLAGS) $(CFLAGS_APPS) -fno-exceptions -ffunction-sections -fdata-sections -Wl,--gc-sections -mfloat-abi=soft -g -Os -T $(SRC_DIR)apps/cpp_hello/loader.ld -o $@ -nostdlib -nostartfiles -ffreestanding $^ -lstdc++ $(APP_LIBC) -lgcc

$(BUILD_APP_DIR)/cpp_hello.bin: $(BUILD_APP_DIR)/cpp_hello.elf
	@echo "Extracting binary $@"
	@$(OBJCOPY) --gap-fill 0xff -O binary $< $@ 

$(BUILD_APP_DIR)/cpp_hello.bin.o: $(BUILD_APP_DIR)/cpp_hello.bin
	@echo "Linking $@"
	@$(LD) -r -b binary -o $@ $<
	@$(OBJCOPY) --rename-section .data=.app.2 $@

