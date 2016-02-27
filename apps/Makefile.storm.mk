CHIP := sam4l
ARCH := cortex-m4

TOOLCHAIN := arm-none-eabi

AS := $(TOOLCHAIN)-as
ASFLAGS += -mcpu=$(ARCH) -mthumb

CC := $(TOOLCHAIN)-gcc
CXX := $(TOOLCHAIN)-g++
CPPFLAGS += -mcpu=$(ARCH) -mthumb -mfloat-abi=soft

LD := $(TOOLCHAIN)-ld

OBJCOPY := $(TOOLCHAIN)-objcopy

OBJDUMP := $(TOOLCHAIN)-objdump
OBJDUMP_FLAGS := --disassemble --source --disassembler-options=force-thumb
OBJDUMP_FLAGS += -C -g --section-headers

SIZE := $(TOOLCHAIN)-size

TOCK_PLATFORM_LINKER_SCRIPT = $(TOCK_DIR)/chips/sam4l/loader.ld
LDFLAGS += -T$(TOCK_PLATFORM_LINKER_SCRIPT) -lm

SLOAD=sload
SDB=$(TOCK_BUILD_DIR)/kernel.sdb
SDB_MAINTAINER=$(shell whoami)
SDB_VERSION=$(shell git show-ref -s HEAD)
SDB_NAME=storm.rs
SDB_DESCRIPTION="An OS for the storm"

JLINK_EXE ?= JLinkExe

# Apps to link may grow over time so defer expanding that
.SECONDEXPANSION:
$(TOCK_APP_BUILD_DIR)/kernel_and_app.elf: $(TOCK_BUILD_DIR)/arch.o $(TOCK_BUILD_DIR)/kernel.o $$(APPS_TO_LINK_TO_KERNEL) | $(TOCK_BUILD_DIR)
	@tput bold ; echo "Linking $@" ; tput sgr0
	$(CC) $(CFLAGS) $(CPPFLAGS) $^ $(LDFLAGS) -Wl,-Map=$(TOCK_APP_BUILD_DIR)/kernel_and_app.Map -o $@
	$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > $(TOCK_APP_BUILD_DIR)/kernel_and_app.lst
	$(SIZE) $@

$(TOCK_APP_BUILD_DIR)/kernel_and_app.bin: $(TOCK_APP_BUILD_DIR)/kernel_and_app.elf
	@tput bold ; echo "Flattening $< to $@..." ; tput sgr0
	$(OBJCOPY) -O binary $< $@

$(TOCK_APP_BUILD_DIR)/kernel_and_app.sdb: $(TOCK_APP_BUILD_DIR)/kernel_and_app.elf
	@tput bold ; echo "Packing SDB..." ; tput sgr0
	$(SLOAD) pack -m "$(SDB_MAINTAINER)" -v "$(SDB_VERSION)" -n "$(SDB_NAME)" -d $(SDB_DESCRIPTION) -o $@ $<

all: $(TOCK_APP_BUILD_DIR)/kernel_and_app.sdb

.PHONY: program
program: $(TOCK_APP_BUILD_DIR)/kernel_and_app.sdb
	$(SLOAD) flash $(TOCK_APP_BUILD_DIR)/kernel_and_app.sdb

.PHONY: listen
listen: program
	$(SLOAD) tail -i

.PHONY: jlink-program
program-jlink: $(TOCK_APP_BUILD_DIR)/kernel_and_app.bin
	@$(JLINK_EXE) $(SRC_DIR)platform/storm/prog.jlink

