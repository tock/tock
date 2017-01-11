NEWLIB_VERSION ?= 2.5.0

all: rebuild-newlib

newlib-$(NEWLIB_VERSION).tar.gz:
	@echo "Downloading newlib source $(@F)"
	@wget -q -O $@ ftp://sourceware.org/pub/newlib/newlib-$(NEWLIB_VERSION).tar.gz
	patch -p0 < newlib-lto.patch

newlib-$(NEWLIB_VERSION): newlib-$(NEWLIB_VERSION).tar.gz
	@echo "Extracting $(<F)"
	@tar -xzf $<
	@touch $@ # Touch so directory appears newer than tarball

rebuild-newlib: newlib-$(NEWLIB_VERSION)
	@rm -rf newlib-$(NEWLIB_VERSION)-out
	@mkdir -p newlib-$(NEWLIB_VERSION)-out
	@echo "Entering directory newlib-$(NEWLIB_VERSION)-out"
	cd newlib-$(NEWLIB_VERSION)-out; sh ../build.sh ../$<
	@echo "Copying built artifacts"
	@cp newlib-$(NEWLIB_VERSION)-out/arm-none-eabi/thumb/v6-m/newlib/libc.a cortex-m0/
	@cp newlib-$(NEWLIB_VERSION)-out/arm-none-eabi/thumb/v6-m/newlib/libg.a cortex-m0/
	@cp newlib-$(NEWLIB_VERSION)-out/arm-none-eabi/thumb/v6-m/newlib/libm.a cortex-m0/
	@cp newlib-$(NEWLIB_VERSION)-out/arm-none-eabi/thumb/v7e-m/newlib/libc.a cortex-m4/
	@cp newlib-$(NEWLIB_VERSION)-out/arm-none-eabi/thumb/v7e-m/newlib/libg.a cortex-m4/
	@cp newlib-$(NEWLIB_VERSION)-out/arm-none-eabi/thumb/v7e-m/newlib/libm.a cortex-m4/

