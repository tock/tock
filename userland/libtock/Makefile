LIBTOCK_DIR := $(TOCK_USERLAND_BASE_DIR)/libtock

ifeq ($(LIBTOCK_DIR),/libtock)
  $(error Do not run make from the libtock directory)
endif

$(call check_defined, TOCK_USERLAND_BASE_DIR)
$(call check_defined, TOCK_ARCH)
$(call check_defined, AR)
$(call check_defined, CC)
$(call check_defined, RANLIB)

LIBTOCK_BUILDDIR ?= $(LIBTOCK_DIR)/build/$(TOCK_ARCH)

CFLAGS += -I$(LIBTOCK_DIR)

LIBTOCK_C_SRCS  := $(wildcard $(LIBTOCK_DIR)/*.c)
LIBTOCK_AS_SRCS := $(wildcard $(LIBTOCK_DIR)/*.s)
LIBTOCK_OBJS    := $(patsubst $(LIBTOCK_DIR)/%.s,$(LIBTOCK_BUILDDIR)/%.o,$(LIBTOCK_AS_SRCS))
LIBTOCK_OBJS    += $(patsubst $(LIBTOCK_DIR)/%.c,$(LIBTOCK_BUILDDIR)/%.o,$(LIBTOCK_C_SRCS))

.PHONY: all
all: $(LIBTOCK_BUILDDIR)/libtock.a

$(LIBTOCK_BUILDDIR):
	$(Q)mkdir -p $@

$(LIBTOCK_BUILDDIR)/%.o: $(LIBTOCK_DIR)/%.c | $(LIBTOCK_BUILDDIR)
	$(TRACE_DEP)
	$(Q)$(CC) $(CFLAGS) $(CPPFLAGS) -MF"$(@:.o=.d)" -MG -MM -MP -MT"$(@:.o=.d)@" -MT"$@" "$<"
	$(TRACE_CC)
	$(Q)$(CC) $(CFLAGS) $(CPPFLAGS) -c -o $@ $<

$(LIBTOCK_BUILDDIR)/%.o: %.S | $(LIBTOCK_BUILDDIR)
	$(TRACE_AS)
	$(Q)$(AS) $(ASFLAGS) $(CPPFLAGS) -c -o $@ $<

$(LIBTOCK_BUILDDIR)/libtock.a: $(LIBTOCK_OBJS) | $(LIBTOCK_BUILDDIR)
	$(TRACE_AR)
	$(Q)$(AR) rc $@ $^
	$(Q)$(RANLIB) $@

.PHONY: clean
clean::
	rm -Rf $(LIBTOCK_BUILDDIR)

# Include dependency rules for picking up header changes (by convention at bottom of makefile)
LIBTOCK_OBJS_NO_ARCHIVES=$(filter %.o,$(LIBTOCK_OBJS))
-include $(LIBTOCK_OBJS_NO_ARCHIVES:.o=.d)
