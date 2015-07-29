$(BUILD_DIR)/arch.o: src/arch/cortex-m4/ctx_switch.S src/arch/cortex-m4/syscalls.S
	$(TOOLCHAIN)as -mcpu=cortex-m4 -mthumb $^ -o $@
