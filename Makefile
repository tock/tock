# By default, let's print out some help
.PHONY: usage
usage:
	@echo "$$(tput bold)Welcome to Tock!$$(tput sgr0)"
	@echo
	@echo "First things first, if you haven't yet, check out doc/Getting_Started."
	@echo "You'll need to install a few requirements before we get going."
	@echo
	@echo "The next step is to choose a board to build Tock for."
	@echo "Mainline Tock currently includes support for:"
	@ls -p boards/ | grep '/$$' | cut -d'/' -f1 | xargs echo "  "
	@echo
	@echo "Run 'make' in a board directory to build Tock for that board,"
	@echo "and usually 'make program' or 'make flash' to load Tock onto hardware."
	@echo "Check out the README in your board's folder for more information."
	@echo
	@echo "This root Makefile has a few useful targets as well:"
	@echo "  allboards: Compiles Tock for all supported boards"
	@echo "   allcheck: Checks, but does not compile, Tock for all supported boards"
	@echo "     alldoc: Builds Tock documentation for all boards"
	@echo "         ci: Run all continuous integration tests"
	@echo "     format: Runs the rustfmt tool on all kernel sources"
	@echo "  formatall: Runs all formatting tools"
	@echo "       list: Lists available boards"
	@echo
	@echo "$$(tput bold)Happy Hacking!$$(tput sgr0)"

.PHONY: allboards
allboards:
	@for f in `./tools/list_boards.sh -1`; do echo "$$(tput bold)Build $$f"; $(MAKE) -C "boards/$$f" || exit 1; done

.PHONY: allcheck
allcheck:
	@for f in `./tools/list_boards.sh -1`; do echo "$$(tput bold)Check $$f"; $(MAKE) -C "boards/$$f" check || exit 1; done

.PHONY: alldoc
alldoc:
	@for f in `./tools/list_boards.sh -1`; do echo "$$(tput bold)Documenting $$f"; $(MAKE) -C "boards/$$f" doc || exit 1; done

.PHONY: ci-travis
ci-travis:
	@printf "$$(tput bold)******************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: Formatting *$$(tput sgr0)\n"
	@printf "$$(tput bold)******************$$(tput sgr0)\n"
	@CI=true ./tools/run_cargo_fmt.sh diff
	@printf "$$(tput bold)*****************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: Libraries *$$(tput sgr0)\n"
	@printf "$$(tput bold)*****************$$(tput sgr0)\n"
	@cd libraries/tock-cells && CI=true cargo test
	@cd libraries/tock-register-interface && CI=true cargo test
	@printf "$$(tput bold)**************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: Syntax *$$(tput sgr0)\n"
	@printf "$$(tput bold)**************$$(tput sgr0)\n"
	@CI=true $(MAKE) allcheck
	@printf "$$(tput bold)****************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: DocTests *$$(tput sgr0)\n"
	@printf "$$(tput bold)****************$$(tput sgr0)\n"
	@cd kernel && CI=true TOCK_KERNEL_VERSION=ci_test cargo test
	@printf "$$(tput bold)*******************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: Compilation *$$(tput sgr0)\n"
	@printf "$$(tput bold)*******************$$(tput sgr0)\n"
	@CI=true $(MAKE) allboards
	@printf "$$(tput bold)***********************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: Special Targets *$$(tput sgr0)\n"
	@printf "$$(tput bold)***********************$$(tput sgr0)\n"
	@CI=true $(MAKE) -C boards/nordic/nrf52dk lst
	@CI=true $(MAKE) -C boards/nordic/nrf52dk debug
	@CI=true $(MAKE) -C boards/nordic/nrf52dk debug-lst
	@printf "$$(tput bold)*********************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: Documentation *$$(tput sgr0)\n"
	@printf "$$(tput bold)*********************$$(tput sgr0)\n"
	@CI=true tools/toc.sh
	@printf "$$(tput bold)*************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: Done! *$$(tput sgr0)\n"
	@printf "$$(tput bold)*************$$(tput sgr0)\n"

.PHONY: ci-netlify
ci-netlify:
	@#n.b. netlify calls tools/netlify-build.sh, which is a wrapper
	@#     that first installs toolchains, then calls this.
	@tools/build-all-docs.sh

.PHONY: ci
ci: ci-travis ci-netlify

.PHONY: fmt format formatall
fmt format formatall:
	@./tools/run_cargo_fmt.sh

.PHONY: list list-boards list-platforms
list list-boards list-platforms:
	@./tools/list_boards.sh

