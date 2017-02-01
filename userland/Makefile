# userland master makefile. Included by application makefiles

# Check for a ~/ at the beginning of a path variable (TOCK_USERLAND_BASE_DIR).
# Make will not properly expand this.
ifdef TOCK_USERLAND_BASE_DIR
    ifneq (,$(findstring BEGINNINGOFVARIABLE~/,BEGINNINGOFVARIABLE$(TOCK_USERLAND_BASE_DIR)))
        $(error Hi! Using "~" in Makefile variables is not supported. Use "$$(HOME)" instead)
    endif
endif

TOCK_USERLAND_BASE_DIR ?= ..
TOCK_BASE_DIR ?= $(TOCK_USERLAND_BASE_DIR)/..
BUILDDIR ?= build/$(TOCK_ARCH)
TOCK_BOARD ?= storm
TOCK_ARCH ?= cortex-m4
LIBTOCK ?= $(TOCK_USERLAND_BASE_DIR)/libtock/build/$(TOCK_ARCH)/libtock.a

TOOLCHAIN := arm-none-eabi
AS := $(TOOLCHAIN)-as
CC := $(TOOLCHAIN)-gcc
CXX := $(TOOLCHAIN)-g++
READELF := $(TOOLCHAIN)-readelf

# PACKAGE_NAME is used to identify the application for IPC and for error reporting
PACKAGE_NAME ?= $(notdir $(shell pwd))

# This could be replaced with an installed version of `elf2tbf`
ELF2TBF ?= cargo run --manifest-path $(abspath $(TOCK_USERLAND_BASE_DIR))/tools/elf2tbf/Cargo.toml --
ELF2TBF_ARGS += -n $(PACKAGE_NAME)

# Collect all desired built output.
OBJS += $(patsubst %.c,$(BUILDDIR)/%.o,$(C_SRCS))
OBJS += $(patsubst %.cc,$(BUILDDIR)/%.o,$(CXX_SRCS))

# Divine or set the stack size
# First, if the make variable STACK_SIZE is set, we add that to the flags
ifdef STACK_SIZE
    CPPFLAGS += -DSTACK_SIZE=$(STACK_SIZE)
endif
# Next, we scan all the reasonable flags for '-DSTACK_SIZE' invocations.
# If there are conflicting values, throw an error.
# If there's none, set a default size.
# If there's one (or multiple identical) grab the value so we can use it elsewhere.
FLAGS_STACK_SIZE := $(shell echo $(CFLAGS) $(CPPFLAGS) $(CXXFLAGS) | tr ' ' '\n' | grep STACK_SIZE | sort | uniq)
ifneq ($(shell echo $(FLAGS_STACK_SIZE) | tr ' ' '\n' | wc -l | tr -d ' '),1)
    $(error Conflicting STACK_SIZE values: $(FLAGS_STACK_SIZE))
endif
ifeq ($(FLAGS_STACK_SIZE),)
    STACK_SIZE := 2048
    CPPFLAGS += -DSTACK_SIZE=$(STACK_SIZE)
else
    STACK_SIZE := $(shell echo $(FLAGS_STACK_SIZE) | cut -d '=' -f2)
endif

# Also detect and pass through APP_HEAP_SIZE and KERNEL_HEAP_SIZE make variables
ifdef APP_HEAP_SIZE
    CPPFLAGS += -DAPP_HEAP_SIZE=$(APP_HEAP_SIZE)
endif
ifdef KERNEL_HEAP_SIZE
    CPPFLAGS += -DKERNEL_HEAP_SIZE=$(KERNEL_HEAP_SIZE)
endif

ASFLAGS += -mcpu=$(TOCK_ARCH) -mthumb

# n.b. make convention is that CPPFLAGS are shared for C and C++ sources
# [CFLAGS is C only, CXXFLAGS is C++ only]
CFLAGS   += -std=gnu11
CPPFLAGS += -I$(TOCK_USERLAND_BASE_DIR)/libtock -g -mcpu=$(TOCK_ARCH) -mthumb -mfloat-abi=soft
CPPFLAGS += \
	    -frecord-gcc-switches\
	    -Os\
	    -fdata-sections -ffunction-sections\
	    -fstack-usage -Wstack-usage=$(STACK_SIZE)\
	    -Wall\
	    -Wextra\
	    -Wl,-gc-sections\
	    -g\
	    -fPIC\
	    -msingle-pic-base\
	    -mpic-register=r9\
	    -mno-pic-data-is-text-relative
OBJDUMP_FLAGS += --disassemble-all --source --disassembler-options=force-thumb -C --section-headers

LIBS += $(LIBTOCK)
LIBS += $(TOCK_USERLAND_BASE_DIR)/newlib/libc.a
LIBS += $(TOCK_USERLAND_BASE_DIR)/newlib/libm.a
LIBS += $(TOCK_USERLAND_BASE_DIR)/libc++/libstdc++.a
LIBS += $(TOCK_USERLAND_BASE_DIR)/libc++/libsupc++.a
LIBS += $(TOCK_USERLAND_BASE_DIR)/libc++/libgcc.a

# First step doesn't actually compile, just generate header dependency information
# More info on our approach here: http://stackoverflow.com/questions/97338
$(BUILDDIR)/%.o: %.c | $(BUILDDIR)
	$(TRACE_DEP)
	$(Q)$(CC) $(CFLAGS) $(CPPFLAGS) -MF"$(@:.o=.d)" -MG -MM -MP -MT"$(@:.o=.d)@" -MT"$@" "$<"
	$(TRACE_CC)
	$(Q)$(CC) $(CFLAGS) $(CPPFLAGS) -c -o $@ $<

$(BUILDDIR)/%.o: %.cc | $(BUILDDIR)
	$(TRACE_DEP)
	$(Q)$(CXX) $(CXXFLAGS) $(CPPFLAGS) -MF"$(@:.o=.d)" -MG -MM -MP -MT"$(@:.o=.d)@" -MT"$@" "$<"
	$(TRACE_CXX)
	$(Q)$(CXX) $(CXXFLAGS) $(CPPFLAGS) -c -o $@ $<

LINKER ?= $(TOCK_USERLAND_BASE_DIR)/linker.ld

SIZE := $(TOOLCHAIN)-size
OBJDUMP := $(TOOLCHAIN)-objdump

.PHONY:	all
all:	$(BUILDDIR)/app.bin size

.PHONY: size
size:	$(BUILDDIR)/app.elf
	@$(SIZE) $<

.PHONY: debug
debug:	$(BUILDDIR)/app.lst

$(BUILDDIR)/app.lst: $(BUILDDIR)/app.elf
	$(TRACE_LST)
	$(Q)$(OBJDUMP) $(OBJDUMP_FLAGS) $< > $(BUILDDIR)/app.lst

# Include the libtock makefile. Adds rules that will rebuild library when needed
include $(TOCK_USERLAND_BASE_DIR)/libtock/Makefile

$(BUILDDIR):
	$(Q)mkdir -p $(BUILDDIR)

$(BUILDDIR)/app.elf: $(OBJS) $(TOCK_USERLAND_BASE_DIR)/newlib/libc.a $(LIBTOCK) | $(BUILDDIR)
	$(TRACE_LD)
	$(Q)$(CC) -Wl,--gc-sections -Wl,--emit-relocs --entry=_start $(CFLAGS) $(CPPFLAGS) -T $(LINKER) -nostdlib -Wl,--start-group $(OBJS) $(LIBS) -Wl,--end-group -Wl,-Map=$(BUILDDIR)/app.Map -o $@

$(BUILDDIR)/app.bin: $(BUILDDIR)/app.elf | $(BUILDDIR) validate_gcc_flags
	$(TRACE_BIN)
	$(Q)$(ELF2TBF) $(ELF2TBF_ARGS) -o $@ $<

.PHONY: validate_gcc_flags
validate_gcc_flags: $(BUILDDIR)/app.elf
ifndef TOCK_NO_CHECK_SWITCHES
	$(Q)$(READELF) -p .GCC.command.line $< 2>&1 | grep -q "does not exist" && { echo "Error: Missing section .GCC.command.line"; echo ""; echo "Tock requires that applications are built with"; echo "  -frecord-gcc-switches"; echo "to validate that all required flags were used"; echo ""; echo "You can skip this check by defining the make variable TOCK_NO_CHECK_SWITCHES"; exit 1; } || exit 0
	$(Q)$(READELF) -p .GCC.command.line $< | grep -q -- -msingle-pic-base && $(READELF) -p .GCC.command.line $< | grep -q -- -mpic-register=r9 && $(READELF) -p .GCC.command.line $< | grep -q -- -mno-pic-data-is-text-relative || { echo "Error: Missing required build flags."; echo ""; echo "Tock requires applications are built with"; echo "  -msingle-pic-base"; echo "  -mpic-register=r9"; echo "  -mno-pic-data-is-text-relative"; echo "But one or more of these flags are missing"; echo ""; echo "To see the flags your application was built with, run"; echo "$(READELF) -p .GCC.command.line $<"; echo ""; exit 1; }
endif

.PHONY:
clean::
	rm -Rf $(BUILDDIR)

# for programming individual apps, include platform app makefile
#	conditionally included in case it doesn't exist for a board
-include $(TOCK_BASE_DIR)/boards/$(TOCK_BOARD)/Makefile-app



#########################################################################################
## Pretty-printing rules

# If environment variable V is non-empty, be verbose
ifneq ($(V),)
Q=
TRACE_BIN =
TRACE_DEP =
TRACE_CC  =
TRACE_CXX =
TRACE_LD  =
TRACE_AR  =
TRACE_AS  =
TRACE_LST =
ELF2TBF_ARGS += -v
else
Q=@
TRACE_BIN = @echo " BIN       " $@
TRACE_DEP = @echo " DEP       " $<
TRACE_CC  = @echo "  CC       " $<
TRACE_CXX = @echo " CXX       " $<
TRACE_LD  = @echo "  LD       " $@
TRACE_AR  = @echo "  AR       " $@
TRACE_AS  = @echo "  AS       " $<
TRACE_LST = @echo " LST       " $<
endif



#########################################################################################
# Include dependency rules for picking up header changes (by convention at bottom of makefile)
OBJS_NO_ARCHIVES=$(filter %.o,$(OBJS))
-include $(OBJS_NO_ARCHIVES:.o=.d)
