ARCH = cortex-m0

RUSTC_FLAGS += -C opt-level=3 -Z no-landing-pads
RUSTC_FLAGS += --target $(SRC_DIR)chips/$(CHIP)/makefile_target.json
RUSTC_FLAGS += -Ctarget-cpu=$(ARCH) -C relocation_model=static
RUSTC_FLAGS += -C no-stack-check

CFLAGS_BASE = -mcpu=$(ARCH) -mthumb -mfloat-abi=soft
CFLAGS += $(CFLAGS_BASE) -g -O3 -std=gnu99 -nostartfiles
LOADER = $(SRC_DIR)chips/$(CHIP)/layout.ld
LDFLAGS += -T$(LOADER) -lm
OBJDUMP_FLAGS := --disassemble --source --disassembler-options=force-thumb
OBJDUMP_FLAGS += -C --section-headers

$(BUILD_PLATFORM_DIR)/lib$(CHIP).rlib: $(call rwildcard,$(SRC_DIR)chips/$(CHIP)/src,*.rs) $(BUILD_PLATFORM_DIR)/libcore.rlib $(BUILD_PLATFORM_DIR)/libhil.rlib $(BUILD_PLATFORM_DIR)/libcommon.rlib $(BUILD_PLATFORM_DIR)/libcrt1.a | $(BUILD_PLATFORM_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_PLATFORM_DIR) $(SRC_DIR)chips/$(CHIP)/src/lib.rs -l static=crt1

$(BUILD_PLATFORM_DIR)/crt1.o: $(SRC_DIR)chips/$(CHIP)/src/crt1.c | $(BUILD_PLATFORM_DIR)
	@echo "Building $@"
	@$(CC) $(CFLAGS) -c $< -o $@ -lc -lgcc

$(BUILD_PLATFORM_DIR)/libcrt1.a: $(BUILD_PLATFORM_DIR)/crt1.o | $(BUILD_PLATFORM_DIR)
	@echo "Building $@"
	$(TOOLCHAIN)ar rcs $@ $<

