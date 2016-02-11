$(BUILD_DIR)/ctx_switch.o: $(SRC_DIR)arch/$(ARCH)/ctx_switch.S
	@$(CC) $(CFLAGS) -c $^ -o $@

$(BUILD_DIR)/sync.o: $(SRC_DIR)arch/$(ARCH)/sync.c
	@$(CC) $(CFLAGS) -c $^ -o $@

