TOCK_USERLAND_BASE_DIR ?= ..
LIBNRFSERIALIZATION_DIR ?= .
LIBNRFSERIALIZATION_BUILDDIR ?= $(LIBNRFSERIALIZATION_DIR)/build

include NRFMakefile.mk

LIB_SRCS += $(SYSTEM_FILE) $(notdir $(APPLICATION_SRCS))

# Top-level rules
.PHONY: all
all: headers.tar.gz

.PHONY: clean
clean::
	rm -Rf $(LIBNRFSERIALIZATION_BUILDDIR)
	rm -Rf headers/
	rm -Rf headers.tar.gz



##############################################################################################
## Rules to create libnrfserialization headers

$(LIBNRFSERIALIZATION_BUILDDIR)/headers:
	$(Q)mkdir -p $@

HDRS += $(patsubst %.c,$(LIBNRFSERIALIZATION_BUILDDIR)/headers/%.headers, $(LIB_SRCS))
HDRS += $(addprefix $(LIBNRFSERIALIZATION_BUILDDIR)/headers/,$(APPLICATION_LIBS))

$(LIBNRFSERIALIZATION_BUILDDIR)/headers/%.headers: %.c | $(LIBNRFSERIALIZATION_BUILDDIR)/headers
	$(TRACE_CCM)
	$(Q)$(CC) $(CFLAGS) $(CPPFLAGS) -MM $< > $@

headers.tar.gz: $(HDRS)
	mkdir -p headers
	cat build/headers/*.headers | awk '{$$1=$$1};1' | awk '{print $$1}' | sort | grep '\.h' | grep -v libtock | uniq | xargs -IFOO cp FOO headers/
	tar czf headers.tar.gz headers


##############################################################################################
## Rules to create libnrfserialization
include $(TOCK_USERLAND_BASE_DIR)/TockLibrary.mk
