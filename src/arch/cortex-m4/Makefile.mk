$(BUILD_DIR)/arch.o: src/arch/cortex-m4/ctx_switch.S src/arch/cortex-m4/syscalls.S
	@$(CC) $(CFLAGS) $(LDFLAGS) $^ -o $@ -ffreestanding -lgcc -lc
