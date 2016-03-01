ifneq ($(findstring cortex-m,$(ARCH)),cortex-m)
$(error ARCH must be cortex-m*, ARCH is "$(ARCH)")
endif

TOOLCHAIN := arm-none-eabi

AS := $(TOOLCHAIN)-as
ASFLAGS += -mcpu=$(ARCH) -mthumb

CC := $(TOOLCHAIN)-gcc
CXX := $(TOOLCHAIN)-g++
CPPFLAGS += -mcpu=$(ARCH) -mthumb -mfloat-abi=soft

LD := $(TOOLCHAIN)-ld
LDFLAGS += -T$(TOCK_PLATFORM_LINKER_SCRIPT) -lm

OBJCOPY := $(TOOLCHAIN)-objcopy

OBJDUMP := $(TOOLCHAIN)-objdump
OBJDUMP_FLAGS := --disassemble --source --disassembler-options=force-thumb
OBJDUMP_FLAGS += -C --section-headers

SIZE := $(TOOLCHAIN)-size

GDB := $(TOOLCHAIN)-gdb

