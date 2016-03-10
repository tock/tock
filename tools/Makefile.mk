$(BUILD_ROOT)/elf2tbf: $(BASE_DIR)tools/elf2tbf/target/release/elf2tbf | $(BUILD_ROOT)
	@cp $(BASE_DIR)tools/elf2tbf/target/release/elf2tbf $(BUILD_ROOT)/elf2tbf

.PHONY: $(BASE_DIR)tools/elf2tbf/target/release/elf2tbf
$(BASE_DIR)tools/elf2tbf/target/release/elf2tbf:
	@$(CARGO) build --release --manifest-path=$(BASE_DIR)tools/elf2tbf/Cargo.toml

