APP ?= $(notdir $(CURDIR))
APP_DIR ?= $(CURDIR)
APP_LINKER_SCRIPT ?= $(CURDIR)/loader.ld

TOCK_PLATFORM ?= storm

TOCK_BUILD_DIR := $(TOCK_APPS_DIR)/../build/$(TOCK_PLATFORM)
TOCK_APP_BUILD_DIR := $(TOCK_BUILD_DIR)/$(APP)
TOCK_APP_LIBS_DIR := $(TOCK_APP_BUILD_DIR)/libs
include $(TOCK_APPS_DIR)/Makefile.$(TOCK_PLATFORM).mk

TOCK_DIR = $(TOCK_APPS_DIR)/../src

# XXX FIXME extern stuff
APP_LIBC := ../../extern/newlib/libc.a

# n.b. make convention is that CPPFLAGS are shared for C and C++ sources
# [CFLAGS is C only, CXXFLAGS is C++ only]
CPPFLAGS += \
	    -I$(TOCK_APPS_DIR)/libs\
	    -fdata-sections -ffunction-sections\
	    -Wl,-gc-sections\
	    -fPIC\
	    -msingle-pic-base\
	    -mno-pic-data-is-text-relative


###############################################################################
## Tock Application Support Library

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


###############################################################################
## Rules to convert a built app to something that can be loaded into tock
##
## This process converts a built app to a single array of bytes, and then puts
## that array of bytes into an object file with a single section containing the
## app that can be later linked into the kernel.

$(TOCK_APP_BUILD_DIR)/$(APP).bin: $(TOCK_APP_BUILD_DIR)/$(APP).elf
	@echo "Extracting binary $@"
	$(OBJCOPY) --gap-fill 0xff -O binary $< $@

$(TOCK_APP_BUILD_DIR)/$(APP).monolithic.o: $(TOCK_APP_BUILD_DIR)/$(APP).bin
	@echo "Re-Linking $@"
	$(LD) -r -b binary -o $@ $<
	$(OBJCOPY) --rename-section .data=.app.$(APP) $@
	$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > $(TOCK_APP_BUILD_DIR)/$(APP).monolithic.lst

APPS_TO_LINK_TO_KERNEL=$(TOCK_APP_BUILD_DIR)/$(APP).monolithic.o

#####################################################################
## Utility Functions

# Recursive wildcard function
# http://blog.jgc.org/2011/07/gnu-make-recursive-wildcard-function.html
rwildcard=$(foreach d,$(wildcard $1*),$(call rwildcard,$d/,$2) \
  $(filter $(subst *,%,$2),$d))


#####################################################################
## Convenience rules

.PHONY: kernel
kernel:
	$(MAKE) -C $(TOCK_APPS_DIR)/..

.PHONY:	clean
clean::
	rm -rf $(TOCK_APP_BUILD_DIR)

$(TOCK_APP_BUILD_DIR):
	mkdir -p $@

