JLINK_OPTIONS = -device nrf51822 -if swd -speed 1000
JLINK = JLinkExe

all: $(BUILD_DIR)/main.bin

$(BUILD_DIR)/libplatform.rlib: $(call rwildcard,$(SRC_DIR)platform/$(PLATFORM),*.rs) $(BUILD_DIR)/libcore.rlib $(BUILD_DIR)/libhil.rlib $(BUILD_DIR)/lib$(CHIP).rlib $(BUILD_DIR)/libdrivers.rlib
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_DIR) $(SRC_DIR)platform/$(PLATFORM)/lib.rs

$(BUILD_DIR)/main.elf: $(BUILD_DIR)/crt1.o $(BUILD_DIR)/ctx_switch.o $(BUILD_DIR)/main.o $(APP_BINS)
	@echo "Linking $@"
	@$(CC) $(CFLAGS) $^ $(LDFLAGS) -o $@ -Wl,-Map=$(BUILD_DIR)/main.Map
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > $(BUILD_DIR)/main.lst
	@$(SIZE) $@

$(BUILD_DIR)/main.bin: $(BUILD_DIR)/main.elf
	@echo "Generating $@"
	@$(OBJCOPY) -Obinary $< $@

.PHONY: rebuild-apps
rebuild-apps: $(BUILD_DIR)/crt1.o $(BUILD_DIR)/arch.o $(BUILD_DIR)/main.o $(APP_BINS)
	@echo "Relinking with APPS=\"$(APPS)\""
	@$(CC) $(LDFLAGS) -T$(LOADER) $^ -o $(BUILD_DIR)/main.elf -ffreestanding -nostdlib -lc -lgcc

# "Flash" process:
# 1) set NVMC.CONFIG to 1 (Write enabled)
# 2) write firmware at address 0
# 3) set NVMC.CONFIG to 0 (Read only access)
.PHONY: program
program: $(BUILD_DIR)/main.bin
	echo \
	connect\\n\
	w4 4001e504 1\\n\
	loadbin $< 0\\n\
	w4 4001e504 0\\n\
	r\\n\
	g\\n\
	exit | $(JLINK) $(JLINK_OPTIONS)

# "Erase all" process:
# 1) set NVMC.CONFIG to 2 (Erase enabled)
# 2) set NVMC.ERASEALL to 1 (Start chip erase)
# 3) wait some time for erase to finish
# 4) set NVMC.CONFIG to 0 (Read only access)
.PHONY: erase-all
erase-all:
	echo \
	connect\\n\
	w4 4001e504 2\\n\
	w4 4001e50c 1\\n\
	sleep 100\\n\
	w4 4001e504 0\\n\
	r\\n\
	exit | $(JLINK) $(JLINK_OPTIONS)
