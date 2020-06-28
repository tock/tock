# For more information on Tock's make system and the CI setup, see the docs at
# https://github.com/tock/tock/tree/master/doc/CodeReview.md#3-continuous-integration

################################################################################
##
## Interal support that needs to run first
##

# First, need to fill out some variables that the Makefile will use
$(eval ALL_BOARDS := $(shell ./tools/list_boards.sh))

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
	@echo "            list: Lists available boards"
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
	@# First, if the dependency is installed, we can bail early
	$(if $(shell bash -c '$(1)'),$(eval $(guard_variable) := true),
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
	@cargo check

.PHONY: alldoc
alldoc:
	@for f in $(ALL_BOARDS);\
		do echo "$$(tput bold)Documenting $$f";\
		$(MAKE) -C "boards/$$f" doc || exit 1;\
		done


## Commands
.PHONY: clean
clean:
	@echo "$$(tput bold)Clean top-level Cargo workspace" && cargo clean
	@for f in `./tools/list_tools.sh`;\
		do echo "$$(tput bold)Clean tools/$$f";\
		cargo clean --manifest-path "tools/$$f/Cargo.toml" || exit 1;\
		done
	@echo "$$(tput bold)Clean rustdoc" && rm -rf doc/rustdoc
	@echo "$$(tput bold)Clean ci-artifacts" && rm -rf tools/ci-artifacts

.PHONY: fmt format
fmt format: tools/.format_fresh
	$(call banner,Formatting complete)

# Get a list of all rust source files (everything fmt operates on)
$(eval RUST_FILES_IN_TREE := $(shell (git ls-files | grep '\.rs$$') || find . -type f -name '*.rs'))
tools/.format_fresh: $(RUST_FILES_IN_TREE)
	@./tools/run_cargo_fmt.sh $(TOCK_FORMAT_MODE)
	@touch tools/.format_fresh

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

# Run all possible CI. If this passses locally, all cloud CI *must* pass as well.
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
	format\
	ci-job-syntax\
	ci-job-clippy
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
	ci-job-capsules\
	ci-job-chips\
	ci-job-tools\
	ci-job-miri
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
	@CI=true TOCK_FORMAT_MODE=diff $(MAKE) format

.PHONY: ci-job-clippy
ci-job-clippy:
	$(call banner,CI-Job: Clippy)
	@CI=true ./tools/run_clippy.sh

define ci_setup_markdown_toc
	$(call banner,CI-Setup: Install markdown-toc)
	npm install -g markdown-toc
endef

.PHONY: ci-setup-markdown-toc
ci-setup-markdown-toc:
	$(call ci_setup_helper,\
		command -v markdown-toc,\
		npm install -g markdown-toc,\
		ci_setup_markdown_toc,\
		CI_JOB_MARKDOWN)

define ci_job_markdown_toc
	$(call banner,CI-Job: Markdown Table of Contents Validation)
	@CI=true tools/toc.sh
endef

.PHONY: ci-job-markdown-toc
ci-job-markdown-toc: ci-setup-markdown-toc
	$(if $(CI_JOB_MARKDOWN),$(call ci_job_markdown_toc))



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

.PHONY: ci-job-capsules
ci-job-capsules:
	$(call banner,CI-Job: Capsules)
	@# Capsule initialization depends on board/chip specific imports, so ignore doc tests
	@cd capsules && CI=true RUSTFLAGS="-D warnings" TOCK_KERNEL_VERSION=ci_test cargo test --lib

.PHONY: ci-job-chips
ci-job-chips:
	$(call banner,CI-Job: Chips)
	@for chip in `./tools/list_chips.sh`;\
		do echo "$$(tput bold)Test $$chip";\
		cd chips/$$chip;\
		CI=true RUSTFLAGS="-D warnings" TOCK_KERNEL_VERSION=ci_test cargo test || exit 1;\
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
	@for tool in `./tools/list_tools.sh`;\
		do echo "$$(tput bold)Build & Test $$tool";\
		cd tools/$$tool;\
		CI=true RUSTFLAGS="-D warnings" cargo build --all-targets || exit 1;\
		cd - > /dev/null;\
		done
endef

.PHONY: ci-job-tools
ci-job-tools: ci-setup-tools
	$(if $(CI_JOB_TOOLS),$(call ci_job_tools))


.PHONY: ci-setup-miri
ci-setup-miri:
	@rustup component list | grep miri | grep -q installed || rustup component add miri

.PHONY: ci-job-miri
ci-job-miri: ci-setup-miri
	$(call banner,CI-Job: Miri)
	#
	# Note: This is highly experimental and limited at the moment.
	#
	@# Hangs forever during `Building` for this one :shrug:
	@#cd libraries/tock-register-interface && CI=true cargo miri test
	@cd kernel && CI=true cargo miri test
	@for a in $$(tools/list_archs.sh); do cd arch/$$a && CI=true cargo miri test && cd ../..; done
	@cd capsules && CI=true cargo miri test
	@for c in $$(tools/list_chips.sh); do cd chips/$$c && CI=true cargo miri test && cd ../..; done


### ci-runner-github-qemu jobs:

define ci_setup_qemu_riscv
	$(call banner,CI-Setup: Install Tock QEMU port)
	@# Use the latest QEMU as it has OpenTitan support
	@printf "Building QEMU, this could take a few minutes\n\n"
	# Download Tock qemu fork if needed
	if ! bash -c 'cd tools/qemu && [[ $$(git rev-parse --short HEAD) == "7ff5b84" ]]'; then \
		rm -rf tools/qemu; \
		cd tools; git clone https://github.com/alistair23/qemu.git --depth 1 -b riscv-tock.next; \
		cd qemu; ./configure --target-list=riscv32-softmmu; \
	fi
	# Build qemu
	@$(MAKE) -C "tools/qemu" || (echo "You might need to install some missing packages" || exit 127)
endef

define ci_setup_qemu_opentitan
	$(call banner,CI-Setup: Get OpenTitan boot ROM image)
	# Download OpenTitan image
	@printf "Downloading OpenTitan boot rom from: 1beb08b474790d4b6c67ae5b3423e2e8dfc9e368\n"
	@pwd=$$(pwd) && \
		temp=$$(mktemp -d)\
		cd $$temp && \
		curl $$(curl "https://dev.azure.com/lowrisc/opentitan/_apis/build/builds/14991/artifacts?artifactName=opentitan-dist&api-version=5.1" | cut -d \" -f 38) --output opentitan-dist.zip; \
		unzip opentitan-dist.zip; \
		tar -xf opentitan-dist/opentitan-snapshot-20191101-*.tar.xz; \
		mv opentitan-snapshot-20191101-*/sw/device/boot_rom/boot_rom_fpga_nexysvideo.elf $$pwd/tools/qemu-runner/opentitan-boot-rom.elf
endef

.PHONY: ci-setup-qemu
ci-setup-qemu:
	$(call ci_setup_helper,\
		cd tools/qemu && [[ $$(git rev-parse --short HEAD) == "1ef6d40" ]] && [ -x riscv32-softmmu ] && echo yes,\
		Clone QEMU fork (with riscv fixes) and run its build scripts,\
		ci_setup_qemu_riscv,\
		CI_JOB_QEMU_RISCV)
	$(call ci_setup_helper,\
		[[ $$(cksum tools/qemu-runner/opentitan-boot-rom.elf | cut -d" " -f1) == "2835238144" ]] && echo yes,\
		Download opentitan archive and unpack a ROM image,\
		ci_setup_qemu_opentitan,\
		CI_JOB_QEMU_OPENTITAN)
	$(if $(CI_JOB_QEMU_RISCV),$(if $(CI_JOB_QEMU_OPENTITAN),$(eval CI_JOB_QEMU := true)))



define ci_job_qemu
	$(call banner,CI-Job: QEMU)
	@cd tools/qemu-runner;\
		PATH="$(shell pwd)/tools/qemu/riscv32-softmmu/:${PATH}"\
		CI=true cargo run
endef

.PHONY: ci-job-qemu
ci-job-qemu: ci-setup-qemu
	$(if $(CI_JOB_QEMU),$(call ci_job_qemu))



### ci-runner-netlify jobs:
.PHONY: ci-job-rustdoc
ci-job-rustdoc:
	$(call banner,CI-Job: Rustdoc Documentation)
	@CI=true tools/build-all-docs.sh

## End CI rules
##
###################################################################

