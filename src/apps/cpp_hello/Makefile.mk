$(BUILD_DIR)/apps/cpp_hello.elf: $(call rwildcard,$(SRC_DIR)apps/cpp_hello/,*.cc) $(BUILD_DIR)/arch.o $(BUILD_DIR)/apps/firestorm.o $(BUILD_DIR)/apps/tock.o $(BUILD_DIR)/apps/crt1.o
	@mkdir -p $(BUILD_DIR)/apps
	@echo "Building $@"
	$(CPP) $(LDFLAGS) $(CFLAGS_APPS) -Wl,--gc-sections -mfloat-abi=soft -g -Os -T $(SRC_DIR)apps/cpp_hello/loader.ld -o $@ -ffreestanding -nostdlib -nostartfiles $^ -lstdc++ $(APP_LIBC) -lgcc -lm

$(BUILD_DIR)/apps/cpp_hello.bin: $(BUILD_DIR)/apps/cpp_hello.elf
	@echo "Extracting binary $@"
	@$(OBJCOPY) --gap-fill 0xff -O binary $< $@ 

$(BUILD_DIR)/apps/cpp_hello.bin.o: $(BUILD_DIR)/apps/cpp_hello.bin
	@echo "Linking $@"
	@$(LD) -r -b binary -o $@ $<
	@$(OBJCOPY) --rename-section .data=.app.2 $@

