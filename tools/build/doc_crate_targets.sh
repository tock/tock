# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

# Shared by build_all_docs.sh and check_docs.sh: which arch/chip crates
# have no host-target implementation (no `doc`-mock cfg branch to fall
# back to), and what real target triple to document each one against.
#
# Target picked by: which board uses the crate, and what target that
# board builds for (boards/*/.cargo/config.toml).
#
# Maintained by hand: add an entry whenever a new arch or chip crate with
# no host-target implementation is added.
#
# Each entry is "crate:target-triple". A plain indexed array (rather than
# an associative array, i.e. `declare -A`) is used deliberately: macOS
# ships bash 3.2, which predates associative arrays entirely.
#
# `x86` is the one entry using a path to a custom JSON target spec instead
# of a builtin triple (no builtin i486 target exists); see the `.json`
# branch wherever this array is consumed for what that requires.
CRATE_TARGETS=(
    "cortexm0:thumbv6m-none-eabi"
    "cortexm0p:thumbv6m-none-eabi"
    "apollo3:thumbv7em-none-eabi"
    "arty_e21_chip:riscv32imac-unknown-none-elf"
    "earlgrey:riscv32imc-unknown-none-elf"
    "esp32-c3:riscv32imc-unknown-none-elf"
    "litex_vexriscv:riscv32imc-unknown-none-elf"
    "rp2040:thumbv6m-none-eabi"
    "rp2350:thumbv8m.main-none-eabi"
    "stm32f4xx:thumbv7em-none-eabi"
    "x86:boards/qemu_i486_q35/i486-unknown-none.json"
)

# Crates whose dependent boards/chips actually build for more than one real
# target -- `riscv` (and its riscv-csr dependency) serve both riscv32 and
# riscv64 boards; `cortexm` (and its cortexv7m companion) serve both
# thumbv7em and thumbv8m.main boards. Picking a single target here is a
# deliberate, somewhat arbitrary call, not a fact about the crate the way
# CRATE_TARGETS' entries are -- called out separately so that's obvious at
# a glance. Folded into CRATE_TARGETS below; nothing past this point
# distinguishes the two.
SHARED_CRATE_TARGETS=(
    "riscv:riscv32imac-unknown-none-elf"
    "riscv-csr:riscv32imac-unknown-none-elf"
    "cortexm:thumbv7em-none-eabi"
    "cortexv7m:thumbv7em-none-eabi"
)
CRATE_TARGETS+=("${SHARED_CRATE_TARGETS[@]}")

# cargo/rustdoc name a target's output directory after its triple, or (for
# a `--target path/to/name.json`) after just `name` -- strip any leading
# path and `.json` suffix to get that directory name from a CRATE_TARGETS
# target value.
target_dir_name() {
    local t="${1##*/}"
    echo "${t%.json}"
}
