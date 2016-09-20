BOARD ?= storm

.PHONY: all
all: $(BOARD)

%: boards/%/
	make -C $<

examples/%: userland/examples/%
	make -C $<

