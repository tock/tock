SLOAD=sload
SDB=$(BUILD_PLATFORM_DIR)/main.sdb
SDB_MAINTAINER=$(shell whoami)
SDB_VERSION=$(shell git show-ref -s HEAD)
SDB_NAME=storm.rs
SDB_DESCRIPTION="An OS for the storm"

JLINK_EXE ?= JLinkExe

$(BUILD_PLATFORM_DIR)/libplatform.rlib: $(call rwildcard,$(SRC_DIR)platform/storm,*.rs) $(BUILD_PLATFORM_DIR)/libcore.rlib $(BUILD_PLATFORM_DIR)/libhil.rlib $(BUILD_PLATFORM_DIR)/libsam4l.rlib $(BUILD_PLATFORM_DIR)/libdrivers.rlib | $(BUILD_PLATFORM_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_PLATFORM_DIR) $(SRC_DIR)platform/storm/lib.rs

$(BUILD_PLATFORM_DIR)/main.elf: $(BUILD_PLATFORM_DIR)/arch.o $(BUILD_PLATFORM_DIR)/main.o $(APP_BINS) | $(BUILD_PLATFORM_DIR)
	@echo "Linking $@"
	@$(CC) $(CFLAGS) $^ $(LDFLAGS) -o $@ -Wl,-Map=$(BUILD_PLATFORM_DIR)/main.Map
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > $(BUILD_PLATFORM_DIR)/main.lst
	@$(SIZE) $@

$(BUILD_PLATFORM_DIR)/%.bin: $(BUILD_PLATFORM_DIR)/%.elf | $(BUILD_PLATFORM_DIR)
	@echo "Flattening $< to $@..."
	@$(TOOLCHAIN)objcopy -O binary $< $@

$(BUILD_PLATFORM_DIR)/%.sdb: $(BUILD_PLATFORM_DIR)/%.elf | $(BUILD_PLATFORM_DIR)
	@echo "Packing SDB..."
	@$(SLOAD) pack -m "$(SDB_MAINTAINER)" -v "$(SDB_VERSION)" -n "$(SDB_NAME)" -d $(SDB_DESCRIPTION) -o $@ $<

all: $(BUILD_PLATFORM_DIR)/main.sdb | $(BUILD_PLATFORM_DIR)

.PHONY: rebuild-apps
rebuild-apps: $(BUILD_PLATFORM_DIR)/crt1.o $(BUILD_PLATFORM_DIR)/arch.o $(BUILD_PLATFORM_DIR)/main.o $(APP_BINS)
	@echo "Relinking with APPS=\"$(APPS)\""
	@$(CC) $(LDFLAGS) -T$(LOADER) $^ -o $(BUILD_PLATFORM_DIR)/main.elf -ffreestanding -nostdlib -lc -lgcc

.PHONY: program
program: $(BUILD_PLATFORM_DIR)/main.sdb
	$(SLOAD) flash $(BUILD_PLATFORM_DIR)/main.sdb

.PHONY: listen
listen: program
	$(SLOAD) tail -i

.PHONY: jlink-program
program-jlink: $(BUILD_PLATFORM_DIR)/main.bin
	@$(JLINK_EXE) $(SRC_DIR)platform/storm/prog.jlink
