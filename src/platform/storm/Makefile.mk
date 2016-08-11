CHIP=sam4l

SLOAD=sload
SDB=$(TOCK_BUILD_DIR)/kernel.sdb
SDB_MAINTAINER=$(shell whoami)
SDB_VERSION=$(shell git show-ref -s HEAD)
SDB_NAME=storm.rs
SDB_DESCRIPTION="An OS for the storm"

PLATFORM_DEPS := $(BUILD_PLATFORM_DIR)/libcore.rlib $(BUILD_PLATFORM_DIR)/libsupport.rlib
PLATFORM_DEPS += $(BUILD_PLATFORM_DIR)/libhil.rlib $(BUILD_PLATFORM_DIR)/libdrivers.rlib
PLATFORM_DEPS += $(BUILD_PLATFORM_DIR)/libmain.rlib

all: $(BUILD_PLATFORM_DIR)/kernel.elf

$(BUILD_PLATFORM_DIR)/libstorm.o: $(call rwildcard,$(SRC_DIR)platform/storm/src,*.rs) $(BUILD_PLATFORM_DIR)/libsam4l.rlib $(PLATFORM_DEPS) | $(BUILD_PLATFORM_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -C lto --emit=obj -o $@ $(SRC_DIR)platform/storm/src/main.rs
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > $(BUILD_PLATFORM_DIR)/kernel.lst

$(BUILD_PLATFORM_DIR)/kernel.elf: $(BUILD_PLATFORM_DIR)/libstorm.o | $(BUILD_PLATFORM_DIR)
	@tput bold ; echo "Linking $@" ; tput sgr0
	@$(CC) $(CFLAGS) -Wl,-gc-sections $^ $(LDFLAGS) -Wl,-Map=$(BUILD_PLATFORM_DIR)/kernel.Map -o $@
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > $(BUILD_PLATFORM_DIR)/kernel_post-link.lst
	@$(SIZE) $@

$(BUILD_PLATFORM_DIR)/kernel.sdb: $(BUILD_PLATFORM_DIR)/kernel.elf
	@tput bold ; echo "Packing SDB..." ; tput sgr0
	@$(SLOAD) pack -m "$(SDB_MAINTAINER)" -v "$(SDB_VERSION)" -n "$(SDB_NAME)" -d $(SDB_DESCRIPTION) -o $@ $<

.PHONY: program
program: $(BUILD_PLATFORM_DIR)/kernel.sdb
	$(SLOAD) flash $<

