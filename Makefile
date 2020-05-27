# For more information on Tock's make system and the CI setup, see the docs at
# https://github.com/tock/tock/tree/master/doc/CodeReview.md#3-continuous-integration

################################################################################
##
## Interal support that needs to run first
##

# First, need to fill out some variables that the Makefile will use
$(eval ALL_BOARDS := $(shell ./tools/list_boards.sh))
$(eval PLATFORM := $(shell uname -s))

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
	@echo "Run 'make' in a board directory to build Tock for that board,"
	@echo "and usually 'make program' or 'make flash' to load Tock onto hardware."
	@echo "Check out the README in your board's folder for more information."
	@echo
	@echo "This root Makefile has a few useful targets as well:"
	@echo "        allaudit: Audit Cargo dependencies for all kernel sources"
	@echo "       allboards: Compiles Tock for all supported boards"
	@echo "        allcheck: Checks, but does not compile, Tock for all supported boards"
	@echo "          alldoc: Builds Tock documentation for all boards"
	@echo "           clean: Clean all builds"
	@echo "          format: Runs the rustfmt tool on all kernel sources"
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

##
## End: functions.
##
################################################################################
##
## User convenience targets
##

# Aggregate targets
.PHONY: allaudit
audit:
	@for f in `./tools/list_lock.sh`;\
		do echo "$$(tput bold)Auditing $$f";\
		(cd "$$f" && cargo audit || exit 1);\
		done

.PHONY: allboards
allboards:
	@for f in $(ALL_BOARDS);\
		do echo "$$(tput bold)Build $$f";\
		$(MAKE) -C "boards/$$f" || exit 1;\
		done

.PHONY: allcheck
allcheck:
	@for f in $(ALL_BOARDS);\
		do echo "$$(tput bold)Check $$f";\
		$(MAKE) -C "boards/$$f" check || exit 1;\
		done

.PHONY: alldoc
alldoc:
	@for f in $(ALL_BOARDS);\
		do echo "$$(tput bold)Documenting $$f";\
		$(MAKE) -C "boards/$$f" doc || exit 1;\
		done


# Commands
.PHONY: fmt format
fmt format:
	@./tools/run_cargo_fmt.sh

.PHONY: clean
clean:
	@echo "$$(tput bold)Clean top-level Cargo workspace" && cargo clean
	@for f in `./tools/list_tools.sh`;\
		do echo "$$(tput bold)Clean tools/$$f";\
		cargo clean --manifest-path "tools/$$f/Cargo.toml" || exit 1;\
		done
	@echo "$$(tput bold)Clean rustdoc" && rm -rf doc/rustdoc
	@echo "$$(tput bold)Clean ci-artifacts" && rm -rf ./ci-artifacts


## Meta-Targets

# Run all possible CI. If this passses locally, all cloud CI *must* pass as well.
.PHONY: ci-all
ci-all:\
	ci-runner-github\
	ci-runner-netlify

# Run the fast jobs.
# This is designed for developers, to be run often and before submitting code upstream.
.PHONY: prepush
prepush:\
	ci-job-format\
	ci-job-syntax\
	ci-job-clippy
	$(call banner,Pre-Push checks all passed!)
	# Note: Tock runs additional and more intense CI checks on all PRs.
	# If one of these error, you can run `make ci-job-NAME` to test locally.


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

# n.b. This *replicates* configuration in the github workflow file
# to allow the GitHub UX to show these subtasks correctly
.PHONY: ci-runner-github
ci-runner-github:\
	ci-runner-github-format\
	ci-runner-github-build\
	ci-runner-github-tests\
	ci-runner-github-qemu
	$(call banner,CI-Runner: All GitHub runners DONE)

.PHONY: ci-runner-github-format
ci-runner-github-format:\
	ci-job-format\
	ci-job-clippy\
	ci-job-markdown-toc
	$(call banner,CI-Runner: GitHub format runner DONE)

.PHONY: ci-runner-github-build
ci-runner-github-build:\
	ci-job-syntax\
	ci-job-compilation\
	ci-job-debug-support-targets\
	ci-job-collect-artifacts
	$(call banner,CI-Runner: GitHub build runner DONE)

.PHONY: ci-runner-github-tests
ci-runner-github-tests:\
	ci-job-libraries\
	ci-job-archs\
	ci-job-kernel\
	ci-job-chips\
	ci-job-tools
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
ci-job-format:
	$(call banner,CI-Job: Format Check)
	@CI=true ./tools/run_cargo_fmt.sh diff

.PHONY: ci-job-clippy
ci-job-clippy:
	$(call banner,CI-Job: Clippy)
	@CI=true ./tools/run_clippy.sh

.PHONY: ci-setup-markdown-toc
ci-setup-markdown-toc:
ifdef CI
	npm install -g markdown-toc
else
	@command -v markdown-toc > /dev/null || (\
		printf "\n$$(tput bold)Missing Dependency$$(tput sgr0)";\
		printf "\n";\
		printf "Need to install 'markdown-toc' on your system.\n";\
		printf "\n";\
		printf "This is easiest installed globally using npm:\n";\
		printf "  npm install -g markdown-toc\n"\
		exit 1)
endif

.PHONY: ci-job-markdown-toc
ci-job-markdown-toc: ci-setup-markdown-toc
	$(call banner,CI-Job: Markdown Table of Contents Validation)
	@CI=true tools/toc.sh



### ci-runner-github-build jobs:
.PHONY: ci-job-syntax
ci-job-syntax:
	$(call banner,CI-Job: Syntax)
	@CI=true $(MAKE) allcheck

.PHONY: ci-job-compilation
ci-job-compilation:
	$(call banner,CI-Job: Compilation)
	@CI=true $(MAKE) allboards

.PHONY: ci-job-debug-support-targets
ci-job-debug-support-targets:
	$(call banner, CI-Job: Debug Support Targets)
	# These are rules that build additional debugging information, but are
	# also quite time consuming. So we want to verify that the rules still
	# work, but don't build them for every board.
	#
	# The choice of building for the nrf52dk was chosen by random die roll.
	@CI=true $(MAKE) -C boards/nordic/nrf52dk lst
	@CI=true $(MAKE) -C boards/nordic/nrf52dk debug
	@CI=true $(MAKE) -C boards/nordic/nrf52dk debug-lst

.PHONY: ci-job-collect-artifacts
ci-job-collect-artifacts: ci-job-compilation
	# Collect binary images for each board
	#
	# This is currently used only for code size detection changes, but in
	# the future may also be used to support checks for deterministic builds.
	@mkdir -p ./ci-artifacts
	@rm -rf "./ci-artifacts/*"
	@for f in $$(find ./target -iname '*.bin' | grep -E "release/.*\.bin");\
		do mkdir -p "ci-artifacts/$$(dirname $$f)";\
		cp "$$f" "ci-artifacts/$$f";\
		done



### ci-runner-github-tests jobs:
.PHONY: ci-job-libraries
ci-job-libraries:
	$(call banner,CI-Job: Libraries)
	@cd libraries/enum_primitive && CI=true RUSTFLAGS="-D warnings" cargo test
	@cd libraries/riscv-csr && CI=true RUSTFLAGS="-D warnings" cargo test
	@cd libraries/tock-cells && CI=true RUSTFLAGS="-D warnings" cargo test
	@cd libraries/tock-register-interface && CI=true RUSTFLAGS="-D warnings" cargo test
	@cd libraries/tock-rt0 && CI=true RUSTFLAGS="-D warnings" cargo test

.PHONY: ci-job-archs
ci-job-archs:
	$(call banner,CI-Job: Archs)
	@for arch in `./tools/list_archs.sh`;\
		do echo "$$(tput bold)Test $$arch";\
		cd arch/$$arch;\
		CI=true RUSTFLAGS="-D warnings" TOCK_KERNEL_VERSION=ci_test cargo test || exit 1;\
		cd ../..;\
		done

.PHONY: ci-job-kernel
ci-job-kernel:
	$(call banner,CI-Job: Kernel)
	@cd kernel && CI=true RUSTFLAGS="-D warnings" TOCK_KERNEL_VERSION=ci_test cargo test

.PHONY: ci-job-chips
ci-job-chips:
	$(call banner,CI-Job: Chips)
	@for chip in `./tools/list_chips.sh`;\
		do echo "$$(tput bold)Test $$chip";\
		cd chips/$$chip;\
		CI=true RUSTFLAGS="-D warnings" TOCK_KERNEL_VERSION=ci_test cargo test || exit 1;\
		cd ../..;\
		done

.PHONY: ci-setup-tools
ci-setup-tools:
ifdef CI
ifeq ($(PLATFORM),Linux)
	sudo apt-get install libusb-1.0.0-dev
else ifeq ($(PLATFORM),Darwin)
	brew install libusb-compat pkg-config
else
	$(error CI on unsupported platform.)
endif
else
	@pkg-config --cflags --libs libusb > /dev/null || (\
		printf "\n$$(tput bold)Missing Dependency$$(tput sgr0)";\
		printf "\n";\
		printf "Need to install 'libusb' for development on your system.\n";\
		printf "  - Debian: sudo apt-get install libusb-1.0.0-dev\n";\
		printf "  - Darwin: brew install libusb-compat pkg-config\n";\
		exit 1)
endif

.PHONY: ci-job-tools
ci-job-tools: ci-setup-tools
	$(call banner,CI-Job: Tools)
	@for tool in `./tools/list_tools.sh`;\
		do echo "$$(tput bold)Build & Test $$tool";\
		cd tools/$$tool;\
		CI=true RUSTFLAGS="-D warnings" cargo build --all-targets || exit 1;\
		cd - > /dev/null;\
		done



### ci-runner-github-qemu jobs:

# ci-setup-qemu uses make as intended a bit to get all the needed parts
.PHONY: ci-setup-qemu
ci-setup-qemu: tools/qemu/riscv32-softmmu tools/qemu-runner/opentitan-boot-rom.elf

tools/qemu/riscv32-softmmu:
	@# Use the latest QEMU as it has OpenTitan support
	@printf "Building QEMU, this could take a few minutes\n\n"
	# Download Tock qemu fork if needed
	@if [[ ! -d tools/qemu || ! -f tools/qemu/VERSION ]]; then \
		rm -rf tools/qemu; \
		cd tools; git clone https://github.com/alistair23/qemu.git --depth 1 -b riscv-tock.next; \
		cd qemu; ./configure --target-list=riscv32-softmmu; \
	fi
	# Build qemu
	@$(MAKE) -C "tools/qemu" || (echo "You might need to install some missing packages" || exit 127)

tools/qemu-runner/opentitan-boot-rom.elf:
	# Download OpenTitan image
	@printf "Downloading OpenTitan boot rom from: 2aedf641120665b91c3a5d5aa214175d09f71ee6\n"
	@pwd=$$(pwd) && \
		temp=$$(mktemp -d)\
		cd $$temp && \
		curl $$(curl "https://dev.azure.com/lowrisc/opentitan/_apis/build/builds/13066/artifacts?artifactName=opentitan-dist&api-version=5.1" | cut -d \" -f 38) --output opentitan-dist.zip; \
		unzip opentitan-dist.zip; \
		tar -xf opentitan-dist/opentitan-snapshot-20191101-*.tar.xz; \
		mv opentitan-snapshot-20191101-*/sw/device/boot_rom/boot_rom_fpga_nexysvideo.elf $$pwd/tools/qemu-runner/opentitan-boot-rom.elf


.PHONY: ci-job-qemu
ci-job-qemu: ci-setup-qemu
	$(call banner,CI-Job: QEMU)
	@cd tools/qemu-runner;\
		PATH="$(shell pwd)/tools/qemu/riscv32-softmmu/:${PATH}"\
		CI=true cargo run



### ci-runner-netlify jobs:
.PHONY: ci-job-rustdoc
ci-job-rustdoc:
	$(call banner,CI-Job: Rustdoc Documentation)
	@CI=true tools/build-all-docs.sh

## End CI rules
##
###################################################################

