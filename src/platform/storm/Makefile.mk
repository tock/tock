SLOAD=sload
SDB=$(BUILD_DIR)/main.sdb
SDB_MAINTAINER=$(shell whoami)
SDB_VERSION=$(shell git show-ref -s HEAD)
SDB_NAME=storm.rs
SDB_DESCRIPTION="An OS for the storm"

$(BUILD_DIR)/libplatform.rlib: $(call rwildcard,$(SRC_DIR)platform/storm,*.rs) $(BUILD_DIR)/libcore.rlib $(BUILD_DIR)/libhil.rlib $(BUILD_DIR)/libsam4l.rlib $(BUILD_DIR)/libdrivers.rlib
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_DIR) $(SRC_DIR)platform/storm/lib.rs

$(BUILD_DIR)/main.elf: $(BUILD_DIR)/crt1.o $(BUILD_DIR)/arch.o $(BUILD_DIR)/main.o $(APP_BINS)
	@echo "Linking $@"
	@$(CC) $(LDFLAGS) -T$(LOADER) $^ -o $@ -ffreestanding -nostdlib -lc -lgcc

$(BUILD_DIR)/%.sdb: $(BUILD_DIR)/%.elf
	@echo "Packing SDB..."
	@$(SLOAD) pack -m "$(SDB_MAINTAINER)" -v "$(SDB_VERSION)" -n "$(SDB_NAME)" -d $(SDB_DESCRIPTION) -o $@ $<

all: $(BUILD_DIR)/main.sdb

.PHONY: rebuild-apps
rebuild-apps: $(BUILD_DIR)/crt1.o $(BUILD_DIR)/arch.o $(BUILD_DIR)/main.o $(APP_BINS)
	@echo "Relinking with APPS=\"$(APPS)\""
	@$(CC) $(LDFLAGS) -T$(LOADER) $^ -o $(BUILD_DIR)/main.elf -ffreestanding -nostdlib -lc -lgcc

.PHONY: program
program: $(BUILD_DIR)/main.sdb
	$(SLOAD) flash $(BUILD_DIR)/main.sdb

