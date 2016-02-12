ARCH = cortex-m0

RUSTC_FLAGS += -C opt-level=3 -Z no-landing-pads
RUSTC_FLAGS += --target $(SRC_DIR)chips/$(CHIP)/target.json
RUSTC_FLAGS += -Ctarget-cpu=$(ARCH) -C relocation_model=static
RUSTC_FLAGS += -g -C no-stack-check

CFLAGS_BASE = -mcpu=$(ARCH) -mthumb -mfloat-abi=soft
CFLAGS += $(CFLAGS_BASE) -g -O3 -std=gnu99 -nostartfiles
LOADER = $(SRC_DIR)chips/$(CHIP)/loader.ld
LDFLAGS += -T$(LOADER) -lm
OBJDUMP_FLAGS = --disassemble --source --disassembler-options=force-thumb

$(BUILD_DIR)/lib$(CHIP).rlib: $(call rwildcard,$(SRC_DIR)chips/$(CHIP),*.rs) $(BUILD_DIR)/libcore.rlib $(BUILD_DIR)/libhil.rlib $(BUILD_DIR)/libcommon.rlib
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_DIR) $(SRC_DIR)chips/$(CHIP)/lib.rs

$(BUILD_DIR)/crt1.o: $(SRC_DIR)chips/$(CHIP)/crt1.c
	@echo "Building $@"
	@$(CC) $(CFLAGS) -c $< -o $@ -lc -lgcc

