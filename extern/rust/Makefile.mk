RUSTC ?= rustc
VERSION_CMD = $(RUSTC) --version | head -n 1 | sed 's/[^(]*(\([^ ]*\).*/\1/'
RUSTC_VERSION=$(shell $(VERSION_CMD))

LIBCORE_DIR=$(BUILD_DIR)/core-$(RUSTC_VERSION)

$(EXTERN_DIR)rust/rustc-$(RUSTC_VERSION)-src.tar.gz:
	@echo "Need libcore to compile Tock: fetching source $(@F)"
	@wget -q -O $@ https://github.com/rust-lang/rust/archive/$(RUSTC_VERSION).tar.gz

$(EXTERN_DIR)rust/rustc/src/libcore/lib.rs: $(EXTERN_DIR)rust/rustc-$(RUSTC_VERSION)-src.tar.gz
	@echo "Extracting $(<F)"
	@rm -rf $(EXTERN_DIR)/rust/rustc
	@mkdir -p $(EXTERN_DIR)/rust/rustc
	@tar -C $(EXTERN_DIR)/rust/rustc -zx --strip-components=1 -f $^ --force-local
	@touch $@ # Touch so lib.rs appears newer than tarball

$(LIBCORE_DIR)/libcore.rlib: $(EXTERN_DIR)rust/rustc/src/libcore/lib.rs | $(BUILD_DIR)
	@echo "Building $@"
	@mkdir -p $(LIBCORE_DIR)
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(LIBCORE_DIR) $(EXTERN_DIR)/rust/rustc/src/libcore/lib.rs

$(BUILD_DIR)/libcore.rlib: $(LIBCORE_DIR)/libcore.rlib
	@echo "Copying $< to $@"
	@cp $< $@

