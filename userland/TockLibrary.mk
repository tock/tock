# userland library master makefile. Included by library makefiles

# The first target Make finds is its default. So this line needs to be first to
# specify `all` as our default rule
all:

# Build settings
include $(TOCK_USERLAND_BASE_DIR)/Configuration.mk

# Helper functions
include $(TOCK_USERLAND_BASE_DIR)/Helpers.mk

$(call check_defined, LIBNAME)
$(call check_defined, $(LIBNAME)_DIR)
$(call check_defined, $(LIBNAME)_SRCS)

# directory for built output
$(LIBNAME)_BUILDDIR ?= $($(LIBNAME)_DIR)/build

# Handle complex paths
#
# Okay, so this merits some explanation:
#
# Our build system aspires to put everything in build/ directories, this means
# that we have to match the path of source files (foo.c) to output directories
# (build/<arch>/foo.o). That's easy enough if all the source files are in the
# same directory, but restricts applications and libraries to a flat file
# structure.
#
# The current solution we employ is built on make's VPATH variable, which is a
# list of directories to search for dependencies, e.g.
#
#    VPATH = foo/ ../bar/
#    somerule: dependency.c
#
# Will find any of ./dependency.c, foo/dependency.c, or ../bar/dependency.c
# We leverage this by flattening the list of SRCS to remove all path
# information and adding all the paths from the SRCS to the VPATH, this means
# we can write rules as-if all the SRCS were in a flat directory.
#
# The obvious pitfall here is what happens when multiple directories hold a
# source file of the same name. However, both libnrf and mbed are set up to
# use VPATH without running into that problem, which gives some pretty serious
# hope that it won't be an issue in practice. The day is actually is a problem,
# we can revisit this, but the only solution I can think of presently is
# another layer of macros that generates the build rules for each path in SRCS,
# which is a pretty hairy sounding proposition

$(LIBNAME)_SRCS_FLAT := $(notdir $($(LIBNAME)_SRCS))
$(LIBNAME)_SRCS_DIRS := $(sort $(dir $($(LIBNAME)_SRCS))) # sort removes duplicates

# Only use vpath for certain types of files
# But must be a global list
VPATH_DIRS += $($(LIBNAME)_SRCS_DIRS)
vpath %.s $(VPATH_DIRS)
vpath %.c $(VPATH_DIRS)
vpath %.cc $(VPATH_DIRS)
vpath %.cpp $(VPATH_DIRS)

# Now, VPATH allows _make_ to find all the sources, but gcc needs to be told
# how to find all of the headers. We do this by `-I`'ing any folder that had a
# LIB_SRC and has any .h files in it. We also check the common convention of
# headers in an include/ folder while we're at it
define LIB_HEADER_INCLUDES
ifneq ($$(wildcard $(1)/*.h),"")
  CPPFLAGS += -I$(1)
endif
ifneq ($$(wildcard $(1)/include/*.h),"")
  CPPFLAGS += -I$(1)
endif
endef
# uncomment to print generated rules
# $(info $(foreach hdrdir,$($(LIBNAME)_SRCS_DIRS),$(call LIB_HEADER_INCLUDES,$(hdrdir))))
# actually generate the rules
$(foreach hdrdir,$($(LIBNAME)_SRCS_DIRS),$(eval $(call LIB_HEADER_INCLUDES,$(hdrdir))))

# Rules to generate libraries for a given Architecture
# These will be used to create the different architecture versions of LibNRFSerialization
# Argument $(1) is the Architecture (e.g. cortex-m0) to build for
define LIB_RULES

$$($(LIBNAME)_BUILDDIR)/$(1):
	$$(Q)mkdir -p $$@

$$($(LIBNAME)_BUILDDIR)/$(1)/%.o: %.c | $$($(LIBNAME)_BUILDDIR)/$(1)
	$$(TRACE_DEP)
	$$(Q)$$(CC) $$(CFLAGS) -mcpu=$(1) $$(CPPFLAGS) -MF"$$(@:.o=.d)" -MG -MM -MP -MT"$$(@:.o=.d)@" -MT"$$@" "$$<"
	$$(TRACE_CC)
	$$(Q)$$(CC) $$(CFLAGS) -mcpu=$(1) $$(CPPFLAGS) -c -o $$@ $$<

$$($(LIBNAME)_BUILDDIR)/$(1)/%.o: %.S | $$($(LIBNAME)_BUILDDIR)/$(1)
	$$(TRACE_AS)
	$$(Q)$$(AS) $$(ASFLAGS) -mcpu=$(1) $$(CPPFLAGS) -c -o $$@ $$<

$(LIBNAME)_OBJS_$(1) += $$(patsubst %.s,$$($(LIBNAME)_BUILDDIR)/$(1)/%.o,$$(filter %.s, $$($(LIBNAME)_SRCS_FLAT)))
$(LIBNAME)_OBJS_$(1) += $$(patsubst %.c,$$($(LIBNAME)_BUILDDIR)/$(1)/%.o,$$(filter %.c, $$($(LIBNAME)_SRCS_FLAT)))
$(LIBNAME)_OBJS_$(1) += $$(patsubst %.cc,$$($(LIBNAME)_BUILDDIR)/$(1)/%.o,$$(filter %.cc, $$($(LIBNAME)_SRCS_FLAT)))
$(LIBNAME)_OBJS_$(1) += $$(patsubst %.cpp,$$($(LIBNAME)_BUILDDIR)/$(1)/%.o,$$(filter %.cpp, $$($(LIBNAME)_SRCS_FLAT)))

# Dependency rules for picking up header changes
-include $$($(LIBNAME)_OBJS_$(1):.o=.d)

# Useful debugging
# $$(info -----------------------------------------------------)
# $$(info $(LIBNAME) $(1))
# $$(info      $(LIBNAME)_SRCS: $$($(LIBNAME)_SRCS))
# $$(info $(LIBNAME)_SRCS_FLAT: $$($(LIBNAME)_SRCS_FLAT))
# $$(info                VPATH: $$(VPATH))
# $$(info $(LIBNAME)_OBJS_$(1): $$($(LIBNAME)_OBJS_$(1)))
# $$(info =====================================================)

$$($(LIBNAME)_BUILDDIR)/$(1)/$$(LIBNAME).a: $$($(LIBNAME)_OBJS_$(1)) | $$($(LIBNAME)_BUILDDIR)/$(1)
	$$(TRACE_AR)
	$$(Q)$$(AR) rc $$@ $$^
	$$(Q)$$(RANLIB) $$@

# If we're building this library as part of a bigger build, add ourselves to
# the list of libraries
LIBS_$(1) += $$($(LIBNAME)_BUILDDIR)/$(1)/$$(LIBNAME).a
endef

# uncomment to print generated rules
# $(info $(foreach arch,$(TOCK_ARCHS), $(call LIB_RULES,$(arch))))
# actually generate the rules for each architecture
$(foreach arch,$(TOCK_ARCHS),$(eval $(call LIB_RULES,$(arch))))

# add each architecture as a target
.PHONY: all
all: $(foreach arch, $(TOCK_ARCHS),$($(LIBNAME)_BUILDDIR)/$(arch)/$(LIBNAME).a)

.PHONY: clean
clean::
	rm -Rf $($(LIBNAME)_BUILDDIR)
