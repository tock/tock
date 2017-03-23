# Makefile for loading applications onto a Tockloader compatible board

$(call check_defined, BUILDDIR)
$(call check_defined, PACKAGE_NAME)

APP_TOCKLOADER ?= tockloader

# Upload programs over UART with tockloader
ifdef PORT
  TOCKLOADER_GENERAL_FLAGS += --port $(PORT)
endif

.PHONY: program
program: $(BUILDDIR)/$(PACKAGE_NAME).tab
	$(APP_TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) replace --add $<

# Upload programs over JTAG
.PHONY: flash
flash: $(BUILDDIR)/$(PACKAGE_NAME).tab
	$(APP_TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) replace --add --jtag $<
