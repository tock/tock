TOCK_USERLAND_BASE_DIR ?= ..
LIBNAME=libnrfserialization
$(LIBNAME)_DIR=.

include NRFMakefile.mk

$(LIBNAME)_SRCS += $(SYSTEM_FILE) $(notdir $(APPLICATION_SRCS))


##############################################################################################
## Rules to create libnrfserialization

# Need libtock headers
CPPFLAGS += -I$(TOCK_USERLAND_BASE_DIR)/libtock

include $(TOCK_USERLAND_BASE_DIR)/TockLibrary.mk


##############################################################################################
## Rules to create libnrfserialization headers

# Require all to build this too
all: headers.tar.gz

$($(LIBNAME)_BUILDDIR)/headers:
	$(Q)mkdir -p $@

HDRS += $(patsubst %.c,$($(LIBNAME)_BUILDDIR)/headers/%.headers, $($(LIBNAME)_SRCS))
HDRS += $(addprefix $($(LIBNAME)_BUILDDIR)/headers/,$(APPLICATION_LIBS))

$($(LIBNAME)_BUILDDIR)/headers/%.headers: %.c | $($(LIBNAME)_BUILDDIR)/headers
	$(TRACE_CCM)
	$(Q)$(CC) $(CFLAGS) $(CPPFLAGS) -MM $< > $@

headers.tar.gz: $(HDRS)
	mkdir -p headers
	cat build/headers/*.headers | awk '{$$1=$$1};1' | awk '{print $$1}' | sort | grep '\.h' | grep -v libtock | uniq | xargs -IFOO cp FOO headers/
	tar czf headers.tar.gz headers

.PHONY: clean
clean::
	rm -Rf headers/
	rm -Rf headers.tar.gz
