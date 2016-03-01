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
	    -MD\
	    -Wall\
	    -Wextra\
	    -Wl,-gc-sections\
	    -fPIC\
	    -msingle-pic-base\
	    -mno-pic-data-is-text-relative


###############################################################################
## Tock Application Support Library

$(TOCK_APP_LIBS_DIR):
	$(Q)mkdir -p $@

TOCK_LIBS := $(subst .c,.o,$(wildcard $(TOCK_APPS_DIR)/libs/*.c))
TOCK_LIBS += $(subst .s,.o,$(wildcard $(TOCK_APPS_DIR)/libs/*.s))

TOCK_LIBS := $(notdir $(TOCK_LIBS))
TOCK_LIBS := $(foreach var,$(TOCK_LIBS),$(TOCK_APP_LIBS_DIR)/$(var))
#$(error $(TOCK_LIBS))

$(TOCK_APP_LIBS_DIR)/%.o: $(TOCK_APPS_DIR)/libs/%.c | $(TOCK_APP_LIBS_DIR)
	$(TRACE_CC)
	$(Q)$(CC) $(CFLAGS) $(CPPFLAGS) $< -c -o $@

$(TOCK_APP_LIBS_DIR)/%.o: $(TOCK_APPS_DIR)/libs/%.s | $(TOCK_APP_LIBS_DIR)
	$(TRACE_AS)
	$(Q)$(AS) $(ASFLAGS) $< -o $@

-include $(patsubst %.o,%.d,$(TOCK_LIBS))

###############################################################################
## Rules to convert a built app to something that can be loaded into tock
##
## This process converts a built app to a single array of bytes, and then puts
## that array of bytes into an object file with a single section containing the
## app that can be later linked into the kernel.

$(TOCK_APP_BUILD_DIR)/$(APP).bin: $(TOCK_APP_BUILD_DIR)/$(APP).elf
	$(TRACE_BIN)
	@tput bold; tput setaf 3; echo Running temporary hack to check for bad relocations ; tput sgr0
	@pushd $(TOCK_APP_BUILD_DIR) &&\
		for f in $$(find . -type f -name '*.o'); do\
			if [ "$$(basename $$f)" = crt1.o ]; then continue; fi ;\
			readelf -rs $$f |\
			awk '1;/\.rel\.debug/{exit}' |\
			grep -B2 R_ARM_ABS32 && \
				tput bold && tput setaf 1 && echo ERROR: Symbol from $$f will not be PIC && exit 1;\
		done; popd
	@tput bold; tput setaf 3; echo Passed. ; tput sgr0
	$(Q)$(SIZE) $(TOCK_APP_BUILD_DIR)/$(APP).elf
	$(Q)$(OBJCOPY) --gap-fill 0xff -O binary $< $@

$(TOCK_APP_BUILD_DIR)/$(APP).monolithic.o: $(TOCK_APP_BUILD_DIR)/$(APP).bin
	$(TRACE_LD)
	$(Q)$(LD) -r -b binary -o $@ $<
	$(Q)$(OBJCOPY) --rename-section .data=.app.$(APP) $@
	$(Q)$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > $(TOCK_APP_BUILD_DIR)/$(APP).monolithic.lst

APPS_TO_LINK_TO_KERNEL=$(TOCK_APP_BUILD_DIR)/$(APP).monolithic.o

#####################################################################
## Utility Functions

# Recursive wildcard function
# http://blog.jgc.org/2011/07/gnu-make-recursive-wildcard-function.html
rwildcard=$(foreach d,$(wildcard $1*),$(call rwildcard,$d/,$2) \
  $(filter $(subst *,%,$2),$d))


#####################################################################
## Convenience rules

# If environment variable V is non-empty, be verbose
ifneq ($(V),)
Q=
TRACE_BIN =
TRACE_CC  =
TRACE_CXX =
TRACE_LD  =
TRACE_AR  =
TRACE_AS  =
TRACE_LST =
else
Q=@
TRACE_BIN = @echo " BIN       " $@
TRACE_CC  = @echo "  CC       " $<
TRACE_CXX = @echo " CXX       " $<
TRACE_LD  = @echo "  LD       " $@
TRACE_AR  = @echo "  AR       " $@
TRACE_AS  = @echo "  AS       " $<
TRACE_LST = @echo " LST       " $<
endif

.PHONY: kernel
kernel:
	@tput bold ; echo "Verifying kernel is up to date" ; tput sgr0
	$(MAKE) -C $(TOCK_APPS_DIR)/..

.PHONY:	clean
clean::
	@tput bold ; echo "Cleaning $(APP)" ; tput sgr0
	$(Q)rm -rf $(TOCK_APP_BUILD_DIR)

$(TOCK_APP_BUILD_DIR):
	$(Q)mkdir -p $@

