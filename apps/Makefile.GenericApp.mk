# Include an all target at the top so that all becomes the default goal
#
# Note that this makefile only gets as far as building/requiring the application
# image that will be loaded into tock. The platform makefile provides further
# dependencies to the all target such that a unified kernel+app image is built
#
# This makefile has rules to create up to $(APP).elf, the Common rules convert
# a built application to the linkable monolithic object given as a target here
.SECONDEXPANSION:
.PHONY:	all
all:	$(TOCK_APPS_DIR)/../build/$$(TOCK_PLATFORM)/$$(APP)/$$(APP).monolithic.o


include $(TOCK_APPS_DIR)/Makefile.Common.mk


###############################################################################
## Rules to collect and build a simple collection of source files

AS_SRCS  := $(wildcard $(APP_DIR)/*.s)
C_SRCS   := $(wildcard $(APP_DIR)/*.c)
CC_SRCS  := $(wildcard $(APP_DIR)/*.cc)
CPP_SRCS := $(wildcard $(APP_DIR)/*.cpp)

LIBS := $(patsubst %.s,%.o,$(AS_SRCS))
LIBS += $(patsubst %.c,%.o,$(C_SRCS))
LIBS += $(patsubst %.cc,%.o,$(CC_SRCS))
LIBS += $(patsubst %.cpp,%.o,$(CPP_SRCS))

LIBS := $(notdir $(LIBS))
LIBS := $(foreach var,$(LIBS),$(TOCK_APP_BUILD_DIR)/$(var))


$(TOCK_APP_BUILD_DIR)/%.o:	$(APP_DIR)/%.c | $(TOCK_APP_BUILD_DIR)
	$(TRACE_CC)
	$(Q)$(CC) $(CFLAGS) $(CPPFLAGS) $< -c -o $@

$(TOCK_APP_BUILD_DIR)/%.o:	$(APP_DIR)/%.cc | $(TOCK_APP_BUILD_DIR)
	$(TRACE_CXX)
	$(Q)$(CXX) $(CXXFLAGS) $(CPPFLAGS) $< -c -o $@

$(TOCK_APP_BUILD_DIR)/%.o:	$(APP_DIR)/%.cpp | $(TOCK_APP_BUILD_DIR)
	$(TRACE_CXX)
	$(Q)$(CXX) $(CXXFLAGS) $(CPPFLAGS) $< -c -o $@

-include $(patsubst %.o,%.d,$(LIBS))


# XXX FIXME
$(TOCK_APP_BUILD_DIR)/syscalls.o:	$(TOCK_DIR)/arch/$(ARCH)/syscalls.S | $(TOCK_APP_BUILD_DIR)
	$(TRACE_AS)
	$(Q)$(AS) $(ASFLAGS) $^ -o $@

LIBS += $(TOCK_APP_BUILD_DIR)/syscalls.o




$(TOCK_APP_BUILD_DIR)/$(APP).elf: $(LIBS) $(TOCK_LIBS) $(APP_LIBC) | $(TOCK_APP_BUILD_DIR) kernel
	$(TRACE_LD)
	$(Q)$(LD) $(CFLAGS) -g -Os -T $(APP_LINKER_SCRIPT) --emit-relocs -nostdlib $^ -o $@
	$(Q)$(LD) $(CFLAGS) -g -Os -T $(APP_LINKER_SCRIPT) --emit-relocs -nostdlib $^ -o $@
	$(Q)$(GENLST) $@ > $(TOCK_APP_BUILD_DIR)/$(APP).lst

