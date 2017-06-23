# default board
TOCK_BOARD ?= hail


# rules for making the kernel
.PHONY: all
all: $(TOCK_BOARD)

.PHONY: $(TOCK_BOARD)
$(TOCK_BOARD): boards/$(TOCK_BOARD)/
	$(MAKE) -C $<

.PHONY: allboards
allboards:
	@for f in `./tools/list_boards.sh -1`; do echo "$$(tput bold)Build $$f"; $(MAKE) -C "boards/$$f" || exit 1; done

.PHONY: clean
clean:: boards/$(TOCK_BOARD)/
	$(MAKE) clean -C $<

.PHONY: doc
doc: boards/$(TOCK_BOARD)/
	$(MAKE) doc -C $<

.PHONY: debug
debug: boards/$(TOCK_BOARD)/
	$(MAKE) debug -C $<

.PHONY: program
program: boards/$(TOCK_BOARD)/
	$(MAKE) program -C $<

.PHONY: flash
flash: boards/$(TOCK_BOARD)/
	$(MAKE) flash -C $<

.PHONY: fmt format
fmt format:
	@./tools/run_cargo_fmt.sh

.PHONY: formatall
formatall: format
	@cd userland/examples && ./format_all.sh

.PHONY: list list-boards list-platforms
list list-boards list-platforms:
	@./tools/list_boards.sh

# rule for making userland example applications
# 	automatically upload after making
examples/%: userland/examples/%
	$(MAKE) -C $< TOCK_BOARD=$(TOCK_BOARD)
	$(MAKE) program -C $< TOCK_BOARD=$(TOCK_BOARD)

