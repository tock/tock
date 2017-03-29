# userland library master makefile. Included by library makefiles

# The first target Make finds is its default. So this line needs to be first to
# specify `all` as our default rule
all:

# directory for built output
LIB_BUILDDIR ?= build

# Build settings
include $(TOCK_USERLAND_BASE_DIR)/Configuration.mk

# Helper functions
include $(TOCK_USERLAND_BASE_DIR)/Helpers.mk


# Rules to generate libraries for a given Architecture
# These will be used to create the different architecture versions of LibNRFSerialization
# Argument $(1) is the Architecture (e.g. cortex-m0) to build for
define LIB_RULES

$$(LIB_BUILDDIR)/$(1):
	$$(Q)mkdir -p $$@

$$(LIB_BUILDDIR)/$(1)/%.o: %.c | $$(LIB_BUILDDIR)/$(1)
	$$(TRACE_DEP)
	$$(Q)$$(CC) $$(CFLAGS) -mcpu=$(1) $$(CPPFLAGS) -MF"$$(@:.o=.d)" -MG -MM -MP -MT"$$(@:.o=.d)@" -MT"$$@" "$$<"
	$$(TRACE_CC)
	$$(Q)$$(CC) $$(CFLAGS) -mcpu=$(1) $$(CPPFLAGS) -c -o $$@ $$<

$$(LIB_BUILDDIR)/$(1)/%.o: %.S | $$(LIB_BUILDDIR)/$(1)
	$$(TRACE_AS)
	$$(Q)$$(AS) $$(ASFLAGS) -mcpu=$(1) $$(CPPFLAGS) -c -o $$@ $$<

LIB_OBJS_$(1) += $$(patsubst %.c,$$(LIB_BUILDDIR)/$(1)/%.o,$$(C_SRCS))
LIB_OBJS_$(1) += $$(patsubst %.cc,$$(LIB_BUILDDIR)/$(1)/%.o,$$(filter %.cc, $$(CXX_SRCS)))
LIB_OBJS_$(1) += $$(patsubst %.cpp,$$(LIB_BUILDDIR)/$(1)/%.o,$$(filter %.cpp, $$(CXX_SRCS)))

LIB_OBJS_$(1) += $$(patsubst %.c,$$(LIB_BUILDDIR)/$(1)/%.o,$$(filter %.c, $$(LIB_SRCS)))
LIB_OBJS_$(1) += $$(patsubst %.cc,$$(LIB_BUILDDIR)/$(1)/%.o,$$(filter %.cc, $$(LIB_SRCS)))
LIB_OBJS_$(1) += $$(patsubst %.cpp,$$(LIB_BUILDDIR)/$(1)/%.o,$$(filter %.cpp, $$(LIB_SRCS)))

$$(LIB_BUILDDIR)/$(1)/$$(PACKAGE_NAME).a: $$(LIB_OBJS_$(1)) | $$(LIB_BUILDDIR)/$(1)
	$$(TRACE_AR)
	$$(Q)$$(AR) rc $$@ $$^
	$$(Q)$$(RANLIB) $$@
endef

# uncomment to print generated rules
# $(info $(foreach arch,$(TOCK_ARCHS), $(call LIB_RULES,$(arch))))
# actually generate the rules for each architecture
$(foreach arch,$(TOCK_ARCHS),$(eval $(call LIB_RULES,$(arch))))

# add each architecture as a target
all: $(foreach arch, $(TOCK_ARCHS),$(LIB_BUILDDIR)/$(arch)/$(PACKAGE_NAME).a)
