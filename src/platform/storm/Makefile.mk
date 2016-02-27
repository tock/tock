CHIP=sam4l

SLOAD=sload
SDB=$(BUILD_PLATFORM_DIR)/kernel.sdb
SDB_MAINTAINER=$(shell whoami)
SDB_VERSION=$(shell git show-ref -s HEAD)
SDB_NAME=storm.rs
SDB_DESCRIPTION="An OS for the storm"

JLINK_EXE ?= JLinkExe

$(BUILD_PLATFORM_DIR)/libplatform.rlib: $(call rwildcard,$(SRC_DIR)platform/storm,*.rs) $(BUILD_PLATFORM_DIR)/libcore.rlib $(BUILD_PLATFORM_DIR)/libhil.rlib $(BUILD_PLATFORM_DIR)/libsam4l.rlib $(BUILD_PLATFORM_DIR)/libdrivers.rlib | $(BUILD_PLATFORM_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_PLATFORM_DIR) $(SRC_DIR)platform/storm/lib.rs

all: $(BUILD_PLATFORM_DIR)/libplatform.rlib $(BUILD_PLATFORM_DIR)/arch.o $(BUILD_PLATFORM_DIR)/kernel.o


# .PHONY: rebuild-apps
# rebuild-apps: $(BUILD_PLATFORM_DIR)/crt1.o $(BUILD_PLATFORM_DIR)/arch.o $(BUILD_PLATFORM_DIR)/kernel.o $(APP_BINS)
# 	@echo "Relinking with APPS=\"$(APPS)\""
# 	@$(CC) $(LDFLAGS) -T$(LOADER) $^ -o $(BUILD_PLATFORM_DIR)/kernel.elf -ffreestanding -nostdlib -lc -lgcc
# 
# .PHONY: program
# program: $(BUILD_PLATFORM_DIR)/kernel.sdb
# 	$(SLOAD) flash $(BUILD_PLATFORM_DIR)/kernel.sdb
# 
# .PHONY: listen
# listen: program
# 	$(SLOAD) tail -i
# 
# .PHONY: jlink-program
# program-jlink: $(BUILD_PLATFORM_DIR)/kernel.bin
# 	@$(JLINK_EXE) $(SRC_DIR)platform/storm/prog.jlink
