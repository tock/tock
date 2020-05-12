# By default, let's print out some help
.PHONY: usage
usage:
	@echo "$$(tput bold)Welcome to Tock!$$(tput sgr0)"
	@echo
	@echo "First things first, if you haven't yet, check out doc/Getting_Started."
	@echo "You'll need to install a few requirements before we get going."
	@echo
	@echo "The next step is to choose a board to build Tock for. Mainline"
	@echo "Tock currently includes support for the following platforms:"
	@for f in `./tools/list_boards.sh`; do printf " - $$f\n"; done
	@echo
	@echo "Run 'make' in a board directory to build Tock for that board,"
	@echo "and usually 'make program' or 'make flash' to load Tock onto hardware."
	@echo "Check out the README in your board's folder for more information."
	@echo
	@echo "This root Makefile has a few useful targets as well:"
	@echo "        allboards: Compiles Tock for all supported boards"
	@echo "        allcheck: Checks, but does not compile, Tock for all supported boards"
	@echo "          alldoc: Builds Tock documentation for all boards"
	@echo "           audit: Audit Cargo dependencies for all kernel sources"
	@echo "              ci: Run all continuous integration tests"
	@echo "    check-format: Checks if the rustfmt tool would require changes, but doesn't make them"
	@echo "           clean: Clean all builds"
	@echo " emulation-check: Run the emulation tests for supported boards"
	@echo " emulation-setup: Setup QEMU for the emulation tests"
	@echo "          format: Runs the rustfmt tool on all kernel sources"
	@echo "           lints: Runs check-format and the clippy code linter"
	@echo "            list: Lists available boards"
	@echo
	@echo "$$(tput bold)Happy Hacking!$$(tput sgr0)"

.PHONY: allboards
allboards:
	@for f in `./tools/list_boards.sh`; do echo "$$(tput bold)Build $$f"; $(MAKE) -C "boards/$$f" || exit 1; done

.PHONY: allcheck
allcheck:
	@for f in `./tools/list_boards.sh`; do echo "$$(tput bold)Check $$f"; $(MAKE) -C "boards/$$f" check || exit 1; done

.PHONY: alldoc
alldoc:
	@for f in `./tools/list_boards.sh`; do echo "$$(tput bold)Documenting $$f"; $(MAKE) -C "boards/$$f" doc || exit 1; done



###################################################################
##
## Continuous Integration Targets
##
## To run all CI locally, use the meta-target `make ci`.
##
## Each of the phases of CI is broken into its own target to enable
## quick local iteration without re-running all phases of CI.
##


## Meta-Targets

.PHONY: ci
ci: ci-travis ci-netlify

.PHONY: ci-travis
ci-travis:\
	ci-lints\
	ci-tools\
	ci-libraries\
	ci-archs\
	ci-kernel\
	ci-chips\
	ci-syntax\
	ci-compilation\
	ci-debug-support-targets\
	ci-documentation \
	emulation-check
	@printf "$$(tput bold)********************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI-Travis: Done! *$$(tput sgr0)\n"
	@printf "$$(tput bold)********************$$(tput sgr0)\n"

.PHONY: ci-netlify
ci-netlify:\
	ci-rustdoc
	@printf "$$(tput bold)*********************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI-Netlify: Done! *$$(tput sgr0)\n"
	@printf "$$(tput bold)*********************$$(tput sgr0)\n"

.PHONY: ci-cargo-tests
ci-cargo-tests:\
	ci-libraries\
	ci-archs\
	ci-kernel\
	ci-chips\

.PHONY: ci-format
ci-format:\
	ci-lints\
	ci-documentation\

.PHONY: ci-build
ci-build:\
	ci-syntax\
	ci-compilation\
	ci-debug-support-targets\

.PHONY: ci-tests
ci-tests:\
	ci-cargo-tests\
	ci-tools\

## Actual Rules (Travis)

.PHONY: ci-lints
ci-lints:
	@printf "$$(tput bold)**************************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: Formatting + Lints *$$(tput sgr0)\n"
	@printf "$$(tput bold)**************************$$(tput sgr0)\n"
	@$(MAKE) lints

.PHONY: ci-tools
ci-tools:
	@printf "$$(tput bold)*************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: Tools *$$(tput sgr0)\n"
	@printf "$$(tput bold)*************$$(tput sgr0)\n"
	@for f in `./tools/list_tools.sh`; do echo "$$(tput bold)Build & Test $$f"; cd tools/$$f && CI=true RUSTFLAGS="-D warnings" cargo build --all-targets || exit 1; cd - > /dev/null; done

.PHONY: ci-libraries
ci-libraries:
	@printf "$$(tput bold)*****************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: Libraries *$$(tput sgr0)\n"
	@printf "$$(tput bold)*****************$$(tput sgr0)\n"
	@cd libraries/enum_primitive && CI=true RUSTFLAGS="-D warnings" cargo test
	@cd libraries/riscv-csr && CI=true RUSTFLAGS="-D warnings" cargo test
	@cd libraries/tock-cells && CI=true RUSTFLAGS="-D warnings" cargo test
	@cd libraries/tock-register-interface && CI=true RUSTFLAGS="-D warnings" cargo test
	@cd libraries/tock-rt0 && CI=true RUSTFLAGS="-D warnings" cargo test

.PHONY: ci-archs
ci-archs:
	@printf "$$(tput bold)*************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: Archs *$$(tput sgr0)\n"
	@printf "$$(tput bold)*************$$(tput sgr0)\n"
	@for f in `./tools/list_archs.sh`; do echo "$$(tput bold)Test $$f"; cd arch/$$f; CI=true RUSTFLAGS="-D warnings" TOCK_KERNEL_VERSION=ci_test cargo test || exit 1; cd ../..; done

.PHONY: ci-chips
ci-chips:
	@printf "$$(tput bold)*************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: Chips *$$(tput sgr0)\n"
	@printf "$$(tput bold)*************$$(tput sgr0)\n"
	@for f in `./tools/list_chips.sh`; do echo "$$(tput bold)Test $$f"; cd chips/$$f; CI=true RUSTFLAGS="-D warnings" TOCK_KERNEL_VERSION=ci_test cargo test || exit 1; cd ../..; done

.PHONY: ci-kernel
ci-kernel:
	@printf "$$(tput bold)**************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: Kernel *$$(tput sgr0)\n"
	@printf "$$(tput bold)**************$$(tput sgr0)\n"
	@cd kernel && CI=true RUSTFLAGS="-D warnings" TOCK_KERNEL_VERSION=ci_test cargo test

.PHONY: ci-syntax
ci-syntax:
	@printf "$$(tput bold)**************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: Syntax *$$(tput sgr0)\n"
	@printf "$$(tput bold)**************$$(tput sgr0)\n"
	@CI=true $(MAKE) allcheck

.PHONY: ci-compilation
ci-compilation:
	@printf "$$(tput bold)*******************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: Compilation *$$(tput sgr0)\n"
	@printf "$$(tput bold)*******************$$(tput sgr0)\n"
	@CI=true $(MAKE) allboards

.PHONY: ci-debug-support-targets
ci-debug-support-targets:
	# These are rules that build additional debugging information, but are
	# also quite time consuming. So we want to verify that the rules still
	# work, but don't build them for every board.
	#
	# The choice of building for the nrf52dk was chosen by random die roll.
	@printf "$$(tput bold)*****************************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: Debug Support Targets *$$(tput sgr0)\n"
	@printf "$$(tput bold)*****************************$$(tput sgr0)\n"
	@CI=true $(MAKE) -C boards/nordic/nrf52dk lst
	@CI=true $(MAKE) -C boards/nordic/nrf52dk debug
	@CI=true $(MAKE) -C boards/nordic/nrf52dk debug-lst

.PHONY: ci-documentation
ci-documentation:
	@printf "$$(tput bold)*********************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: Documentation *$$(tput sgr0)\n"
	@printf "$$(tput bold)*********************$$(tput sgr0)\n"
	@CI=true tools/toc.sh


## Actual Rules (Netlify)

.PHONY: ci-rustdoc
ci-rustdoc:
	@printf "$$(tput bold)*****************************$$(tput sgr0)\n"
	@printf "$$(tput bold)* CI: Rustdoc Documentation *$$(tput sgr0)\n"
	@printf "$$(tput bold)*****************************$$(tput sgr0)\n"
	@#n.b. netlify calls tools/netlify-build.sh, which is a wrapper
	@#     that first installs toolchains, then calls this.
	@tools/build-all-docs.sh

## End CI rules
##
###################################################################


.PHONY: audit
audit:
	@for f in `./tools/list_lock.sh`; do echo "$$(tput bold)Auditing $$f"; (cd "$$f" && cargo audit || exit 1); done

.PHONY: emulation-setup
emulation-setup: SHELL:=/usr/bin/env bash
emulation-setup:
	@#Use the latest QEMU as it has OpenTitan support
	@if [[ ! -d tools/qemu || ! -f tools/qemu/VERSION ]]; then \
		rm -rf tools/qemu; \
		cd tools; git clone https://github.com/qemu/qemu.git; \
		cd qemu; ./configure --target-list=riscv32-softmmu; \
	fi
	@$(MAKE) -C "tools/qemu" > /dev/null

.PHONY: emulation-check
emulation-check: emulation-setup
	@$(MAKE) -C "boards/hifive1"
	@cd tools/qemu-runner; PATH="$(shell pwd)/tools/qemu/riscv32-softmmu/:${PATH}" cargo run

.PHONY: clean
clean:
	@echo "$$(tput bold)Clean top-level Cargo workspace" && cargo clean
	@for f in `./tools/list_tools.sh`; do echo "$$(tput bold)Clean tools/$$f"; cargo clean --manifest-path "tools/$$f/Cargo.toml" || exit 1; done
	@echo "$$(tput bold)Clean rustdoc" && rm -Rf doc/rustdoc
	@echo "$$(tput bold)Clean ci-artifacts" && rm -Rf ./ci-artifacts

.PHONY: fmt format
fmt format formatall:
	@./tools/run_cargo_fmt.sh

.PHONY: check-format
check-format:
	@CI=true ./tools/run_cargo_fmt.sh diff

.PHONY: lints
lints:\
	check-format
	@./tools/run_clippy.sh

.PHONY: list list-boards list-platforms
list list-boards list-platforms:
	@echo "Supported Tock Boards:"
	@for f in `./tools/list_boards.sh`; do printf " - $$f\n"; done
	@echo
	@echo "To build the kernel for a particular board, change to that directory"
	@echo "and run make:"
	@echo "    cd boards/hail"
	@echo "    make"

.PHONY: ci-collect-artifacts
ci-collect-artifacts:
	@test -d ./target || (echo "Target directory not found! Build some boards first to have their artifacts collected"; exit 1)
	@mkdir -p ./ci-artifacts
	@rm -rf "./ci-artifacts/*"
	@for f in $$(find ./target -iname '*.bin' | grep -E "release/.*\.bin"); do mkdir -p "ci-artifacts/$$(dirname $$f)"; cp "$$f" "ci-artifacts/$$f"; done
