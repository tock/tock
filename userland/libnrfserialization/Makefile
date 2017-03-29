TOCK_USERLAND_BASE_DIR ?= ..
LIBNRFSERIALIZATION_DIR ?= .
LIBNRFSERIALIZATION_BUILDDIR ?= $(LIBNRFSERIALIZATION_DIR)/build

include NRFMakefile.mk

# Build settings
include $(TOCK_USERLAND_BASE_DIR)/Configuration.mk

# Helper functions
include $(TOCK_USERLAND_BASE_DIR)/Helpers.mk

LIBNRFSERIALIZATION_SRCS += $(SYSTEM_FILE) $(notdir $(APPLICATION_SRCS))

# Top-level rules
.PHONY: all
all: $(foreach arch, $(TOCK_ARCHS),$(LIBNRFSERIALIZATION_BUILDDIR)/$(arch)/libnrfserialization.a)
all: headers.tar.gz

.PHONY: clean
clean::
	rm -Rf $(LIBNRFSERIALIZATION_BUILDDIR)
	rm -Rf headers/
	rm -Rf headers.tar.gz



##############################################################################################
## Rules to actually build libnrfserialization

# Rules to generate LibNRFSerialization for a given Architecture
# These will be used to create the different architecture versions of LibNRFSerialization
# Argument $(1) is the Architecture (e.g. cortex-m0) to build for
#
# Note: all variables, other than $(1), used within this block must be double
# dollar-signed so that their values will be evaluated when run, not when
# generated
#
# To see the generated rules, run:
define LIBNRFSERIALIZATION_RULES

$$(LIBNRFSERIALIZATION_BUILDDIR)/$(1):
	$$(Q)mkdir -p $$@

$$(LIBNRFSERIALIZATION_BUILDDIR)/$(1)/%.o: $$(LIBNRFSERIALIZATION_DIR)/%.c | $$(LIBNRFSERIALIZATION_BUILDDIR)/$(1)
	$$(TRACE_DEP)
	$$(Q)$$(CC) $$(CFLAGS) -mcpu=$(1) $$(CPPFLAGS) -MF"$$(@:.o=.d)" -MG -MM -MP -MT"$$(@:.o=.d)@" -MT"$$@" "$$<"
	$$(TRACE_CC)
	$$(Q)$$(CC) $$(CFLAGS) -mcpu=$(1) $$(CPPFLAGS) -c -o $$@ $$<

$$(LIBNRFSERIALIZATION_BUILDDIR)/$(1)/%.o: %.S | $$(LIBNRFSERIALIZATION_BUILDDIR)/$(1)
	$$(TRACE_AS)
	$$(Q)$$(AS) $$(ASFLAGS) -mcpu=$(1) $$(CPPFLAGS) -c -o $$@ $$<

LIBNRFSERIALIZATION_OBJS_$(1) += $(patsubst %.c,$(LIBNRFSERIALIZATION_BUILDDIR)/$(1)/%.o,$(LIBNRFSERIALIZATION_SRCS))

$$(LIBNRFSERIALIZATION_BUILDDIR)/$(1)/libnrfserialization.a: $$(LIBNRFSERIALIZATION_OBJS_$(1)) | $$(LIBNRFSERIALIZATION_BUILDDIR)/$(1)
	$$(TRACE_AR)
	$$(Q)$$(AR) rc $$@ $$^
	$$(Q)$$(RANLIB) $$@
endef

# actually generate the rules for each architecture
$(foreach arch,$(TOCK_ARCHS),$(eval $(call LIBNRFSERIALIZATION_RULES,$(arch))))



##############################################################################################
## Rules to create libnrfserialization headers

$(LIBNRFSERIALIZATION_BUILDDIR)/headers:
	$(Q)mkdir -p $@

HDRS += $(patsubst %.c,$(LIBNRFSERIALIZATION_BUILDDIR)/headers/%.headers, $(LIBNRFSERIALIZATION_SRCS))
HDRS += $(addprefix $(LIBNRFSERIALIZATION_BUILDDIR)/headers/,$(APPLICATION_LIBS))

$(LIBNRFSERIALIZATION_BUILDDIR)/headers/%.headers: %.c | $(LIBNRFSERIALIZATION_BUILDDIR)/headers
	$(TRACE_CCM)
	$(Q)$(CC) $(CFLAGS) $(CPPFLAGS) -MM $< > $@

headers.tar.gz: $(HDRS)
	mkdir -p headers
	cat build/headers/*.headers | awk '{$$1=$$1};1' | awk '{print $$1}' | sort | grep '\.h' | grep -v libtock | uniq | xargs -IFOO cp FOO headers/
	tar czf headers.tar.gz headers

