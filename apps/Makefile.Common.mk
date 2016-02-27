ifndef APP
$(error Required variable APP is not set)
endif

TOCK_PLATFORM ?= storm

TOCK_BUILD_DIR := $(TOCK_APPS_DIR)/../build/$(TOCK_PLATFORM)
TOCK_APP_BUILD_DIR := $(TOCK_BUILD_DIR)/$(APP)
TOCK_APP_LIBS_DIR := $(TOCK_APP_BUILD_DIR)/libs
include $(TOCK_APPS_DIR)/Makefile.$(TOCK_PLATFORM).mk

TOCK_DIR = $(TOCK_APPS_DIR)/../src

# XXX FIXME extern stuff
APP_LIBC := ../../extern/newlib/libc.a

# n.b. CPPFLAGS are shared for C and C++ sources [CFLAGS is C only, CXXFLAGS C++ only]
CPPFLAGS += -I$(TOCK_APPS_DIR)/libs -fdata-sections -ffunction-sections -Wl,-gc-sections -fPIC -msingle-pic-base -mno-pic-data-is-text-relative


$(TOCK_APP_LIBS_DIR):
	mkdir -p $@

TOCK_LIBS := $(subst .c,.o,$(wildcard $(TOCK_APPS_DIR)/libs/*.c))
TOCK_LIBS += $(subst .s,.o,$(wildcard $(TOCK_APPS_DIR)/libs/*.s))

TOCK_LIBS := $(notdir $(TOCK_LIBS))
TOCK_LIBS := $(foreach var,$(TOCK_LIBS),$(TOCK_APP_LIBS_DIR)/$(var))
#$(error $(TOCK_LIBS))

$(TOCK_APP_LIBS_DIR)/%.o: $(TOCK_APPS_DIR)/libs/%.c | $(TOCK_APP_LIBS_DIR)
	$(CC) $(CFLAGS) $(CPPFLAGS) $^ -c -o $@

$(TOCK_APP_LIBS_DIR)/%.o: $(TOCK_APPS_DIR)/libs/%.s | $(TOCK_APP_LIBS_DIR)
	$(AS) $(ASFLAGS) $^ -o $@

# Recursive wildcard function
# http://blog.jgc.org/2011/07/gnu-make-recursive-wildcard-function.html
rwildcard=$(foreach d,$(wildcard $1*),$(call rwildcard,$d/,$2) \
  $(filter $(subst *,%,$2),$d))

.PHONY: kernel
kernel:
	$(MAKE) -C $(TOCK_APPS_DIR)/..

