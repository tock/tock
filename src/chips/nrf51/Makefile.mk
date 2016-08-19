ARCH = cortex-m0

RUST_TARGET ?= $(SRC_DIR)chips/nrf51/target.json

RUSTC_FLAGS += -C opt-level=3 -Z no-landing-pads
RUSTC_FLAGS += --target $(RUST_TARGET)
RUSTC_FLAGS += -Ctarget-cpu=cortex-m0 -C relocation_model=static
RUSTC_FLAGS += -C no-stack-check -C soft-float -C target-feature="+soft-float"

CFLAGS_BASE = -mcpu=$(ARCH) -mthumb -mfloat-abi=soft
CFLAGS += $(CFLAGS_BASE) -g -O3 -std=gnu99 -nostartfiles
LOADER = $(SRC_DIR)chips/nrf51/layout.ld
LDFLAGS += -T$(LOADER) -lm
OBJDUMP_FLAGS := --disassemble --source --disassembler-options=force-thumb
OBJDUMP_FLAGS += -C --section-headers

$(BUILD_PLATFORM_DIR)/libnrf51.rlib: $(call rwildcard,$(SRC_DIR)chips/nrf51/src,*.rs) $(BUILD_PLATFORM_DIR)/libcortexm0.rlib $(BUILD_PLATFORM_DIR)/libcore.rlib $(BUILD_PLATFORM_DIR)/libcore.rlib $(BUILD_PLATFORM_DIR)/libhil.rlib $(BUILD_PLATFORM_DIR)/libcommon.rlib $(BUILD_PLATFORM_DIR)/libmain.rlib | $(BUILD_PLATFORM_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_PLATFORM_DIR) $(SRC_DIR)chips/nrf51/src/lib.rs

