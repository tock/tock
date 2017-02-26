# default board
TOCK_BOARD ?= storm


# rules for making the kernel
.PHONY: all allboards fmt format
all: $(TOCK_BOARD)

$(TOCK_BOARD): boards/$(TOCK_BOARD)/
	$(MAKE) -C $<

allboards:
	@for f in `./tools/list_boards.sh -1`; do echo "$$(tput bold)Build $$f"; $(MAKE) -C "boards/$$f" || exit 1; done

clean:: boards/$(TOCK_BOARD)/
	$(MAKE) clean -C $<

doc: boards/$(TOCK_BOARD)/
	$(MAKE) doc -C $<

debug: boards/$(TOCK_BOARD)/
	$(MAKE) debug -C $<

program: boards/$(TOCK_BOARD)/
	$(MAKE) program -C $<

flash: boards/$(TOCK_BOARD)/
	$(MAKE) flash -C $<

fmt format:
	@./tools/run_cargo_fmt.sh

list list-boards list-platforms:
	@./tools/list_boards.sh

# rule for making userland example applications
# 	automatically upload after making
examples/%: userland/examples/%
	$(MAKE) -C $< TOCK_BOARD=$(TOCK_BOARD)
	$(MAKE) program -C $< TOCK_BOARD=$(TOCK_BOARD)

