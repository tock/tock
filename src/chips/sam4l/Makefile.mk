RUSTC_FLAGS += -C opt-level=2 -Z no-landing-pads
RUSTC_FLAGS += --target src/chips/sam4l/target.json
RUSTC_FLAGS += -Ctarget-cpu=cortex-m4 -C relocation_model=static
RUSTC_FLAGS += -g -C no-stack-check

CFLAGS += -g -O3 -std=gnu99 -mcpu=cortex-m4 -mthumb -nostdlib
LDFLAGS += -Tsrc/chips/sam4l/loader.ld

$(BUILD_DIR)/libsam4l.rlib: $(call rwildcard,src/chips/sam4l,*.rs) $(BUILD_DIR)/libcore.rlib $(BUILD_DIR)/libhil.rlib $(BUILD_DIR)/libcommon.rlib
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_DIR) src/chips/sam4l/lib.rs

$(BUILD_DIR)/crt1.o: src/chips/sam4l/crt1.c
	@echo "+ storm crt1"
	@$(CC) $(CFLAGS) $(INC_FLAGS) -c $< -o $@

