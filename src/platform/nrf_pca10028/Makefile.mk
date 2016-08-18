CHIP=nrf51

#all: $(BUILD_PLATFORM_DIR)/main.bin | $(BUILD_PLATFORM_DIR)
#
#$(BUILD_PLATFORM_DIR)/libplatform.rlib: $(call rwildcard,$(SRC_DIR)platform/$(TOCK_PLATFORM),*.rs) $(BUILD_PLATFORM_DIR)/libcore.rlib $(BUILD_PLATFORM_DIR)/libhil.rlib $(BUILD_PLATFORM_DIR)/lib$(CHIP).rlib $(BUILD_PLATFORM_DIR)/libdrivers.rlib | $(BUILD_PLATFORM_DIR)
#	@echo "Building $@"
#	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_PLATFORM_DIR) $(SRC_DIR)platform/$(TOCK_PLATFORM)/lib.rs

$(BUILD_PLATFORM_DIR)/libplatform.rlib: $(call rwildcard,$(SRC_DIR)platform/$(TOCK_PLATFORM),*.rs) $(BUILD_PLATFORM_DIR)/libcore.rlib $(BUILD_PLATFORM_DIR)/libhil.rlib $(BUILD_PLATFORM_DIR)/lib$(CHIP).rlib $(BUILD_PLATFORM_DIR)/libdrivers.rlib | $(BUILD_PLATFORM_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_PLATFORM_DIR) $(SRC_DIR)platform/$(TOCK_PLATFORM)/lib.rs

all: $(BUILD_PLATFORM_DIR)/libplatform.rlib $(BUILD_PLATFORM_DIR)/ctx_switch.o $(BUILD_PLATFORM_DIR)/crt1.o $(BUILD_PLATFORM_DIR)/kernel.o


