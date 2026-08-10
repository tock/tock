#!/usr/bin/env bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

# Ask rustup to pick the latest version that will work.
# This requires rustup >= 1.20.0.
echo "Updating rustc to latest compatible version..."
rustup toolchain install nightly --allow-downgrade --component cargo --component clippy --component llvm-tools --component miri --component rust-analysis --component rust-docs --component rust-src --component rust-std --component rustc --component rustfmt

# # Rerun the command so that it prints out the version it installed. We then have
# # to extract that from the output. If there is a better way to do this then we
# # should update this.
# RUSTUP_NIGHTLY_VERSION=`rustup update nightly 2>&1`
# BEST_DATE=`echo $RUSTUP_NIGHTLY_VERSION | sed 's/.* \([0-9]*-[0-9]*-[0-9]*\).*/\1/g'`

# I just do not know how to get rustup to tell us the version of the toolchain
# it decided on with the format required for `rust-toolchain`. That the dates
# are off-by-one day is annoying. I'm resorting to just asking the user.


echo "Please enter the version of Rust to use."
echo "It is probably just one day later than whatever was printed out above."
echo ""
read -p "Date string: " BEST_DATE

# Nightly version string
NIGHTLY=nightly-$BEST_DATE

echo Updating Rust to $NIGHTLY

# Update all relevant files with the new version string.
# Note, x-platform `sed -i` has odd, but particular syntax
# https://stackoverflow.com/questions/5694228/sed-in-place-flag-that-works-both-on-mac-bsd-and-linux
sed -i._SED_HACK "s/nightly-[0-9]*-[0-9]*-[0-9]*/${NIGHTLY}/g" rust-toolchain.toml
sed -i._SED_HACK "s/nightly-[0-9]*-[0-9]*-[0-9]*/${NIGHTLY}/g" .vscode/settings.json
sed -i._SED_HACK "s/nightly-[0-9]*-[0-9]*-[0-9]*/${NIGHTLY}/g" doc/Getting_Started.md

find . -name '*._SED_HACK' -delete

# Update custom RISC-V target specs to match the new toolchain.
#
# Tock uses custom target JSON specs (boards/cargo/riscv*.json) rather than
# the built-in RISC-V targets so that the +relax feature (which enables LLVM
# to emit relaxable relocations for linker relaxation, providing significant
# code size savings) can be included in the base target definition. Features
# specified via -Ctarget-feature trigger an "unstable feature" warning for
# +relax because rustc has not yet stabilized it; features in the target spec
# itself do not. There is no active upstream stabilization effort as of 2026;
# track progress at https://github.com/rust-lang/rust/issues/150257.
#
# These JSON files are derived from `rustc --print target-spec-json` with
# +relax appended to features and the metadata block stripped; they must be
# kept in sync when the toolchain changes.
echo "Checking RISC-V custom target specs..."
for TARGET in riscv32imc-unknown-none-elf riscv32imac-unknown-none-elf riscv64imac-unknown-none-elf; do
    JSON="boards/cargo/${TARGET}.json"
    [ -f "$JSON" ] || continue

    # rust-toolchain.toml was already updated to $NIGHTLY above, so plain
    # `rustc` here picks up the new toolchain via rustup's toolchain-file
    # override, installing it first if needed.
    UPSTREAM_FILE=$(mktemp)
    rustc -Z unstable-options --print target-spec-json --target "$TARGET" > "$UPSTREAM_FILE"
    if [ ! -s "$UPSTREAM_FILE" ]; then
        echo "  ${TARGET}: rustc produced no target spec, aborting" >&2
        rm "$UPSTREAM_FILE"
        exit 1
    fi

    python3 - "$JSON" "$TARGET" "$UPSTREAM_FILE" <<'PYEOF'
import json, sys

with open(sys.argv[3]) as f:
    upstream = json.load(f)
upstream['features'] = upstream.get('features', '') + ',+relax'
upstream.pop('metadata', None)

path = sys.argv[1]
with open(path) as f:
    current = json.load(f)

if current != upstream:
    print(f"  {sys.argv[2]}: spec changed, updating")
    with open(path, 'w') as f:
        json.dump(upstream, f, indent=2)
        f.write('\n')
else:
    print(f"  {sys.argv[2]}: up to date")
PYEOF
    rm "$UPSTREAM_FILE"
done
