CHIP := sam4l
ARCH := cortex-m4
TOCK_PLATFORM_LINKER_SCRIPT = $(TOCK_DIR)/chips/sam4l/loader.ld

include $(TOCK_APPS_DIR)/Makefile.Arm-M.mk

SLOAD=sload
SDB=$(TOCK_BUILD_DIR)/kernel.sdb
SDB_MAINTAINER=$(shell whoami)
SDB_VERSION=$(shell git show-ref -s HEAD)
SDB_NAME=storm.rs
SDB_DESCRIPTION="An OS for the storm"

JLINK_EXE ?= JLinkExe

# Apps to link may grow over time so defer expanding that
.SECONDEXPANSION:
$(TOCK_APP_BUILD_DIR)/kernel_and_app.elf: $(TOCK_BUILD_DIR)/kernel.elf $(TOCK_APP_BUILD_DIR)/$(APP).bin | $(TOCK_BUILD_DIR)
	@tput bold ; echo "Linking $@" ; tput sgr0
	$(Q)$(OBJCOPY) --add-section .$(APP)=$(TOCK_APP_BUILD_DIR)/$(APP).bin --set-section .$(APP)=alloc,load,readonly,code,contents --change-section-address=.$(APP)=0x20000 $(TOCK_BUILD_DIR)/kernel.elf $@
	$(Q)$(GENLST) $@ > $(TOCK_APP_BUILD_DIR)/kernel_and_app.lst
	$(Q)$(SIZE) $@

# XXX Temporary until new kernel build system in place
$(TOCK_BUILD_DIR)/kernel.elf: kernel

$(TOCK_APP_BUILD_DIR)/kernel_and_app.bin: $(TOCK_APP_BUILD_DIR)/kernel_and_app.elf
	@tput bold ; echo "Flattening $< to $@..." ; tput sgr0
	$(Q)$(OBJCOPY) -O binary $< $@

$(TOCK_APP_BUILD_DIR)/kernel_and_app.sdb: $(TOCK_APP_BUILD_DIR)/kernel_and_app.elf
	@tput bold ; echo "Packing SDB..." ; tput sgr0
	$(Q)$(SLOAD) pack -m "$(SDB_MAINTAINER)" -v "$(SDB_VERSION)" -n "$(SDB_NAME)" -d $(SDB_DESCRIPTION) -o $@ $<

all: $(TOCK_APP_BUILD_DIR)/kernel_and_app.elf
	@tput bold ; echo "Finished building $(APP) for $(TOCK_PLATFORM)" ; tput sgr0

.PHONY: program
program: $(TOCK_APP_BUILD_DIR)/kernel_and_app.sdb
	$(SLOAD) flash $(TOCK_APP_BUILD_DIR)/kernel_and_app.sdb

.PHONY: listen
listen: program
	$(SLOAD) tail -i

.PHONY: jlink-program
program-jlink: $(TOCK_APP_BUILD_DIR)/kernel_and_app.bin
	@$(JLINK_EXE) $(SRC_DIR)platform/storm/prog.jlink

