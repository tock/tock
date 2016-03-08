TOCK_APPS_DIR := ..
APP := ble_example

###
# app/*/Makefile
###

PROJECT_NAME = $(shell basename "$(realpath ./)")

APPLICATION_SRCS = $(notdir $(wildcard ./*.c))
APPLICATION_SRCS += led.c
APPLICATION_SRCS += simple_ble.c
APPLICATION_SRCS += simple_adv.c
APPLICATION_SRCS += eddystone.c

APPLICATION_SRCS += softdevice_handler.c
APPLICATION_SRCS += ble_advdata.c
APPLICATION_SRCS += ble_conn_params.c
APPLICATION_SRCS += app_timer.c

DEVICE := NRF51
SOFTDEVICE_MODEL := s110

NRF_BASE_PATH ?= $(TOCK_APPS_DIR)/$(APP)/nrf5x-base


###
# Our makefile?
###

include $(TOCK_APPS_DIR)/Makefile.Common.mk


# XXX FIXME
$(TOCK_APP_BUILD_DIR)/syscalls.o:	$(TOCK_DIR)/arch/$(ARCH)/syscalls.S | $(TOCK_APP_BUILD_DIR)
	$(AS) $(ASFLAGS) $^ -o $@

TOCK_LIBS += $(TOCK_APP_BUILD_DIR)/syscalls.o

.SECONDEXPANSION:
$(TOCK_APP_BUILD_DIR)/$(APP).elf: $$(OBJS) $(TOCK_LIBS) $(APP_LIBC) | $(TOCK_APP_BUILD_DIR) kernel
	$(TRACE_LD)
	$(Q)$(CC) $(CFLAGS) -g -Os -T $(APP_LINKER_SCRIPT) -nostdlib $^ -o $@
	$(Q)$(GENLST) $@ > $(TOCK_APP_BUILD_DIR)/$(APP).lst


###
# nrf5x-base/make/Makefile
###

# Guess nRF51 unless otherwise set
NRF_MODEL ?= nrf51
NRF_IC ?= nrf51822

# Set default board
BOARD ?= BOARD_NULL

# Set hardware memory sizes
RAM_KB   ?= 16
FLASH_KB ?= 256

# Default to the newest SDK
SDK_VERSION ?= 10

# Configure which stacks we need for various softdevices
ifeq ($(SOFTDEVICE_MODEL),s110)
USE_BLE := 1
SOFTDEVICE_VERSION ?= 8.0.0
NRF_MODEL := nrf51
endif

# Location for BLE Address if stored in Flash
BLEADDR_FLASH_LOCATION ?= 0x0007FFF8
CFLAGS += -DBLEADDR_FLASH_LOCATION=$(BLEADDR_FLASH_LOCATION)


# Add useful paths from nRF5x-base
LIBRARY_PATHS += $(NRF_BASE_PATH)/advertisement/
LIBRARY_PATHS += $(NRF_BASE_PATH)/devices/
LIBRARY_PATHS += $(NRF_BASE_PATH)/lib/
LIBRARY_PATHS += $(NRF_BASE_PATH)/peripherals/
LIBRARY_PATHS += $(NRF_BASE_PATH)/services/

SOURCE_PATHS += $(NRF_BASE_PATH)/advertisement/
SOURCE_PATHS += $(NRF_BASE_PATH)/devices/
SOURCE_PATHS += $(NRF_BASE_PATH)/lib/
SOURCE_PATHS += $(NRF_BASE_PATH)/peripherals/
SOURCE_PATHS += $(NRF_BASE_PATH)/services/
SOURCE_PATHS += $(NRF_BASE_PATH)/startup


# Add paths for each SDK version
ifeq ($(NRF_MODEL), nrf51)
ifeq ($(SDK_VERSION), 10)

	# Set the path
	SDK_PATH ?= $(NRF_BASE_PATH)/sdk/nrf51_sdk_10.0.0/

	# Other knowns about the SDK paths
	SDK_INCLUDE_PATH   = $(SDK_PATH)components/
	SDK_SOURCE_PATH    = $(SDK_PATH)components/
	CMSIS_INCLUDE_PATH = $(SDK_PATH)components/toolchain/gcc/

	# Need to add the paths for all the directories in the SDK.
	# Note that we do not use * because some folders have conflicting files.
	LIBRARY_PATHS += $(wildcard $(SDK_INCLUDE_PATH)libraries/*/)
	LIBRARY_PATHS += $(wildcard $(SDK_INCLUDE_PATH)drivers_nrf/*/)
	LIBRARY_PATHS += $(wildcard $(SDK_INCLUDE_PATH)drivers_ext/*/)
	LIBRARY_PATHS += $(wildcard $(SDK_INCLUDE_PATH)device/)
	LIBRARY_PATHS += $(wildcard $(SDK_INCLUDE_PATH)serialization/*/)
	LIBRARY_PATHS += $(SDK_INCLUDE_PATH)toolchain/gcc/
	LIBRARY_PATHS += $(SDK_INCLUDE_PATH)toolchain/

	SOURCE_PATHS += $(SDK_SOURCE_PATH)
	SOURCE_PATHS += $(wildcard $(SDK_SOURCE_PATH)*/)
	SOURCE_PATHS += $(wildcard $(SDK_SOURCE_PATH)libraries/*/)
	SOURCE_PATHS += $(wildcard $(SDK_SOURCE_PATH)drivers_nrf/*/)
	SOURCE_PATHS += $(wildcard $(SDK_SOURCE_PATH)drivers_ext/*/)
	SOURCE_PATHS += $(wildcard $(SDK_SOURCE_PATH)serialization/*/)

ifdef USE_BLE
	LIBRARY_PATHS += $(wildcard $(SDK_INCLUDE_PATH)ble/*/)
	LIBRARY_PATHS += $(wildcard $(SDK_INCLUDE_PATH)ble/ble_services/*/)
	SOURCE_PATHS += $(wildcard $(SDK_SOURCE_PATH)ble/*/)
	SOURCE_PATHS += $(wildcard $(SDK_SOURCE_PATH)ble/ble_services/*/)
	CFLAGS += -DBLE_STACK_SUPPORT_REQD
endif

ifdef SOFTDEVICE_MODEL
	LIBRARY_PATHS += $(SDK_INCLUDE_PATH)softdevice/common/softdevice_handler/
	LIBRARY_PATHS += $(SDK_INCLUDE_PATH)softdevice/$(SOFTDEVICE_MODEL)/headers/
	SOURCE_PATHS += $(SDK_INCLUDE_PATH)softdevice/common/softdevice_handler/
	SOURCE_PATHS += $(SDK_INCLUDE_PATH)softdevice/$(SOFTDEVICE_MODEL)/headers/
endif

endif # SDK 10
endif # nrf51


print-% : ; @echo $* = $($*)

LIBRARY_INCLUDES = $(addprefix -I,$(LIBRARY_PATHS))
CMSIS_INCLUDE = $(addprefix -I,$(CMSIS_INCLUDE_PATH))

VPATH = $(SOURCE_PATHS)
OBJDIR = $(TOCK_APP_BUILD_DIR)/_nrf_build/

OUTPUT_PATH = $(TOCK_APP_BUILD_DIR)/_nrf_build/

CFLAGS += -g -D$(DEVICE) -D$(BOARD) $(LIBRARY_INCLUDES) -MD -std=c11 -Os
COMPILE_ONLY += -c

OUTPUT_NAME ?= $(addsuffix _$(SOFTDEVICE_MODEL), $(PROJECT_NAME))
HEX = $(OUTPUT_PATH)$(OUTPUT_NAME).hex
ELF = $(OUTPUT_PATH)$(OUTPUT_NAME).elf
BIN = $(OUTPUT_PATH)$(OUTPUT_NAME).bin
LST = $(OUTPUT_PATH)$(OUTPUT_NAME).lst

SRCS = $(SYSTEM_FILE) $(notdir $(APPLICATION_SRCS))
OBJS = $(addprefix $(OUTPUT_PATH), $(SRCS:.c=.o)) $(addprefix $(OUTPUT_PATH),$(APPLICATION_LIBS))
DEPS = $(addprefix $(OUTPUT_PATH), $(SRCS:.c=.d))
LSTS = $(addprefix $(OUTPUT_PATH), $(SRCS:.c=.lst))
#SRCS_AS = $(START_CODE)
#OBJS_AS = $(addprefix $(OUTPUT_PATH), $(SRCS_AS:.s=.os))

#all: $(OBJS) $(OBJS_AS) $(HEX)
all: $(OBJS) $(LSTS) $(HEX)

#$(HEX): $(OBJS) $(OBJS_AS)
#	$(LD) $(LDFLAGS) $(OBJS_AS) $(OBJS) -o $(ELF)
#	$(OBJCOPY) -Oihex $(ELF) $(HEX)
#	$(OBJCOPY) -Obinary $(ELF) $(BIN)
#	$(SIZE) $(ELF)
$(HEX): $(OBJS)
	$(LD) $(LDFLAGS) $(OBJS) -o $(ELF)
	$(OBJCOPY) -Oihex $(ELF) $(HEX)
	$(OBJCOPY) -Obinary $(ELF) $(BIN)
	$(GENLST) $(ELF) > $(LST)
	$(LD) -r -b binary -o ../code_to_load.o $(BIN)
	$(OBJCOPY) --rename-section .data=.app ../code_to_load.o
	$(SIZE) $(ELF)

size: $(ELF)
	$(SIZE) $(ELF)

$(OUTPUT_PATH)%.o: %.c | $(OUTPUT_PATH)
	$(TRACE_CC)
	$(Q)$(CC) $(CFLAGS) $(COMPILE_ONLY) $< -o $@

$(OUTPUT_PATH)%.os: %.s | $(OUTPUT_PATH)
	$(TRACE_AS)
	$(Q)$(AS) $(ASFLAGS) $< -o $@

$(OUTPUT_PATH)%.lst: %.o | $(OUTPUT_PATH)
	$(TRACE_LST)
	$(Q)$(GENLST) $< > $@

.PHONY: all size clean


##################################################
# nrf5x-base/make/Makefile.posix
##################################################

$(OUTPUT_PATH):
	@mkdir -p $@

clean::
	$(Q)rm -rf $(OUTPUT_PATH)
	$(Q)rm -f *.jlink
	$(Q)rm -f JLink.log
	$(Q)rm -f .gdbinit
	$(Q)rm -f *.d

