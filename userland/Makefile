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

# Set default region sizes
STACK_SIZE       ?= 2048
APP_HEAP_SIZE    ?= 1024
KERNEL_HEAP_SIZE ?= 1024

# This could be replaced with an installed version of `elf2tbf`
ELF2TBF ?= cargo run --manifest-path $(abspath $(TOCK_USERLAND_BASE_DIR))/tools/elf2tbf/Cargo.toml --
ELF2TBF_ARGS += -n $(PACKAGE_NAME)

# Collect all desired built output.
OBJS += $(patsubst %.c,$(BUILDDIR)/%.o,$(C_SRCS))
OBJS += $(patsubst %.cc,$(BUILDDIR)/%.o,$(CXX_SRCS))

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

# Extra warning flags not enabled by Wall or Wextra.
# I read through the gcc manual and grabbed the ones that I thought might be
# interesting / useful. I've left a few commented that may be interesting but I
# want to think about more
CPPFLAGS += -Winit-self #                # { int i = i }
CPPFLAGS += -Wswitch-enum #              # switch on an enum doesn't cover all cases
CPPFLAGS += -Wunused-parameter #         # function parameter is unused aside from its declaration
CPPFLAGS += -Wfloat-equal #              # warn if floats used with '=' operator, likely imprecise
CPPFLAGS += -Wshadow #                   # int foo(int a) { int a = 1; } inner a shadows outer a
CPPFLAGS += -Wpointer-arith #            # sizeof things not define'd (i.e. sizeof(void))
CPPFLAGS += -Wwrite-strings #            # { char* c = "foo"; c[0] = 'b' } <-- "foo" should be r/o
CPPFLAGS += -Wlogical-op #               # "suspicous use of logical operators in expressions" (a lint)
CPPFLAGS += -Wmissing-declarations #     # ^same? not sure how these differ
CPPFLAGS += -Wmissing-field-initializers # if init'ing struct w/out field names, warn if not all used
CPPFLAGS += -Wmissing-noreturn #         # __attribute__((noreturn)) like -> ! in Rust, should use it
CPPFLAGS += -Wmissing-format-attribute # # something looks printf-like but isn't marked as such
CPPFLAGS += -Wredundant-decls #          # { int i; int i; } (a lint)

# C-only warnings
CFLAGS += -Wbad-function-cast #          # not obvious when this would trigger, could drop if annoying
CFLAGS += -Wmissing-prototypes #         # global fn defined w/out prototype (should be static or in .h)
CFLAGS += -Wnested-externs #             # mis/weird-use of extern keyword

# CXX-only warnings
# XXX todo

#CPPFLAGS += -Wundef                      # undefined identifier is evaluated in an `#if' directive
#                                         ^ Lots of library #if SAMD || SMAR21 stuff
#                                           Should probably be ifdef, but too much noise
#CPPFLAGS += -Wconversion                 # implicit conversion that may unexpectedly alter value
#                                         ^ A ton of these from syscalls I think, XXX look later
#CPPFLAGS += -Wpadded               -- Noisy for argument passing structs
#CPPFLAGS += -Wunreachable-code     -- Obnoxious during development
#CPPFLAGS += -Wstrict-prototypes    -- Who wants a warning for not decarling main?
#CPPFLAGS += -Wvla                  -- XXX Didn't try, but interested
#CPPFLAGS += -Wmissing-include-dirs -- XXX Didn't try, afriad could be annoying


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
	$(Q)$(CC) $(CFLAGS) $(CPPFLAGS)\
	    -Wl,--warn-common\
	    -Wl,--gc-sections -Wl,--emit-relocs\
	    --entry=_start\
	    -Xlinker --defsym=STACK_SIZE=$(STACK_SIZE)\
	    -Xlinker --defsym=APP_HEAP_SIZE=$(APP_HEAP_SIZE)\
	    -Xlinker --defsym=KERNEL_HEAP_SIZE=$(KERNEL_HEAP_SIZE)\
	    -T $(LINKER)\
	    -nostdlib\
	    -Wl,--start-group $(OBJS) $(LIBS) -Wl,--end-group\
	    -Wl,-Map=$(BUILDDIR)/app.Map\
	    -o $@

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
