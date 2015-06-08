$(BUILD_DIR)/arch.o: src/arch/cortex-m4/ctx_switch.S
	@$(CC) $(CFLAGS) $(LDFLAGS) $^ -o $@ -ffreestanding -lgcc -lc
