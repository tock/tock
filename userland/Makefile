# userland master makefile. Included by application makefiles

# Default target:
all:


# Remove built-in rules and variables
# n.b. no-op for make --version < 4.0
MAKEFLAGS += -r
MAKEFLAGS += -R

# http://stackoverflow.com/questions/10858261/abort-makefile-if-variable-not-set
# Check that given variables are set and all have non-empty values,
# die with an error otherwise.
#
# Params:
#   1. Variable name(s) to test.
#   2. (optional) Error message to print.
check_defined = \
    $(strip $(foreach 1,$1, \
        $(call __check_defined,$1,$(strip $(value 2)))))
__check_defined = \
    $(if $(value $1),, \
      $(error Undefined $1$(if $2, ($2))))

# Check for a ~/ at the beginning of a path variable (TOCK_USERLAND_BASE_DIR).
# Make will not properly expand this.
ifdef TOCK_USERLAND_BASE_DIR
    ifneq (,$(findstring BEGINNINGOFVARIABLE~/,BEGINNINGOFVARIABLE$(TOCK_USERLAND_BASE_DIR)))
        $(error Hi! Using "~" in Makefile variables is not supported. Use "$$(HOME)" instead)
    endif
endif

# Default platform
TOCK_BOARD ?= storm
TOCK_USERLAND_BASE_DIR ?= ..
TOCK_BASE_DIR ?= $(TOCK_USERLAND_BASE_DIR)/..

# Include platform app makefile.
#  - Should set appropriate TOCK_ARCH for this platform
#  - Adds rules for loading applications onto this board
# Conditionally included in case it doesn't exist for a board
-include $(TOCK_BASE_DIR)/boards/$(TOCK_BOARD)/Makefile-app

ifndef TOCK_ARCH
    $(warning The board "$(TOCK_BOARD)" did not specify an architecture)
    $(warning Defaulting to cortex-m0 for maximum compatibility)
    $(warning This will result in less efficient code if your platform supports)
    $(warning a more advanced instruction set. Update the board Makefile-app or)
    $(warning define TOCK_ARCH in your Makefile to fix.)
    TOCK_ARCH := cortex-m0
endif


# TODO(Pat) at some point this should change names to
#  - BUILDDIR: build/
#  - ARCHBUILDDIR: build/$(TOCK_ARCH)
#  etc

# BUILDDIR holds architecture dependent, but board-independent outputs
BUILDDIR ?= build/$(TOCK_ARCH)
$(BUILDDIR):
	$(Q)mkdir -p $(BUILDDIR)

# BOARD_BUILDDIR holds board-specific outputs
BOARD_BUILDDIR ?= build/$(TOCK_BOARD)
$(BOARD_BUILDDIR):
	$(Q)mkdir -p $(BOARD_BUILDDIR)


LIBTOCK ?= $(TOCK_USERLAND_BASE_DIR)/libtock/build/$(TOCK_ARCH)/libtock.a

# PACKAGE_NAME is used to identify the application for IPC and for error reporting
PACKAGE_NAME ?= $(notdir $(shell pwd))

# Set default region sizes
STACK_SIZE       ?= 2048
APP_HEAP_SIZE    ?= 1024
KERNEL_HEAP_SIZE ?= 1024

ifdef HEAP_SIZE
    $(warning The variable HEAP_SIZE is set but will not be used.)
    $(warning Tock has two heaps, the application heap which is memory your program)
    $(warning uses and the kernel heap or grant regions, which is memory dynamically)
    $(warning allocated by drivers on behalf of your program.)
    $(warning )
    $(warning These regions are controlled by the APP_HEAP_SIZE and KERNEL_HEAP_SIZE)
    $(warning variables respectively.)
endif

TOOLCHAIN := arm-none-eabi
AR := $(TOOLCHAIN)-ar
AS := $(TOOLCHAIN)-as
CC := $(TOOLCHAIN)-gcc
CXX := $(TOOLCHAIN)-g++
OBJDUMP := $(TOOLCHAIN)-objdump
RANLIB := $(TOOLCHAIN)-ranlib
READELF := $(TOOLCHAIN)-readelf
SIZE := $(TOOLCHAIN)-size

# Validate the the toolchain is new enough (known not to work for gcc <= 5.1)
CC_VERSION_MAJOR := $(shell $(CC) -dumpversion | cut -d '.' -f1)
ifeq (1,$(shell expr $(CC_VERSION_MAJOR) \>= 6))
  # Opportunistically turn on gcc 6.0+ warnings since we're already version checking:
  CPPFLAGS += -Wduplicated-cond #          # if (p->q != NULL) { ... } else if (p->q != NULL) { ... }
  CPPFLAGS += -Wnull-dereference #         # deref of NULL (thought default if -fdelete-null-pointer-checks, in -Os, but no?)
else
  ifneq (5,$(CC_VERSION_MAJOR))
    $(error Your compiler is too old. Need gcc version > 5.1)
  endif
    CC_VERSION_MINOR := $(shell $(CC) -dumpversion | cut -d '.' -f2)
  ifneq (1,$(shell expr $(CC_VERSION_MINOR) \> 1))
    $(error Your compiler is too old. Need gcc version > 5.1)
  endif
endif

# This could be replaced with an installed version of `elf2tbf`
ELF2TBF ?= cargo run --manifest-path $(abspath $(TOCK_USERLAND_BASE_DIR))/tools/elf2tbf/Cargo.toml --
ELF2TBF_ARGS += -n $(PACKAGE_NAME)

# Collect all desired built output.
OBJS += $(patsubst %.c,$(BUILDDIR)/%.o,$(C_SRCS))
OBJS += $(patsubst %.cc,$(BUILDDIR)/%.o,$(filter %.cc, $(CXX_SRCS)))
OBJS += $(patsubst %.cpp,$(BUILDDIR)/%.o,$(filter %.cpp, $(CXX_SRCS)))

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

##################################################################################################
# Extra warning flags not enabled by Wall or Wextra.
#
# I read through the gcc manual and grabbed the ones that I thought might be
# interesting / useful. Then I grabbed that snippet below to find other things
# that were left out of the manual that may be worth adding. Below are all
# warnings and a short description supported by (arm-none-eabi)-gcc as of
# v6.2.1

# http://stackoverflow.com/questions/11714827/
# List all supported warnings and their status:
#   gcc -Wall -Wextra -Q --help=warning
# Below are all warnings produced in an un-merged set of sorted lists
# broken into C/C++, C only, C++ only, other languages

# TODO(Pat) libnrfserialization noise with these, but I think they're useful
# and I want them back when I get a chance to clean that up.
#CPPFLAGS += -Wcast-qual #                # const char* -> char*
#CPPFLAGS += -Wswitch-default #           # switch w/out default (doesn't cover all cases) (maybe annoying?)
#CFLAGS += -Wstrict-prototypes #          # function defined w/out specifying argument types

CPPFLAGS += -Wdate-time #                # warn if __TIME__, __DATE__, or __TIMESTAMP__ used
                                         # ^on b/c flashing assumes same code => no flash, these enforce
CPPFLAGS += -Wfloat-equal #              # floats used with '=' operator, likely imprecise
CPPFLAGS += -Wformat-nonliteral #        # can't check format string (maybe disable if annoying)
CPPFLAGS += -Wformat-security #          # using untrusted format strings (maybe disable)
CPPFLAGS += -Wformat-y2k #               # use of strftime that assumes two digit years
CPPFLAGS += -Winit-self #                # { int i = i }
CPPFLAGS += -Wlogical-op #               # "suspicous use of logical operators in expressions" (a lint)
CPPFLAGS += -Wmissing-declarations #     # ^same? not sure how these differ
CPPFLAGS += -Wmissing-field-initializers # if init'ing struct w/out field names, warn if not all used
CPPFLAGS += -Wmissing-format-attribute # # something looks printf-like but isn't marked as such
CPPFLAGS += -Wmissing-noreturn #         # __attribute__((noreturn)) like -> ! in Rust, should use it
CPPFLAGS += -Wmultichar #                # use of 'foo' instead of "foo" (surpised not on by default?)
CPPFLAGS += -Wpointer-arith #            # sizeof things not define'd (i.e. sizeof(void))
CPPFLAGS += -Wredundant-decls #          # { int i; int i; } (a lint)
CPPFLAGS += -Wshadow #                   # int foo(int a) { int a = 1; } inner a shadows outer a
CPPFLAGS += -Wsuggest-attribute=const    # does what it sounds like
CPPFLAGS += -Wsuggest-attribute=pure     # does what it sounds like
CPPFLAGS += -Wtrampolines #              # attempt to generate a trampoline on the NX stack
CPPFLAGS += -Wunused-macros #            # macro defined in this file not used
CPPFLAGS += -Wunused-parameter #         # function parameter is unused aside from its declaration
CXXFLAGS += -Wuseless-cast #             # pretty much what ya think here
CPPFLAGS += -Wwrite-strings #            # { char* c = "foo"; c[0] = 'b' } <-- "foo" should be r/o

#CPPFLAGS += -Wabi -Wabi-tag              # inter-compiler abi issues
#CPPFLAGS += -Waggregate-return           # warn if things return struct's
#CPPFLAGS += -Wcast-align                 # { char *c; int *i = (int*) c}, 1 byte -> 4 byte align
#CPPFLAGS += -Wconversion                 # implicit conversion that may unexpectedly alter value
#                                         ^ A ton of these from syscalls I think, XXX look later
#CPPFLAGS += -Wdisabled-optimization      # gcc skipped an optimization for any of a thousand reasons
#CPPFLAGS += -Wdouble-promotion           # warn if float -> double implicitly XXX maybe?
#CPPFLAGS += -Wformat-signedness #        # { int i; printf("%d %u", i, i) } second bad (maybe annoying?)
#                                         ^ Too obnoxious when you want hex of an int
#CPPFLAGS += -Wfloat-conversion           # subset of -Wconversion
#CPPFLAGS += -Winline                     # something marked `inline` wasn't inlined
#CPPFLAGS += -Winvalid-pch                # bad precompiled header found in an include dir
#CPPFLAGS += -Wmissing-include-dirs -- XXX Didn't try, afriad could be annoying
#CPPFLAGS += -Woverlength-strings         # complier compat: strings > [509 C90, 4095 C99] chars
#CPPFLAGS += -Wpacked                     # struct with __attribute__((packed)) that does nothing
#CPPFLAGS += -Wpadded                     # padding added to a struct. Noisy for argument structs
#CPPFLAGS += -Wpedantic                   # strict ISO C/C++
#CPPFLAGS += -Wsign-conversion            # implicit integer sign conversions, part of -Wconversion
#CPPFLAGS += -Wstack-protector            # only if -fstack-protector, on by default, warn fn not protect
#CPPFLAGS += -Wswitch-enum #              # switch of enum doesn't explicitly cover all cases
#                                         ^ annoying in practice, let default: do its job
#CPPFLAGS += -Wsystem-headers             # warnings from system headers
#CPPFLAGS += -Wtraditional                # stuff gcc allows that "traditional" C doesn't
#CPPFLAGS += -Wundef                      # undefined identifier is evaluated in an `#if' directive
#                                         ^ Lots of library #if SAMD || SMAR21 stuff
#                                           Should probably be ifdef, but too much noise
#CPPFLAGS += -Wunsafe-loop-optimizations  # compiler can't divine loop bounds XXX maybe interesting?
#CPPFLAGS += -Wvariadic-macros            # can't be used in ISO C
#CPPFLAGS += -Wvector-operation-performance # perf option not appropriate for these systems
#CPPFLAGS += -Wvla                  -- XXX Didn't try, but interested

# C-only warnings
CFLAGS += -Wbad-function-cast #          # not obvious when this would trigger, could drop if annoying
CFLAGS += -Wjump-misses-init #           # goto or switch skips over a variable initialziation
CFLAGS += -Wmissing-prototypes #         # global fn defined w/out prototype (should be static or in .h)
CFLAGS += -Wnested-externs #             # mis/weird-use of extern keyword
CFLAGS += -Wold-style-definition #       # this garbage: void bar (a) int a; { }

#CFLAGS += -Wunsuffixed-float-constants # # { float f=0.67; if(f==0.67) printf("y"); else printf("n"); } => n
#                                         ^ doesn't seem to work right? find_north does funny stuff

#CFLAGS += -Wtraditional-conversion #     # prototype causes a conversion different than w/o prototype (?)
#                                         ^ real noisy

# CXX-only warnings
CXXFLAGS += -Wctor-dtor-privacy #        # unusable class b/c everything private and no friends
CXXFLAGS += -Wdelete-non-virtual-dtor #  # catches undefined behavior
CXXFLAGS += -Wold-style-cast #           # C-style cast in C++ code
CXXFLAGS += -Woverloaded-virtual #       # subclass shadowing makes parent impl's unavailable
CXXFLAGS += -Wsign-promo #               # gcc did what spec requires, but probably not what you want
CXXFLAGS += -Wstrict-null-sentinel #     # seems like a not-very-C++ thing to do? very unsure
CXXFLAGS += -Wsuggest-final-methods #    # does what it sounds like
CXXFLAGS += -Wsuggest-final-types #      # does what it sounds like
CXXFLAGS += -Wsuggest-override #         # overridden virtual func w/out override keyword
CXXFLAGS += -Wzero-as-null-pointer-constant # use of 0 as NULL

# -Wc++-compat #                         # C/C++ compat issues
# -Wc++11-compat #                       # C11 compat issues
# -Wc++14-compat #                       # C14 compat issues
# -Wconditionally-supported #            # conditionally-supported (C++11 [intro.defs]) constructs (?)
# -Weffc++                               # violations of style guidelines from Meyers' Effective C++ books
# -Wmultiple-inheritance                 # used to enforce coding conventions, does what you'd think
# -Wnamespaces                           # used to enforce coding conventions, warn if namespace opened
# -Wnoexcept #                           # (?) I think warns if missing noexcept
# -Wnon-virtual-dtor #                   # something deeply c++, part of effc++
# -Wsynth                                # legacy flag, g++ != cfront
# -Wtemplates                            # used to enforce coding conventions, warn if new template
# -Wvirtual-inheritance                  # used to enforce coding conventions, does what you'd think

# Fortran-only warnings
# -Waliasing
# -Wampersand
# -Warray-temporaries
# -Wc-binding-type
# -Wcharacter-truncation
# -Wcompare-reals
# -Wconversion-extra
# -Wfunction-elimination
# -Wimplicit-interface
# -Wimplicit-procedure
# -Winteger-division
# -Wintrinsic-shadow
# -Wintrinsics-std
# -Wreal-q-constant
# -Wrealloc-lhs
# -Wrealloc-lhs-all
# -Wsurprising
# -Wtabs
# -Wtarget-lifetime
# -Wunused-dummy-argument
# -Wuse-without-only

# Objective-C(++)-only
# -Wassign-intercept
# -Wselector
# -Wstrict-selector-match
# -Wundeclared-selector

# END WARNINGS
##################################################################################################

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

$(BUILDDIR)/%.o: %.cpp | $(BUILDDIR)
	$(TRACE_DEP)
	$(Q)$(CXX) $(CXXFLAGS) $(CPPFLAGS) -MF"$(@:.o=.d)" -MG -MM -MP -MT"$(@:.o=.d)@" -MT"$@" "$<"
	$(TRACE_CXX)
	$(Q)$(CXX) $(CXXFLAGS) $(CPPFLAGS) -c -o $@ $<


# As different boards have different RAM/ROM sizes, we dynamically generate a
# linker script (unless the user provide their own)
LAYOUT ?= $(BOARD_BUILDDIR)/layout.ld

# XXX(Pat) out of tree path to TOCK_BOARD directory? (?= chip hack)
USERLAND_LAYOUT := $(TOCK_USERLAND_BASE_DIR)/userland_layout.ld
CHIP_LAYOUT ?= $(TOCK_BASE_DIR)/boards/$(TOCK_BOARD)/chip_layout.ld

$(BOARD_BUILDDIR)/layout.ld:	$(USERLAND_LAYOUT) $(CHIP_LAYOUT) | $(BOARD_BUILDDIR)
	$(Q)echo "INCLUDE $(CHIP_LAYOUT)" > $@
	$(Q)echo "INCLUDE $(USERLAND_LAYOUT)" >> $@


.PHONY:	all
all:	$(BOARD_BUILDDIR)/app.bin size

.PHONY: size
size:	$(BOARD_BUILDDIR)/app.elf
	@$(SIZE) $<

.PHONY: debug
debug:	$(BOARD_BUILDDIR)/app.lst

$(BOARD_BUILDDIR)/app.lst: $(BOARD_BUILDDIR)/app.elf
	$(TRACE_LST)
	$(Q)$(OBJDUMP) $(OBJDUMP_FLAGS) $< > $(BOARD_BUILDDIR)/app.lst

# Include the libtock makefile. Adds rules that will rebuild library when needed
include $(TOCK_USERLAND_BASE_DIR)/libtock/Makefile

$(BOARD_BUILDDIR)/app.elf: $(OBJS) $(TOCK_USERLAND_BASE_DIR)/newlib/libc.a $(LIBTOCK) $(LAYOUT) | $(BOARD_BUILDDIR)
	$(TRACE_LD)
	$(Q)$(CC) $(CFLAGS) $(CPPFLAGS)\
	    -Wl,--warn-common\
	    -Wl,--gc-sections -Wl,--emit-relocs\
	    --entry=_start\
	    -Xlinker --defsym=STACK_SIZE=$(STACK_SIZE)\
	    -Xlinker --defsym=APP_HEAP_SIZE=$(APP_HEAP_SIZE)\
	    -Xlinker --defsym=KERNEL_HEAP_SIZE=$(KERNEL_HEAP_SIZE)\
	    -T $(LAYOUT)\
	    -nostdlib\
	    -Wl,--start-group $(OBJS) $(LIBS) -Wl,--end-group\
	    -Wl,-Map=$(BOARD_BUILDDIR)/app.Map\
	    -o $@

$(BOARD_BUILDDIR)/app.bin: $(BOARD_BUILDDIR)/app.elf | $(BOARD_BUILDDIR) validate_gcc_flags
	$(TRACE_BIN)
	$(Q)$(ELF2TBF) $(ELF2TBF_ARGS) -o $@ $<

.PHONY: validate_gcc_flags
validate_gcc_flags: $(BOARD_BUILDDIR)/app.elf
ifndef TOCK_NO_CHECK_SWITCHES
	$(Q)$(READELF) -p .GCC.command.line $< 2>&1 | grep -q "does not exist" && { echo "Error: Missing section .GCC.command.line"; echo ""; echo "Tock requires that applications are built with"; echo "  -frecord-gcc-switches"; echo "to validate that all required flags were used"; echo ""; echo "You can skip this check by defining the make variable TOCK_NO_CHECK_SWITCHES"; exit 1; } || exit 0
	$(Q)$(READELF) -p .GCC.command.line $< | grep -q -- -msingle-pic-base && $(READELF) -p .GCC.command.line $< | grep -q -- -mpic-register=r9 && $(READELF) -p .GCC.command.line $< | grep -q -- -mno-pic-data-is-text-relative || { echo "Error: Missing required build flags."; echo ""; echo "Tock requires applications are built with"; echo "  -msingle-pic-base"; echo "  -mpic-register=r9"; echo "  -mno-pic-data-is-text-relative"; echo "But one or more of these flags are missing"; echo ""; echo "To see the flags your application was built with, run"; echo "$(READELF) -p .GCC.command.line $<"; echo ""; exit 1; }
endif

.PHONY:
clean::
	rm -Rf $(BUILDDIR)
	rm -Rf $(BOARD_BUILDDIR)



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
