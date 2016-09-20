# default board and architecture
BOARD ?= storm
ARCH ?= cortex-m4


# rules for making the kernel
.PHONY: all
all: $(BOARD)

$(BOARD): boards/$(BOARD)/
	$(MAKE) -C $<

clean: boards/$(BOARD)/
	$(MAKE) clean -C $<

doc: boards/$(BOARD)/
	$(MAKE) doc -C $<

program: boards/$(BOARD)/
	$(MAKE) program -C $<

flash: boards/$(BOARD)/
	$(MAKE) flash -C $<


# rule for making userland example applications
examples/%: userland/examples/%
	$(MAKE) -C $< ARCH=$(ARCH)

