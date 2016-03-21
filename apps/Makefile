DIRS := $(patsubst %/,%,$(filter-out libs/,$(filter-out extern/,$(wildcard */))))

.PHONY: all
all:	kernel $(DIRS)

.PHONY: clean
clean:	$(DIRS)

.PHONY: $(DIRS)
$(DIRS): kernel
	$(MAKE) -C $@ $(MAKECMDGOALS)

.PHONY: kernel
kernel:
	$(MAKE) -C .. $(MAKECMDGOALS)

