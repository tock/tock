CHIP=nrf51822

PLATFORM_DEPS=$(BUILD_PLATFORM_DIR)/libcore.rlib $(BUILD_PLATFORM_DIR)/libsupport.rlib
PLATFORM_DEPS+=$(BUILD_PLATFORM_DIR)/libhil.rlib $(BUILD_PLATFORM_DIR)/libdrivers.rlib
PLATFORM_DEPS+=$(BUILD_PLATFORM_DIR)/libmain.rlib

all: $(BUILD_PLATFORM_DIR)/kernel.elf $(BUILD_PLATFORM_DIR)/crt1.o

$(BUILD_PLATFORM_DIR)/kernel.o: $(call rwildcard,$(SRC_DIR)platform/nrf_pca10001,*.rs) $(BUILD_PLATFORM_DIR)/libnrf51822.rlib $(PLATFORM_DEPS) | $(BUILD_PLATFORM_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -C lto --emit obj -o $@ $(SRC_DIR)platform/nrf_pca10001/main.rs
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > $(BUILD_PLATFORM_DIR)/kernel.lst

$(BUILD_PLATFORM_DIR)/kernel.elf: $(BUILD_PLATFORM_DIR)/ctx_switch.o $(BUILD_PLATFORM_DIR)/kernel.o | $(BUILD_PLATFORM_DIR)
	@tput bold ; echo "Linking $@" ; tput sgr0
	@$(CC) $(CFLAGS) -Wl,-gc-sections $^ $(LDFLAGS) -Wl,-Map=$(BUILD_PLATFORM_DIR)/kernel.Map -o $@
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > $(BUILD_PLATFORM_DIR)/kernel_post-link.lst
	@$(SIZE) $@

