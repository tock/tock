# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# For more information on Tock's make system and the CI setup, see the docs at
# https://github.com/tock/tock/tree/master/doc/CodeReview.md#3-continuous-integration

################################################################################
##
## Internal support that needs to run first
##

# First, need to fill out some variables that the Makefile will use
$(eval ALL_BOARDS := $(shell ./tools/list_boards.sh))

# Force the Shell to be bash as some systems have strange default shells
SHELL := bash

##
## End: internal support.
##
################################################################################
##
## User interface / usage
##

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
	@for f in $(ALL_BOARDS); do printf " - $$f\n"; done
	@echo
	@echo "Run 'make' in a board directory to build Tock for that board, and then"
	@echo "run 'make install' to load Tock onto hardware. Check out the README in"
	@echo "your board's folder for more information."
	@echo
	@echo "There are a few helpful targets that can be run for individual boards. To"
	@echo "run these, run 'make {target}' from the board directory for these targets:"
	@echo "      cargobloat: Runs the cargo-bloat tool for attributing binary size"
	@echo "  stack-analysis: Prints the 5 largest stack frames for the board"
	@echo
	@echo "This root Makefile has a few useful targets as well:"
	@echo "           audit: Audit Cargo dependencies for all kernel sources"
	@echo "          boards: Compiles Tock for all supported boards"
	@echo "           check: Checks, but does not compile, Tock for all supported boards"
	@echo "             doc: Builds Tock documentation for all boards"
	@echo "           stack: Prints a basic stack frame analysis for all boards"
	@echo "           clean: Clean all builds"
	@echo "    format-check: Checks for formatting errors in kernel sources"
	@echo "            list: Lists available boards"
	@echo
	@echo "We also define the following aliases:"
	@echo "          format: cargo fmt"
	@echo
	@echo "The make system also drives all continuous integration and testing:"
	@echo "         $$(tput bold)prepush$$(tput sgr0): Fast checks to run before pushing changes upstream"
	@echo "          ci-all: Run all continuous integration tests (possibly slow!)"
	@echo "         ci-help: More information on Tock CI and testing"
	@echo
	@echo "$$(tput bold)Happy Hacking!$$(tput sgr0)"

##
## End: usage.
##
################################################################################
##
## Utility functions
##

define banner
	@printf "\n"
	@printf "$$(tput bold)********************************************************************************$$(tput sgr0)\n"
	@string="$(1)" && printf "$$(tput bold)* %-$$((76))s *\n" "$$string"
	@printf "$$(tput bold)********************************************************************************$$(tput sgr0)\n"
	@printf "\n"
endef

# Four arguments:
#  1) Command to check if already installed
#  2) String that explains what will be executed if install runs
#  3) Make function that does the work
#  4) Guard variable that is defined if job is to run
define ci_setup_helper
	@# The line continuation adds a leading space, remove
	$(eval explanation := $(strip $(2)))
	$(eval build_function := $(strip $(3)))
	$(eval guard_variable := $(strip $(4)))
	$(eval already_installed := $(shell $(1)))
	@# First, if the dependency is installed, we can bail early
	$(if $(already_installed),$(eval $(guard_variable) := true),
	@# If running in CI context always yes
	$(if $(CI),$(eval do_install := yes_CI),
	@# If running nosetup always no
	$(if $(TOCK_NOSETUP),$(eval do_install := ),
	@# Otherwise, ask
	$(info )
	$(info You have run a (likely CI) rule that requires Tock to run setup commands on your)
	$(info machine. Tock can do this automatically for you, or you can look at the recipe)
	$(info for '$(build_function)' and do it yourself.)
	$(info )
	$(info Continuing will: $(explanation).)
	$(info )
	$(info You can use 'make ci-nosetup' to run all CI with no new setup requirements.)
	$(info )
	$(eval do_install := $(shell read -p "Should Tock run setup commands for you? [y/N] " response && if [[ ( "$$(echo "$$response" | tr :upper: :lower:)" == "y" ) ]]; then echo yes; fi))
	) @# End if TOCK_NOSETUP
	) @# End if CI
	$(if $(do_install),
	$(call $(3))
	$(eval $(guard_variable) := true)
	, @# else of do_install
	$(if $(TOCK_NOSETUP),
	@# If no setup requested, let this go quietly
	, @# else of TOCK_NOSETUP
	$(error Missing required external dependency)
	) @# End if TOCK_NOSETUP
	) @# End of if do_install
	) @# End if already installed
endef

##
## End: functions.
##
################################################################################
##
## User convenience targets
##

## Aggregate targets
.PHONY: allaudit audit
allaudit audit:
	@for f in `./tools/list_lock.sh`;\
		do echo "$$(tput bold)Auditing $$f";\
		(cd "$$f" && cargo audit || exit 1);\
		done

.PHONY: allboards boards
allboards boards:
	@for f in $(ALL_BOARDS);\
		do echo "$$(tput bold)Build $$f";\
		$(MAKE) -C "boards/$$f" || exit 1;\
		done

.PHONY: allcheck check
allcheck check:
	@cargo check

.PHONY: alldoc doc
alldoc doc:
	@for f in $(ALL_BOARDS);\
		do echo "$$(tput bold)Documenting $$f";\
		$(MAKE) -C "boards/$$f" doc || exit 1;\
		done

.PHONY: allstack stack stack-analysis
allstack stack stack-analysis:
	@for f in $(ALL_BOARDS);\
		do $(MAKE) --no-print-directory -C "boards/$$f" stack-analysis || exit 1;\
		done

.PHONY: licensecheck
licensecheck:
	$(call banner,License checker)
	@cargo run --manifest-path=tools/license-checker/Cargo.toml --release

## Commands
.PHONY: clean
clean:
	@echo "$$(tput bold)Clean top-level Cargo workspace" && cargo clean
	@echo "$$(tput bold)Clean tools Cargo workspace" && cargo clean --manifest-path tools/Cargo.toml
	@echo "$$(tput bold)Clean rustdoc" && rm -rf doc/rustdoc
	@echo "$$(tput bold)Clean ci-artifacts" && rm -rf tools/ci-artifacts

.PHONY: fmt format
fmt format:
	$(call banner,Running \"cargo fmt\" -- for a complete format check run \"make format-check\")
	cargo fmt

.PHONY: format-check
format-check:
	$(call banner,Formatting checker)
	@./tools/check_format.sh
	$(call banner,Check for formatting complete)

.PHONY: list
list:
	@echo "Supported Tock Boards:"
	@for f in $(ALL_BOARDS); do printf " - $$f\n"; done
	@echo
	@echo "To build the kernel for a particular board, change to that directory"
	@echo "and run make:"
	@echo "    cd boards/hail"
	@echo "    make"


## Meta-Targets

# Run all possible CI. If this passes locally, all cloud CI *must* pass as well.
.PHONY: ci-all
ci-all:\
	ci-runner-github\
	ci-runner-netlify

# Run all CI that doesn't require installation of extra tools.
#
# Note that this will run things that require setup that has already been
# completed. It simply will not prompt for *new* installs.
.PHONY: ci-nosetup
ci-nosetup:
	@TOCK_NOSETUP=true $(MAKE) ci-all

# Run the fast jobs.
# This is designed for developers, to be run often and before submitting code upstream.
.PHONY: prepush
prepush:\
	format-check\
	ci-job-clippy\
	ci-job-syntax\
	licensecheck
	$(call banner,Pre-Push checks all passed!)
	# Note: Tock runs additional and more intense CI checks on all PRs.
	# If one of these error, you can run `make ci-job-NAME` to test locally.


## Hidden convenience targets
##
## These are aliases often used by the core team, but are not intended to be
## part of the official build system interface. They are subject to change at
## any time without notice.
.PHONY: clippy
clippy: ci-job-clippy


# And print some help
#
# https://stackoverflow.com/questions/4219255/how-do-you-get-the-list-of-targets-in-a-makefile
.PHONY: ci-help
ci-help:
	@echo "Welcome to Tock CI"
	@echo
	@echo "Tock works hard to automate as much of testing as possible to ensure that"
	@echo "platforms always work. For full details on the CI infrastructure, please"
	@echo "review the documentation at 'doc/CodeReview.md'."
	@echo
	@echo "The following CI runners are available:"
	@$(MAKE) -pRrq -f $(lastword $(MAKEFILE_LIST)) : 2>/dev/null | awk -v RS= -F: '/^# File/,/^# Finished Make data base/ {if ($$1 !~ "^[#.]") {print $$1}}' | sort | egrep -v -e '^[^[:alnum:]]' -e '^$@$$' | grep ci-runner | sed 's/^/ - /'
	@echo
	@echo "The following CI jobs are available:"
	@$(MAKE) -pRrq -f $(lastword $(MAKEFILE_LIST)) : 2>/dev/null | awk -v RS= -F: '/^# File/,/^# Finished Make data base/ {if ($$1 !~ "^[#.]") {print $$1}}' | sort | egrep -v -e '^[^[:alnum:]]' -e '^$@$$' | grep ci-job | sed 's/^/ - /'
	@echo
	@echo To run the recommended local development CI run $$(tput bold)make prepush$$(tput sgr0).
	@echo Developers are encouraged to always run this before pushing code.
	@echo
	@echo To run all possible CI run $$(tput bold)make ci-all$$(tput sgr0).
	@echo Note this may ask you to set up additional support on your machine.
	@echo To run all CI that does not require installation, use $$(tput bold)make ci-nosetup$$(tput sgr0).

# Alias the plain `ci` target to `ci-help` to help guessing users
.PHONY: ci
ci: ci-help

##
## End: user targets.
##
################################################################################
##
## Continuous Integration Targets
##

## Runners
##
## These each correspond to a 'status check' line in GitHub PR UX.
##
## These recipes *must not* contain rules, they simply collect jobs.
##
## NOTE: If you modify these, you must also modify the ci.yml CI workflow file
##       in `.github/workflows`. This *replicates* configuration in the github
##       workflow file to allow the GitHub UX to show these subtasks correctly.
.PHONY: ci-runner-github
ci-runner-github:\
	ci-runner-github-format\
	ci-runner-github-clippy\
	ci-runner-github-build\
	ci-runner-github-tests\
	ci-runner-github-qemu
	$(call banner,CI-Runner: All GitHub runners DONE)

.PHONY: ci-runner-github-format
ci-runner-github-format:\
	ci-job-format\
	ci-job-markdown-toc\
	ci-job-readme-check
	$(call banner,CI-Runner: GitHub format runner DONE)

.PHONY: ci-runner-github-clippy
ci-runner-github-clippy:\
	ci-job-clippy
	$(call banner,CI-Runner: GitHub clippy runner DONE)

.PHONY: ci-runner-github-build
ci-runner-github-build:\
	ci-job-syntax\
	ci-job-compilation\
	ci-job-msrv\
	ci-job-debug-support-targets\
	ci-job-collect-artifacts
	$(call banner,CI-Runner: GitHub build runner DONE)

.PHONY: ci-runner-github-tests
ci-runner-github-tests:\
	ci-job-libraries\
	ci-job-archs\
	ci-job-kernel\
	ci-job-capsules\
	ci-job-chips\
	ci-job-tools\
	ci-job-cargo-test-build\
	ci-job-miri # EXPERIMENTAL
	$(call banner,CI-Runner: GitHub tests runner DONE)

.PHONY: ci-runner-github-qemu
ci-runner-github-qemu:\
	ci-job-qemu
	$(call banner,CI-Runner: GitHub qemu runner DONE)


#n.b. netlify calls tools/netlify-build.sh, which is a wrapper
#     that first installs toolchains, then calls this.
.PHONY: ci-runner-netlify
ci-runner-netlify:\
	ci-job-rustdoc
	$(call banner,CI-Runner: Netlify runner DONE)


## Jobs & Setup
##
## These are the individual CI actions. These should be the smallest reasonable
## unit of execution that can run independently of other jobs.
##
## Developers **must** be able to execute `make ci-job-[...]` and have the
## status match the result of the CI infrastructure.
##
## These rules are ordered by the runners that call them.
## If rules require setup, the setup rule comes right before the job definition.
## The order of rules within a runner try to optimize for performance if
## executed in linear order.




### ci-runner-github-format jobs:
.PHONY: ci-job-format
ci-job-format: licensecheck format-check
	$(call banner,CI-Job: Format Check DONE)

define ci_setup_markdown_toc
	$(call banner,CI-Setup: Install markdown-toc)
	npm install markdown-toc
endef

.PHONY: ci-setup-markdown-toc
ci-setup-markdown-toc:
	$(call ci_setup_helper,\
		PATH="node_modules/.bin:${PATH}" command -v markdown-toc,\
		npm install markdown-toc,\
		ci_setup_markdown_toc,\
		CI_JOB_MARKDOWN)

define ci_job_markdown_toc
	$(call banner,CI-Job: Markdown Table of Contents Validation)
	@NOWARNINGS=true PATH="node_modules/.bin:${PATH}" tools/toc.sh
endef

.PHONY: ci-job-markdown-toc
ci-job-markdown-toc: ci-setup-markdown-toc
	$(if $(CI_JOB_MARKDOWN),$(call ci_job_markdown_toc))

define ci_job_readme_check
	$(call banner,CI-Job: README Validation)
	tools/check_boards_readme.py
	tools/check_capsule_readme.py
	tools/check-for-readmes.sh
endef

.PHONY: ci-job-readme-check
ci-job-readme-check:
	$(call ci_job_readme_check)



### ci-runner-github-clippy jobs:
.PHONY: ci-job-clippy
ci-job-clippy:
	$(call banner,CI-Job: Clippy)
	@cargo clippy -- -D warnings
	# Run `cargo clippy` in select boards so we run clippy with targets that
	# actually check the arch-specific functions.
	@cd boards/nordic/nrf52840dk && cargo clippy -- -D warnings
	@cd boards/hifive1 && cargo clippy -- -D warnings



### ci-runner-github-build jobs:
.PHONY: ci-job-syntax
ci-job-syntax:
	$(call banner,CI-Job: Syntax)
	@NOWARNINGS=true $(MAKE) allcheck

.PHONY: ci-job-compilation
ci-job-compilation:
	$(call banner,CI-Job: Compilation)
	@NOWARNINGS=true $(MAKE) allboards


define ci_setup_msrv
	$(call banner,CI-Setup: Install cargo-hack)
	cargo install cargo-hack
endef

.PHONY: ci-setup-msrv
ci-setup-msrv:
	$(call ci_setup_helper,\
		cargo hack -V &> /dev/null && echo yes,\
		Install 'cargo-hack' using cargo,\
		ci_setup_msrv,\
		CI_JOB_MSRV)

define ci_job_msrv
	$(call banner,CI-Job: MSRV Check)
	@cd boards/hail && cargo hack check --rust-version --target thumbv7em-none-eabihf
endef

.PHONY: ci-job-msrv
ci-job-msrv: ci-setup-msrv
	$(if $(CI_JOB_MSRV),$(call ci_job_msrv))

.PHONY: ci-job-debug-support-targets
ci-job-debug-support-targets:
	$(call banner, CI-Job: Debug Support Targets)
	# These are rules that build additional debugging information, but are
	# also quite time consuming. So we want to verify that the rules still
	# work, but don't build them for every board.
	#
	# The choice of building for the nrf52dk was chosen by random die roll.
	@NOWARNINGS=true $(MAKE) -C boards/nordic/nrf52dk lst
	@NOWARNINGS=true $(MAKE) -C boards/nordic/nrf52dk debug
	@NOWARNINGS=true $(MAKE) -C boards/nordic/nrf52dk debug-lst

.PHONY: ci-job-collect-artifacts
ci-job-collect-artifacts: ci-job-compilation
	$(call banner, CI-Job: Collect artifacts)
	# Collect binary images for each board
	#
	# This is currently used only for code size detection changes, but in
	# the future may also be used to support checks for deterministic builds.
	@rm -rf "tools/ci-artifacts"
	@mkdir tools/ci-artifacts
	@for f in $$(find target -iname '*.bin' | grep -E "release/.*\.bin");\
		do mkdir -p "tools/ci-artifacts/$$(dirname $$f)";\
		cp "$$f" "tools/ci-artifacts/$$f";\
		done



### ci-runner-github-tests jobs:
.PHONY: ci-job-libraries
ci-job-libraries:
	$(call banner,CI-Job: Libraries)
	@cd libraries/enum_primitive && NOWARNINGS=true RUSTFLAGS="-D warnings" cargo test
	@cd libraries/riscv-csr && NOWARNINGS=true RUSTFLAGS="-D warnings" cargo test
	@cd libraries/tock-cells && NOWARNINGS=true RUSTFLAGS="-D warnings" cargo test
	@cd libraries/tock-register-interface && NOWARNINGS=true RUSTFLAGS="-D warnings" cargo test
	@cd libraries/tickv && NOWARNINGS=true RUSTFLAGS="-D warnings" cargo test

.PHONY: ci-job-archs
ci-job-archs:
	$(call banner,CI-Job: Archs)
	@for arch in `./tools/list_archs.sh`;\
		do echo "$$(tput bold)Test $$arch";\
		cd arch/$$arch;\
		NOWARNINGS=true RUSTFLAGS="-D warnings" TOCK_KERNEL_VERSION=ci_test cargo test || exit 1;\
		cd ../..;\
		done

.PHONY: ci-job-kernel
ci-job-kernel:
	$(call banner,CI-Job: Kernel)
	@cd kernel && NOWARNINGS=true RUSTFLAGS="-D warnings" TOCK_KERNEL_VERSION=ci_test cargo test

.PHONY: ci-job-capsules
ci-job-capsules:
	$(call banner,CI-Job: Capsules)
	@# Capsule initialization depends on board/chip specific imports, so ignore doc tests
	@cd capsules/core && NOWARNINGS=true RUSTFLAGS="-D warnings" TOCK_KERNEL_VERSION=ci_test cargo test
	@cd capsules/extra && NOWARNINGS=true RUSTFLAGS="-D warnings" TOCK_KERNEL_VERSION=ci_test cargo test
	@cd capsules/system && NOWARNINGS=true RUSTFLAGS="-D warnings" TOCK_KERNEL_VERSION=ci_test cargo test

.PHONY: ci-job-chips
ci-job-chips:
	$(call banner,CI-Job: Chips)
	@for chip in `./tools/list_chips.sh`;\
		do echo "$$(tput bold)Test $$chip";\
		cd chips/$$chip;\
		NOWARNINGS=true RUSTFLAGS="-D warnings" TOCK_KERNEL_VERSION=ci_test cargo test || exit 1;\
		cd ../..;\
		done

define ci_setup_tools
	$(call banner,CI-Setup: Install support for 'tools' checks)
	@if command -v apt-get > /dev/null; then\
		echo "Running: sudo apt-get install libusb-1.0.0-dev";\
		sudo apt-get install libusb-1.0.0-dev;\
	elif command -v brew > /dev/null; then\
		echo "Running: brew install libusb-compat pkg-config";\
		brew install libusb-compat pkg-config;\
	elif command -v dnf > /dev/null; then\
		echo "Running: sudo dnf install libusb-devel";\
		sudo dnf install libusb-devel;\
	else\
		echo "";\
		echo "ERR: Do not know how to install libusb on this platform.";\
		exit 1;\
	fi
endef

.PHONY: ci-setup-tools
ci-setup-tools:
	$(call ci_setup_helper,\
		pkg-config --cflags --libs libusb &> /dev/null && echo yes,\
		Install 'libusb' for development using your package manager,\
		ci_setup_tools,\
		CI_JOB_TOOLS)

define ci_job_tools
	$(call banner,CI-Job: Tools)
	@NOWARNINGS=true RUSTFLAGS="-D warnings" \
		cargo test --all-targets --manifest-path=tools/Cargo.toml --workspace || exit 1
endef

.PHONY: ci-job-tools
ci-job-tools: ci-setup-tools
	$(if $(CI_JOB_TOOLS),$(call ci_job_tools))


.PHONY: ci-job-miri
ci-job-miri:
	$(call banner,CI-Job: Miri)
	#
	# Note: This is highly experimental and limited at the moment.
	#
	@# Hangs forever during `Building` for this one :shrug:
	@#cd libraries/tock-register-interface && NOWARNINGS=true cargo miri test
	@cd kernel && NOWARNINGS=true cargo miri test
	@for a in $$(tools/list_archs.sh); do cd arch/$$a && NOWARNINGS=true cargo miri test && cd ../..; done
	@cd capsules/core && NOWARNINGS=true cargo miri test
	@cd capsules/extra && NOWARNINGS=true cargo miri test
	@cd capsules/system && NOWARNINGS=true cargo miri test
	@for c in $$(tools/list_chips.sh); do cd chips/$$c && NOWARNINGS=true cargo miri test && cd ../..; done


.PHONY: ci-job-cargo-test-build
ci-job-cargo-test-build:
	@$(MAKE) NO_RUN="--no-run" -C "boards/opentitan/earlgrey-cw310" test
	@$(MAKE) NO_RUN="--no-run" -C "boards/esp32-c3-devkitM-1" test
	@$(MAKE) NO_RUN="--no-run" -C "boards/apollo3/lora_things_plus" test
	@$(MAKE) NO_RUN="--no-run" -C "boards/apollo3/lora_things_plus" test-atecc508a
	@$(MAKE) NO_RUN="--no-run" -C "boards/apollo3/lora_things_plus" test-chirp_i2c_moisture
	@$(MAKE) NO_RUN="--no-run" -C "boards/apollo3/redboard_artemis_atp" test
	@$(MAKE) NO_RUN="--no-run" -C "boards/apollo3/redboard_artemis_nano" test



### ci-runner-github-qemu jobs:
QEMU_COMMIT_HASH=abb1565d3d863cf210f18f70c4a42b0f39b8ccdb
define ci_setup_qemu_riscv
	$(call banner,CI-Setup: Build QEMU)
	@# Use the latest QEMU as it has OpenTitan support
	@printf "Building QEMU, this could take a few minutes\n\n"
	@git clone https://github.com/qemu/qemu ./tools/qemu 2>/dev/null || echo "qemu already cloned, checking out"
	@cd tools/qemu; git checkout ${QEMU_COMMIT_HASH}; ../qemu/configure --target-list=riscv32-softmmu --disable-linux-io-uring --disable-libdaxctl;
	@# Build qemu
	@$(MAKE) -C "tools/qemu/build" -j2 || (echo "You might need to install some missing packages" || exit 127)
endef

.PHONY: ci-setup-qemu
ci-setup-qemu:
	$(call ci_setup_helper,\
		[[ $$(git -C ./tools/qemu rev-parse HEAD 2>/dev/null || echo 0) == "${QEMU_COMMIT_HASH}" ]] && \
			cd tools/qemu/build && make -q riscv32-softmmu && echo yes,\
		Clone QEMU and run its build scripts,\
		ci_setup_qemu_riscv,\
		CI_JOB_QEMU_RISCV)
	$(if $(CI_JOB_QEMU_RISCV),$(eval CI_JOB_QEMU := true))

define ci_job_qemu
	$(call banner,CI-Job: QEMU)
	@cd tools/qemu-runner;\
		PATH="$(shell pwd)/tools/qemu/build/:${PATH}"\
		NOWARNINGS=true cargo run
	@cd boards/opentitan/earlgrey-cw310;\
		PATH="$(shell pwd)/tools/qemu/build/:${PATH}"\
		make test
endef

.PHONY: ci-job-qemu
ci-job-qemu: ci-setup-qemu
	$(if $(CI_JOB_QEMU),$(call ci_job_qemu))



### ci-runner-netlify jobs:
.PHONY: ci-job-rustdoc
ci-job-rustdoc:
	$(call banner,CI-Job: Rustdoc Documentation)
	@NOWARNINGS=true tools/build-all-docs.sh

## End CI rules
##
################################################################################

.PHONY: board-release-test
board-release-test:
	@cd tools/board-runner;\
		cargo run ${TARGET}
