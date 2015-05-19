RUSTC_FLAGS += -C opt-level=2 -Z no-landing-pads
RUSTC_FLAGS += --target platform/storm/target.json
RUSTC_FLAGS += -Ctarget-cpu=cortex-m4 -C relocation_model=static
RUSTC_FLAGS += -g -C no-stack-check

CFLAGS += -g -O3 -std=gnu99 -mcpu=cortex-m4 -mthumb -nostdlib
LDFLAGS += -Tplatform/storm/loader.ld

SLOAD=sload
SDB=$(BUILD_DIR)/main.sdb
SDB_MAINTAINER=$(shell whoami)
SDB_VERSION=$(shell git show-ref -s HEAD)
SDB_NAME=storm.rs
SDB_DESCRIPTION="An OS for the storm"

$(BUILD_DIR)/crt1.o: platform/storm/crt1.c
	@echo "+ storm crt1"
	@$(CC) $(CFLAGS) $(INC_FLAGS) -c $< -o $@

$(BUILD_DIR)/main.elf: $(BUILD_DIR)/main.o
	@echo "Linking $@"
	@$(CC) $(CFLAGS) $(LDFLAGS) $^ -o $@ -ffreestanding -lgcc -lc

$(BUILD_DIR)/%.sdb: $(BUILD_DIR)/%.elf
	@echo "Packing SDB..."
	@$(SLOAD) pack -m "$(SDB_MAINTAINER)" -v "$(SDB_VERSION)" -n "$(SDB_NAME)" -d $(SDB_DESCRIPTION) -o $@ $<

all: $(BUILD_DIR)/main.sdb

.PHONY: program

program: $(BUILD_DIR)/main.sdb
	sload flash $(BUILD_DIR)/main.sdb

