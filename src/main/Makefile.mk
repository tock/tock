# Exports rules to build `main.o`, to be linked with platform specific code to
# compile a binary image. For convenience,Also exposes rules to 

CORE_SOURCES=$(call rwildcard,src/main/,*.rs)
MAIN_DEPS=$(BUILD_DIR)/libcore.rlib $(BUILD_DIR)/libsupport.rlib $(CORE_SOURCES)

foo:
	@echo $(CORE_SOURCES)

$(BUILD_DIR)/main.o: $(MAIN_DEPS)
	@echo "Building $@"
	$(RUSTC) $(RUSTC_FLAGS) -C lto --emit obj -o $@ src/main/main.rs

$(BUILD_DIR)/main.S: $(MAIN_DEPS)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -C lto --emit asm -o $@ src/main/main.rs

$(BUILD_DIR)/main.ir: $(MAIN_DEPS)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -C lto --emit llvm-ir -o $@ src/main/main.rs

