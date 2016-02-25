$(BUILD_PLATFORM_DIR)/ctx_switch.o: $(SRC_DIR)arch/$(ARCH)/ctx_switch.S | $(BUILD_PLATFORM_DIR)
	@$(CC) $(CFLAGS) -c $^ -o $@

