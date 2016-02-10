LIBFIRESTORM = $(BUILD_APP_DIR)/firestorm.o
LIBTOCK = $(BUILD_APP_DIR)/tock.o

$(LIBFIRESTORM): $(SRC_DIR)apps/libs/firestorm.c $(SRC_DIR)apps/libs/firestorm.h | $(BUILD_APP_DIR)
	@echo "Building libfirestorm for apps"
	@$(CC) $(CFLAGS_BASE) $(CFLAGS_APPS) -c -g -Os -o $@ -ffreestanding -nostdlib $<

$(LIBTOCK): $(SRC_DIR)apps/libs/tock.c $(SRC_DIR)apps/libs/tock.h | $(BUILD_APP_DIR)
	@echo "Building libtock for apps"
	@$(CC) $(CFLAGS_BASE) $(CFLAGS_APPS) -c -g -Os -o $@ -ffreestanding -nostdlib $<

$(BUILD_APP_DIR)/arch.o: $(SRC_DIR)arch/$(ARCH)/syscalls.S
	@echo "Building arch.o for apps"
	@$(CC) $(CFLAGS_BASE) $(CFLAGS_APPS) -c -g -Os -o $@ -ffreestanding -nostdlib $<

$(BUILD_APP_DIR)/crt1.o: $(SRC_DIR)apps/libs/crt1.c | $(BUILD_APP_DIR)
	@echo "Building crt1 for apps"
	@$(CC) $(CFLAGS_BASE) $(CFLAGS_APPS) -c -g -Os -o $@ -ffreestanding -nostdlib $<

$(BUILD_APP_DIR)/sys.o: $(SRC_DIR)apps/libs/sys.c | $(BUILD_APP_DIR)
	@echo "Building libc stubs for apps"
	@$(CC) $(CFLAGS_BASE) $(CFLAGS_APPS) -c -g -Os -o $@ -ffreestanding -nostdlib $<

$(BUILD_APP_DIR)/tmp006.o: $(SRC_DIR)apps/libs/tmp006.c $(SRC_DIR)apps/libs/tmp006.h | $(BUILD_APP_DIR)
	@echo "Building TMP006 driver for apps"
	@$(CC) $(CFLAGS_BASE) $(CFLAGS_APPS) -c -g -Os -o $@ -ffreestanding -nostdlib $<

