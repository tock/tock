ifneq ($(findstring cortex-m,$(ARCH)),cortex-m)
$(error ARCH must be cortex-m*, ARCH is "$(ARCH)")
endif

TOOLCHAIN := arm-none-eabi

AS := $(TOOLCHAIN)-as
ASFLAGS += -mcpu=$(ARCH) -mthumb

CC := $(TOOLCHAIN)-gcc
CXX := $(TOOLCHAIN)-g++
CPPFLAGS += -g -mcpu=$(ARCH) -mthumb -mfloat-abi=soft

LD := $(TOOLCHAIN)-ld
LDFLAGS += -g -T$(TOCK_PLATFORM_LINKER_SCRIPT) -lm

OBJCOPY := $(TOOLCHAIN)-objcopy

OBJDUMP := $(TOOLCHAIN)-objdump
OBJDUMP_FLAGS := --disassemble-all --source --disassembler-options=force-thumb
OBJDUMP_FLAGS += -C --section-headers

ifdef TOCK_BUILD_GENERATE_LSTS
GENLST = $(OBJDUMP) $(OBJDUMP_FLAGS)
else
GENLST = printf "Set TOCK_BUILD_GENERATE_LSTS to generate lst files (slow to build), or run:\n$(OBJDUMP) $(OBJDUMP_FLAGS) <file>.elf > <file>.lst\n"
endif

SIZE := $(TOOLCHAIN)-size

GDB := $(TOOLCHAIN)-gdb

