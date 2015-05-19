VERSION_CMD = rustc --version | head -n 1 | sed 's/[^(]*(\([^ ]*\).*/\1/'
RUSTC_VERSION=$(shell $(VERSION_CMD))

LIBCORE_DIR=$(BUILD_DIR)/core-$(RUSTC_VERSION)

extern/rustc-$(RUSTC_VERSION)-src.tar.gz:
	@echo "Need libcore to compile Tock: fetching source $(@F)"
	wget -q -O $@ https://github.com/rust-lang/rust/archive/$(RUSTC_VERSION).tar.gz

extern/rustc/src/libcore/lib.rs: extern/rustc-$(RUSTC_VERSION)-src.tar.gz
	@echo "Untarring $(<F)"
	@rm -rf extern/rustc
	@mkdir -p extern/rustc
	@tar -C extern/rustc -zx --strip-components=1 -f $^
	@touch $@ # Touch so lib.rs appears newer than tarball

$(LIBCORE_DIR)/libcore.rlib: extern/rustc/src/libcore/lib.rs | $(BUILD_DIR)
	@echo "Building $@"
	@mkdir -p $(LIBCORE_DIR)
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(LIBCORE_DIR) extern/rustc/src/libcore/lib.rs

$(BUILD_DIR)/libcore.rlib: $(LIBCORE_DIR)/libcore.rlib
	@echo "Copying $< to $@"
	@cp $< $@
