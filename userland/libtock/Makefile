# Libtock makefile. Can build Libtock standalone. Also included by application
# makefiles to ensure their libtock dependency is built

# Base folder definitions
TOCK_USERLAND_BASE_DIR ?= ..
LIBTOCK_DIR := $(TOCK_USERLAND_BASE_DIR)/libtock
LIBTOCK_BUILDDIR ?= $(LIBTOCK_DIR)/build

# Build settings
include $(TOCK_USERLAND_BASE_DIR)/Configuration.mk

# Helper functions
include $(TOCK_USERLAND_BASE_DIR)/Helpers.mk

# Check that required variables are already defined
# (they should be in Configuration.mk)
$(call check_defined, TOCK_USERLAND_BASE_DIR)
$(call check_defined, TOCK_ARCHS)
$(call check_defined, AR)
$(call check_defined, CC)
$(call check_defined, RANLIB)

# Flags for building C files
CFLAGS += -I$(LIBTOCK_DIR)

# List all C and Assembly files
LIBTOCK_C_SRCS  := $(wildcard $(LIBTOCK_DIR)/*.c)
LIBTOCK_AS_SRCS := $(wildcard $(LIBTOCK_DIR)/*.s)

# Rules for building libtock
.PHONY: all
all: $(foreach arch, $(TOCK_ARCHS),$(LIBTOCK_BUILDDIR)/$(arch)/libtock.a)

.PHONY: clean
clean::
	rm -Rf $(LIBTOCK_BUILDDIR)


# Rules to generate LibTock for a given Architecture
# These will be used to create the different architecture versions of LibTock
# Argument $(1) is the Architecture (e.g. cortex-m0) to build for
#
# Note: all variables, other than $(1), used within this block must be double
# dollar-signed so that their values will be evaluated when run, not when
# generated
#
# To see the generated rules, run:
# $(info $(foreach arch,$(TOCK_ARCHS),$(call LIBTOCK_RULES,$(arch))))
define LIBTOCK_RULES

$$(LIBTOCK_BUILDDIR)/$(1):
	$$(Q)mkdir -p $$@

$$(LIBTOCK_BUILDDIR)/$(1)/%.o: $$(LIBTOCK_DIR)/%.c | $$(LIBTOCK_BUILDDIR)/$(1)
	$$(TRACE_DEP)
	$$(Q)$$(CC) $$(CFLAGS) -mcpu=$(1) $$(CPPFLAGS) -MF"$$(@:.o=.d)" -MG -MM -MP -MT"$$(@:.o=.d)@" -MT"$$@" "$$<"
	$$(TRACE_CC)
	$$(Q)$$(CC) $$(CFLAGS) -mcpu=$(1) $$(CPPFLAGS) -c -o $$@ $$<

$$(LIBTOCK_BUILDDIR)/$(1)/%.o: %.S | $$(LIBTOCK_BUILDDIR)/$(1)
	$$(TRACE_AS)
	$$(Q)$$(AS) $$(ASFLAGS) -mcpu=$(1) $$(CPPFLAGS) -c -o $$@ $$<

LIBTOCK_OBJS_$(1) += $$(patsubst $$(LIBTOCK_DIR)/%.s,$$(LIBTOCK_BUILDDIR)/$(1)/%.o,$$(LIBTOCK_AS_SRCS))
LIBTOCK_OBJS_$(1) += $$(patsubst $$(LIBTOCK_DIR)/%.c,$$(LIBTOCK_BUILDDIR)/$(1)/%.o,$$(LIBTOCK_C_SRCS))

$$(LIBTOCK_BUILDDIR)/$(1)/libtock.a: $$(LIBTOCK_OBJS_$(1)) | $$(LIBTOCK_BUILDDIR)/$(1)
	$$(TRACE_AR)
	$$(Q)$$(AR) rc $$@ $$^
	$$(Q)$$(RANLIB) $$@
endef

# actually generate the rules for each architecture
$(foreach arch,$(TOCK_ARCHS),$(eval $(call LIBTOCK_RULES,$(arch))))


# Include dependency rules for picking up header changes (by convention at bottom of makefile)
LIBTOCK_OBJS_NO_ARCHIVES += $(filter %.o,$(foreach arch, $(TOCK_ARCHS), $(LIBTOCK_OBJS_$(arch))))
-include $(LIBTOCK_OBJS_NO_ARCHIVES:.o=.d)
