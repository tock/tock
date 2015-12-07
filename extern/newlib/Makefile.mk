NEWLIB_VERSION ?= 2.2.0.20150423

$(EXTERN_DIR)newlib/newlib-$(NEWLIB_VERSION).tar.gz:
	@echo "Downloading newlib source $(@F)"
	@wget -q -O $@ ftp://sourceware.org/pub/newlib/newlib-$(NEWLIB_VERSION).tar.gz

$(EXTERN_DIR)newlib/newlib-$(NEWLIB_VERSION): $(EXTERN_DIR)newlib/newlib-$(NEWLIB_VERSION).tar.gz
	@echo "Extracting $(<F)"
	@tar -C $(EXTERN_DIR)newlib/ -xzf $<
	@touch $@ # Touch so directory appears newer than tarball

rebuild-newlib: $(EXTERN_DIR)newlib/newlib-$(NEWLIB_VERSION)
	@rm -rf $(EXTERN_DIR)newlib/newlib-$(NEWLIB_VERSION)-out
	@mkdir -p $(EXTERN_DIR)newlib/newlib-$(NEWLIB_VERSION)-out
	@echo "Entering directory $(EXTERN_DIR)newlib/newlib-$(NEWLIB_VERSION)-out"
	@cd $(EXTERN_DIR)newlib/newlib-$(NEWLIB_VERSION)-out; sh ../build.sh $<

