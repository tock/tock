# Configuration parameters for building Tock applications
# Included by AppMakefile.mk and TockLibrary.mk

# ensure that this file is only included once
ifndef CONFIGURATION_MAKEFILE
CONFIGURATION_MAKEFILE = 1

# Remove built-in rules and variables
# n.b. no-op for make --version < 4.0
MAKEFLAGS += -r
MAKEFLAGS += -R

# Toolchain programs
TOOLCHAIN := arm-none-eabi
AR := $(TOOLCHAIN)-ar
AS := $(TOOLCHAIN)-as
CC := $(TOOLCHAIN)-gcc
CXX := $(TOOLCHAIN)-g++
OBJDUMP := $(TOOLCHAIN)-objdump
RANLIB := $(TOOLCHAIN)-ranlib
READELF := $(TOOLCHAIN)-readelf
SIZE := $(TOOLCHAIN)-size

# Set default region sizes
STACK_SIZE       ?= 2048
APP_HEAP_SIZE    ?= 1024
KERNEL_HEAP_SIZE ?= 1024

# PACKAGE_NAME is used to identify the application for IPC and for error reporting
PACKAGE_NAME ?= $(notdir $(shell pwd))

# Tock supported architectures
TOCK_ARCHS := cortex-m0 cortex-m4

# This could be replaced with an installed version of `elf2tbf`
ELF2TBF ?= cargo run --manifest-path $(abspath $(TOCK_USERLAND_BASE_DIR))/tools/elf2tbf/Cargo.toml --
ELF2TBF_ARGS += -n $(PACKAGE_NAME)

# Flags for building app Assembly, C, C++ files
# n.b. make convention is that CPPFLAGS are shared for C and C++ sources
# [CFLAGS is C only, CXXFLAGS is C++ only]
ASFLAGS += -mthumb
CFLAGS   += -std=gnu11
CPPFLAGS += \
	    -frecord-gcc-switches\
	    -g\
	    -Os\
	    -fdata-sections -ffunction-sections\
	    -fstack-usage -Wstack-usage=$(STACK_SIZE)\
	    -Wall\
	    -Wextra\
	    -Wl,--warn-common\
	    -Wl,--gc-sections\
	    -Wl,--emit-relocs\
	    -fPIC\
	    -mthumb\
	    -mfloat-abi=soft\
	    -msingle-pic-base\
	    -mpic-register=r9\
	    -mno-pic-data-is-text-relative

# Flags for creating application Object files
OBJDUMP_FLAGS += --disassemble-all --source --disassembler-options=force-thumb -C --section-headers

# Use a generic linker script that over provisions.
LAYOUT ?= $(TOCK_USERLAND_BASE_DIR)/userland_generic.ld

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

endif

