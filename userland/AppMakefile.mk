# userland master makefile. Included by application makefiles

# The first target Make finds is its default. So this line needs to be first to
# specify `all` as our default rule
all:

# directory for built output
BUILDDIR ?= build

# Build settings
include $(TOCK_USERLAND_BASE_DIR)/Configuration.mk

# Helper functions
include $(TOCK_USERLAND_BASE_DIR)/Helpers.mk

# Include the libtock makefile. Adds rules that will rebuild library when needed
include $(TOCK_USERLAND_BASE_DIR)/libtock/Makefile

# Connection to the Tock kernel. Apps need the ability to be loaded onto a
# board, and that method is board-specific. So for now, we have the TOCK_BOARD
# variable which selects one and we include the appropriate Makefile-app from
# within the Tock base directory.
TOCK_BOARD ?= hail

# Include the makefile that has the programming functions for each board.
include $(TOCK_USERLAND_BASE_DIR)/Program.mk

# Single-arch libraries, to be phased out
LEGACY_LIBS += $(TOCK_USERLAND_BASE_DIR)/newlib/libc.a
LEGACY_LIBS += $(TOCK_USERLAND_BASE_DIR)/newlib/libm.a
LEGACY_LIBS += $(TOCK_USERLAND_BASE_DIR)/libc++/libstdc++.a
LEGACY_LIBS += $(TOCK_USERLAND_BASE_DIR)/libc++/libsupc++.a
LEGACY_LIBS += $(TOCK_USERLAND_BASE_DIR)/libc++/libgcc.a



# Rules to incorporate external libraries
define EXTERN_LIB_RULES
EXTERN_LIB_NAME_$(notdir $(1)) := $(notdir $(1))

# If this library has any additional rules, add them
-include $(1)/Makefile.app

# If this library has an include directory, add it to search path
ifneq "$$(wildcard $(1)/include)" ""
  override CPPFLAGS += -I$(1)/include
endif

# Add arch-specific rules for each library
$$(foreach arch, $$(TOCK_ARCHS), $$(eval LIBS_$$(arch) += $(1)/build/$$(arch)/$(notdir $(1)).a))

endef

# To see the generated rules, run:
# $(info $(foreach lib, $(EXTERN_LIBS), $(call EXTERN_LIB_RULES,$(lib))))
$(foreach lib, $(EXTERN_LIBS), $(eval $(call EXTERN_LIB_RULES,$(lib))))


# Some sanity checks for variables before they are used
ifdef LDFLAGS
  $(warning *******************************************************)
  $(warning LDFLAGS are currently ignored!!)
  $(warning )
  $(warning This is because we need to invoke the gcc frontend not the)
  $(warning ld frontend for the final link step, which means that we would)
  $(warning need to parse the LDFLAGS into things like -Wl,-<flag> for each)
  $(warning entry, but that proved a little fragile on first attempt so)
  $(warning it is not currently done. Sorry.)
  $(warning *******************************************************)
endif

# Warn users about improperly defined HEAP_SIZE
ifdef HEAP_SIZE
    $(warning The variable HEAP_SIZE is set but will not be used.)
    $(warning Tock has two heaps, the application heap which is memory your program)
    $(warning uses and the kernel heap or grant regions, which is memory dynamically)
    $(warning allocated by drivers on behalf of your program.)
    $(warning )
    $(warning These regions are controlled by the APP_HEAP_SIZE and KERNEL_HEAP_SIZE)
    $(warning variables respectively.)
endif



# Rules to generate an app for a given architecture
# These will be used to create the different architecture versions of an app
# Argument $(1) is the Architecture (e.g. cortex-m0) to build for
#
# Note: all variables, other than $(1), used within this block must be double
# dollar-signed so that their values will be evaluated when run, not when
# generated
define BUILD_RULES

# BUILDDIR holds architecture dependent, but board-independent outputs
$$(BUILDDIR)/$(1):
	$$(TRACE_DIR)
	$$(Q)mkdir -p $$@

# First step doesn't actually compile, just generate header dependency information
# More info on our approach here: http://stackoverflow.com/questions/97338
$$(BUILDDIR)/$(1)/%.o: %.c | $$(BUILDDIR)/$(1)
	$$(TRACE_CC)
	$$(Q)$$(CC) $$(CFLAGS) -mcpu=$(1) $$(CPPFLAGS) -MF"$$(@:.o=.d)" -MG -MM -MP -MT"$$(@:.o=.d)@" -MT"$$@" "$$<"
	$$(Q)$$(CC) $$(CFLAGS) -mcpu=$(1) $$(CPPFLAGS) -c -o $$@ $$<

$$(BUILDDIR)/$(1)/%.o: %.cc | $$(BUILDDIR)/$(1)
	$$(TRACE_CXX)
	$$(Q)$$(CXX) $$(CXXFLAGS) -mcpu=$(1) $$(CPPFLAGS) -MF"$$(@:.o=.d)" -MG -MM -MP -MT"$$(@:.o=.d)@" -MT"$$@" "$$<"
	$$(Q)$$(CXX) $$(CXXFLAGS) -mcpu=$(1) $$(CPPFLAGS) -c -o $$@ $$<

$$(BUILDDIR)/$(1)/%.o: %.cpp | $$(BUILDDIR)/$(1)
	$$(TRACE_CXX)
	$$(Q)$$(CXX) $$(CXXFLAGS) -mcpu=$(1) $$(CPPFLAGS) -MF"$$(@:.o=.d)" -MG -MM -MP -MT"$$(@:.o=.d)@" -MT"$$@" "$$<"
	$$(Q)$$(CXX) $$(CXXFLAGS) -mcpu=$(1) $$(CPPFLAGS) -c -o $$@ $$<

$$(BUILDDIR)/$(1)/%.o: %.cxx | $$(BUILDDIR)/$(1)
	$$(TRACE_CXX)
	$$(Q)$$(CXX) $$(CXXFLAGS) -mcpu=$(1) $$(CPPFLAGS) -MF"$$(@:.o=.d)" -MG -MM -MP -MT"$$(@:.o=.d)@" -MT"$$@" "$$<"
	$$(Q)$$(CXX) $$(CXXFLAGS) -mcpu=$(1) $$(CPPFLAGS) -c -o $$@ $$<

OBJS_$(1) += $$(patsubst %.c,$$(BUILDDIR)/$(1)/%.o,$$(C_SRCS))
OBJS_$(1) += $$(patsubst %.cc,$$(BUILDDIR)/$(1)/%.o,$$(filter %.cc, $$(CXX_SRCS)))
OBJS_$(1) += $$(patsubst %.cpp,$$(BUILDDIR)/$(1)/%.o,$$(filter %.cpp, $$(CXX_SRCS)))
OBJS_$(1) += $$(patsubst %.cxx,$$(BUILDDIR)/$(1)/%.o,$$(filter %.cxx, $$(CXX_SRCS)))

# Collect all desired built output.
$$(BUILDDIR)/$(1)/$(1).elf: $$(OBJS_$(1)) $$(TOCK_USERLAND_BASE_DIR)/newlib/libc.a $$(LIBS_$(1)) $$(LAYOUT) | $$(BUILDDIR)/$(1)
	$$(TRACE_LD)
	$$(Q)$$(CC) $$(CFLAGS) -mcpu=$(1) $$(CPPFLAGS)\
	    --entry=_start\
	    -Xlinker --defsym=STACK_SIZE=$$(STACK_SIZE)\
	    -Xlinker --defsym=APP_HEAP_SIZE=$$(APP_HEAP_SIZE)\
	    -Xlinker --defsym=KERNEL_HEAP_SIZE=$$(KERNEL_HEAP_SIZE)\
	    -T $$(LAYOUT)\
	    -nostdlib\
	    -Wl,--start-group $$(OBJS_$(1)) $$(LIBS_$(1)) $$(LEGACY_LIBS) -Wl,--end-group\
	    -Wl,-Map=$$(BUILDDIR)/$(1)/$(1).Map\
	    -o $$@

# NOTE: This rule creates an lst file for the elf as flashed on the board
#       (i.e. at address 0x80000000). This is not likely what you want.
$$(BUILDDIR)/$(1)/$(1).lst: $$(BUILDDIR)/$(1)/$(1).elf
	$$(TRACE_LST)
	$$(Q)$$(OBJDUMP) $$(OBJDUMP_FLAGS) $$< > $$@

# checks compiled ELF files to ensure that all libraries and applications were
# built with the correct flags in order to work on a Tock board
.PHONY: validate_gcc_flags
validate_gcc_flags:: $$(BUILDDIR)/$(1)/$(1).elf
ifndef TOCK_NO_CHECK_SWITCHES
	$$(Q)$$(READELF) -p .GCC.command.line $$< 2>&1 | grep -q "does not exist" && { echo "Error: Missing section .GCC.command.line"; echo ""; echo "Tock requires that applications are built with"; echo "  -frecord-gcc-switches"; echo "to validate that all required flags were used"; echo ""; echo "You can skip this check by defining the make variable TOCK_NO_CHECK_SWITCHES"; exit 1; } || exit 0
	$$(Q)$$(READELF) -p .GCC.command.line $$< | grep -q -- -msingle-pic-base && $$(READELF) -p .GCC.command.line $$< | grep -q -- -mpic-register=r9 && $$(READELF) -p .GCC.command.line $$< | grep -q -- -mno-pic-data-is-text-relative || { echo "Error: Missing required build flags."; echo ""; echo "Tock requires applications are built with"; echo "  -msingle-pic-base"; echo "  -mpic-register=r9"; echo "  -mno-pic-data-is-text-relative"; echo "But one or more of these flags are missing"; echo ""; echo "To see the flags your application was built with, run"; echo "$$(READELF) -p .GCC.command.line $$<"; echo ""; exit 1; }
endif



############################################################################################
# DEBUGGING STUFF
#
# The approach here is that we're going create a new elf file that is compiled
# at the actual flash and ram offset of the loaded program
#
# We want to build a rule that fails if these needed env variables aren't set
# only when actually trying to use them to build the lst file. We also want to
# force this to rerun every time it's invoked so that it picks up new env
# variable settings


# Step 0: Force this to be built every time
.PHONY: _FORCE_USERLAND_DEBUG_LD

# Step 1: Create a new linker script. Note this depends on original (non-shifted) elf
# (supposedly this could be one-lined, but I couldn't make that work, so here goes)
ifdef RAM_START
  ifdef FLASH_INIT
    _USERLAND_DEBUG_ALL_NEEDED_VARS := 1
  endif
endif

$$(BUILDDIR)/$(1)/$(1).userland_debug.ld: $$(TOCK_USERLAND_BASE_DIR)/userland_generic.ld $$(BUILDDIR)/$(1)/$(1).elf _FORCE_USERLAND_DEBUG_LD
ifndef _USERLAND_DEBUG_ALL_NEEDED_VARS
	@echo "ERROR: Required variables RAM_START and FLASH_INIT are not set."
	@echo "       These are needed to compute the offset your program was loaded at."
	@echo "       See the kernel panic message for these values."
	@exit 1
else
	@# Start with a copy of the template / generic ld script
	$$(Q)cp $$< $$@
	@# And with apologies to future readers, this is easier as one shell command/script so
	@# we can set intervening variables, away we go
	@#
	@# Get the offset between the init function and the start of text (0x80000000).
	@# We then use that offset to calculate where the start of text was on the actual MCU.
	@# Create a new LD file at the correct flash and ram locations.
	@#
	@# #616 #635: sed is not cross-platform
	@# https://stackoverflow.com/a/22247781/358675 <-- Use perl in place of sed
	$$(Q)set -e ;\
	  ORIGINAL_ENTRY=`$$(READELF) -h $$(BUILDDIR)/$(1)/$(1).elf | grep Entry | awk '{print $$$$4}'` ;\
	  INIT_OFFSET=$$$$(($$$$ORIGINAL_ENTRY - 0x80000000)) ;\
	  FLASH_START=$$$$(($$$$FLASH_INIT-$$$$INIT_OFFSET)) ;\
	  perl -pi -e "s/(FLASH.*ORIGIN[ =]*)([x0-9]*)(,.*LENGTH)/\$$$${1}$$$$FLASH_START\$$$$3/" $$@ ;\
	  perl -pi -e "s/(SRAM.*ORIGIN[ =]*)([x0-9]*)(,.*LENGTH)/\$$$${1}$$$$RAM_START\$$$$3/" $$@
endif

# Step 2: Create a new ELF with the layout that matches what's loaded
$$(BUILDDIR)/$(1)/$(1).userland_debug.elf: $$(OBJS_$(1)) $$(TOCK_USERLAND_BASE_DIR)/newlib/libc.a $$(LIBS_$(1)) $$(BUILDDIR)/$(1)/$(1).userland_debug.ld | $$(BUILDDIR)/$(1)
	$$(TRACE_LD)
	$$(Q)$$(CC) $$(CFLAGS) -mcpu=$(1) $$(CPPFLAGS)\
	    --entry=_start\
	    -Xlinker --defsym=STACK_SIZE=$$(STACK_SIZE)\
	    -Xlinker --defsym=APP_HEAP_SIZE=$$(APP_HEAP_SIZE)\
	    -Xlinker --defsym=KERNEL_HEAP_SIZE=$$(KERNEL_HEAP_SIZE)\
	    -T $$(BUILDDIR)/$(1)/$(1).userland_debug.ld\
	    -nostdlib\
	    -Wl,--start-group $$(OBJS_$(1)) $$(LIBS_$(1)) $$(LEGACY_LIBS) -Wl,--end-group\
	    -Wl,-Map=$$(BUILDDIR)/$(1)/$(1).Map\
	    -o $$@

# Step 3: Now we can finally generate an LST
$$(BUILDDIR)/$(1)/$(1).userland_debug.lst: $$(BUILDDIR)/$(1)/$(1).userland_debug.elf
	$$(TRACE_LST)
	$$(Q)$$(OBJDUMP) $$(OBJDUMP_FLAGS) $$< > $$@
	@echo $$$$(tput bold)Listings generated at $$@$$$$(tput sgr0)

# END DEBUGGING STUFF
############################################################################################
endef

# To see the generated rules, run:
# $(info $(foreach arch,$(TOCK_ARCHS),$(call BUILD_RULES,$(arch))))
# Actually generate the rules for each architecture
$(foreach arch, $(TOCK_ARCHS), $(eval $(call BUILD_RULES,$(arch))))



# TAB file generation. Used for Tockloader
$(BUILDDIR)/$(PACKAGE_NAME).tab: $(foreach arch, $(TOCK_ARCHS), $(BUILDDIR)/$(arch)/$(arch).elf)
	$(Q)$(ELF2TAB) $(ELF2TAB_ARGS) -o $@ $^



# Rules for building apps
.PHONY:	all
all:	$(BUILDDIR)/$(PACKAGE_NAME).tab size

.PHONY: size
size:	$(foreach arch, $(TOCK_ARCHS), $(BUILDDIR)/$(arch)/$(arch).elf)
	@$(SIZE) $^

.PHONY: debug
debug:	$(foreach arch, $(TOCK_ARCHS), $(BUILDDIR)/$(arch)/$(arch).userland_debug.lst)

.PHONY:
clean::
	rm -Rf $(BUILDDIR)


# Rules for running the C linter
FORMATTED_FILES := $(patsubst %.c,$(BUILDDIR)/format/%.uncrustify,$(C_SRCS))
FORMATTED_FILES += $(patsubst %.cc,$(BUILDDIR)/format/%.uncrustify,$(filter %.cc, $(CXX_SRCS)))
FORMATTED_FILES += $(patsubst %.cpp,$(BUILDDIR)/format/%.uncrustify,$(filter %.cpp, $(CXX_SRCS)))
FORMATTED_FILES += $(patsubst %.cxx,$(BUILDDIR)/format/%.uncrustify,$(filter %.cxx, $(CXX_SRCS)))

$(BUILDDIR)/format:
	@mkdir -p $@

.PHONY: fmt format
fmt format:: $(FORMATTED_FILES)

$(BUILDDIR)/format/%.uncrustify: %.c | _format_check_unstaged
	$(Q)$(UNCRUSTIFY) -f $< -o $@
	$(Q)cmp -s $< $@ || (if [ "$$CI" = "true" ]; then diff -y $< $@; rm $@; exit 1; else cp $@ $<; fi)
$(BUILDDIR)/format/%.uncrustify: %.cc | _format_check_unstaged
	$(Q)$(UNCRUSTIFY) -f $< -o $@
	$(Q)cmp -s $< $@ || (if [ "$$CI" = "true" ]; then diff -y $< $@; rm $@; exit 1; else cp $@ $<; fi)
$(BUILDDIR)/format/%.uncrustify: %.cpp | _format_check_unstaged
	$(Q)$(UNCRUSTIFY) -f $< -o $@
	$(Q)cmp -s $< $@ || (if [ "$$CI" = "true" ]; then diff -y $< $@; rm $@; exit 1; else cp $@ $<; fi)
$(BUILDDIR)/format/%.uncrustify: %.cxx | _format_check_unstaged
	$(Q)$(UNCRUSTIFY) -f $< -o $@
	$(Q)cmp -s $< $@ || (if [ "$$CI" = "true" ]; then diff -y $< $@; rm $@; exit 1; else cp $@ $<; fi)


# Rules to help validate build configuration
fmt format::
	$(Q)$(TOCK_USERLAND_BASE_DIR)/tools/check_override.sh


#########################################################################################
# Include dependency rules for picking up header changes (by convention at bottom of makefile)
OBJS_NO_ARCHIVES += $(filter %.o,$(foreach arch, $(TOCK_ARCHS), $(OBJS_$(arch))))
-include $(OBJS_NO_ARCHIVES:.o=.d)
