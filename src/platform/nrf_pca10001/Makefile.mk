CHIP=nrf51822

PLATFORM_DEPS=$(BUILD_PLATFORM_DIR)/libcore.rlib $(BUILD_PLATFORM_DIR)/libsupport.rlib
PLATFORM_DEPS+=$(BUILD_PLATFORM_DIR)/libhil.rlib $(BUILD_PLATFORM_DIR)/libdrivers.rlib
PLATFORM_DEPS+=$(BUILD_PLATFORM_DIR)/libmain.rlib $(BUILD_PLATFORM_DIR)/libnrf51822.rlib
PLATFORM_DEPS+=$(BUILD_PLATFORM_DIR)/libcortexm0.rlib

all: $(BUILD_PLATFORM_DIR)/kernel.elf $(BUILD_PLATFORM_DIR)/crt1.o

$(BUILD_PLATFORM_DIR)/kernel.elf: $(call rwildcard,$(SRC_DIR)platform/nrf_pca10001/src,*.rs) $(PLATFORM_DEPS) | $(BUILD_PLATFORM_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -C lto -o $@ $(SRC_DIR)platform/nrf_pca10001/src/main.rs -L native=$(BUILD_PLATFORM_DIR)
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > $(BUILD_PLATFORM_DIR)/kernel.lst
	@$(SIZE) $@

