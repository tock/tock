# userland library master makefile. Included by library makefiles

# The first target Make finds is its default. So this line needs to be first to
# specify `all` as our default rule
all:

# Build settings
include $(TOCK_USERLAND_BASE_DIR)/Configuration.mk

# Helper functions
include $(TOCK_USERLAND_BASE_DIR)/Helpers.mk

# Okay.. so here's what the goals are:
#
#  - LIBNAME should be the name of the _current_ library that's including this file
#  - We'd like to set this automatically based on the current directory
#  --- But at the same time not trample over a user-specified definition
#  - We'd like to allow multiple different libraries to include this file
#
# So, our approach is to keep track of all the SEEN_LIBNAMES, and if the
# current LIBNAME is in the list of SEEN_LIBNAMES, assume that this variable
# is simply still set from a previous inclusion of this file and overwrite it

CURRENT_DIRNAME := $(notdir $(shell pwd))
ifdef SEEN_LIBNAMES
  ifneq ($(filter $(LIBNAME),$(SEEN_LIBNAMES)),"")
    # LIBNAME in SEEN_LIBNAMES, replace
    LIBNAME := $(CURRENT_DIRNAME)
  endif
else
  ifndef LIBNAME
    LIBNAME := $(CURRENT_DIRNAME)
  endif
endif
SEEN_LIBNAMES += $(LIBNAME)

# Grab the directory this library is in
$(LIBNAME)_DIR ?= $(dir $(abspath $(lastword $(MAKEFILE_LIST))))

# directory for built output
$(LIBNAME)_BUILDDIR ?= $($(LIBNAME)_DIR)build


$(LIBNAME)_SRCS = $(AS_SRCS) $(C_SRCS) $(CXX_SRCS) $(LIB_SRCS)

# Rules to generate libraries for a given Architecture
# These will be used to create the different architecture versions of LibNRFSerialization
# Argument $(1) is the Architecture (e.g. cortex-m0) to build for
define LIB_RULES

$$($(LIBNAME)_BUILDDIR)/$(1):
	$$(Q)mkdir -p $$@

$$($(LIBNAME)_BUILDDIR)/$(1)/%.o: %.c | $$($(LIBNAME)_BUILDDIR)/$(1)
	$$(TRACE_DEP)
	$$(Q)$$(CC) $$(CFLAGS) -mcpu=$(1) $$(CPPFLAGS) -MF"$$(@:.o=.d)" -MG -MM -MP -MT"$$(@:.o=.d)@" -MT"$$@" "$$<"
	$$(TRACE_CC)
	$$(Q)$$(CC) $$(CFLAGS) -mcpu=$(1) $$(CPPFLAGS) -c -o $$@ $$<

$$($(LIBNAME)_BUILDDIR)/$(1)/%.o: %.S | $$($(LIBNAME)_BUILDDIR)/$(1)
	$$(TRACE_AS)
	$$(Q)$$(AS) $$(ASFLAGS) -mcpu=$(1) $$(CPPFLAGS) -c -o $$@ $$<

$(LIBNAME)_OBJS_$(1) += $$(patsubst %.s,$$($(LIBNAME)_BUILDDIR)/$(1)/%.o,$$(filter %.s, $$($(LIBNAME)_SRCS)))
$(LIBNAME)_OBJS_$(1) += $$(patsubst %.c,$$($(LIBNAME)_BUILDDIR)/$(1)/%.o,$$(filter %.c, $$($(LIBNAME)_SRCS)))
$(LIBNAME)_OBJS_$(1) += $$(patsubst %.cc,$$($(LIBNAME)_BUILDDIR)/$(1)/%.o,$$(filter %.cc, $$($(LIBNAME)_SRCS)))
$(LIBNAME)_OBJS_$(1) += $$(patsubst %.cpp,$$($(LIBNAME)_BUILDDIR)/$(1)/%.o,$$(filter %.cpp, $$($(LIBNAME)_SRCS)))

$$($(LIBNAME)_BUILDDIR)/$(1)/$$(LIBNAME).a: $$($(LIBNAME)_OBJS_$(1)) | $$($(LIBNAME)_BUILDDIR)/$(1)
	$$(TRACE_AR)
	$$(Q)$$(AR) rc $$@ $$^
	$$(Q)$$(RANLIB) $$@
endef

# uncomment to print generated rules
# $(info $(foreach arch,$(TOCK_ARCHS), $(call LIB_RULES,$(arch))))
# actually generate the rules for each architecture
$(foreach arch,$(TOCK_ARCHS),$(eval $(call LIB_RULES,$(arch))))

# add each architecture as a target
.PHONY: all
all: $(foreach arch, $(TOCK_ARCHS),$($(LIBNAME)_BUILDDIR)/$(arch)/$(LIBNAME).a)

.PHONY: clean
clean::
	rm -Rf $($(LIBNAME)_BUILDDIR)
