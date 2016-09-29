# default board and architecture
TOCK_BOARD ?= storm
TOCK_ARCH ?= cortex-m4


# rules for making the kernel
.PHONY: all
all: $(TOCK_BOARD)

$(TOCK_BOARD): boards/$(TOCK_BOARD)/
	$(MAKE) -C $<

clean: boards/$(TOCK_BOARD)/
	$(MAKE) clean -C $<

doc: boards/$(TOCK_BOARD)/
	$(MAKE) doc -C $<

debug: boards/$(TOCK_BOARD)/
	$(MAKE) debug -C $<

program: boards/$(TOCK_BOARD)/
	$(MAKE) program -C $<

flash: boards/$(TOCK_BOARD)/
	$(MAKE) flash -C $<


# rule for making userland example applications
# 	automatically upload after making
examples/%: userland/examples/%
	$(MAKE) -C $< TOCK_ARCH=$(TOCK_ARCH)
	$(MAKE) program -C $< TOCK_ARCH=$(TOCK_ARCH)

