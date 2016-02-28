DIRS := $(patsubst %/,%,$(filter-out libs/,$(filter-out extern/,$(wildcard */))))

.PHONY: all
all:	kernel $(DIRS)

.PHONY: $(DIRS)
$(DIRS): kernel
	$(MAKE) -C $@

.PHONY: kernel
kernel:
	$(MAKE) -C ..

