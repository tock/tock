#!/usr/bin/env bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

# Builds all of the board documentation into doc/rustdoc.
#
# This relies on rustdoc's unstable cross-crate-info merging mechanism
# (RFC 3662, "Mergeable rustdoc cross-crate info": --write-doc-meta-dir /
# --read-doc-meta-dir, gated behind -Z unstable-options). The actual merge
# logic runs inside rustdoc itself -- this script never parses or rewrites
# rustdoc's own output format -- but the flags it's built on are still
# actively changing shape, not close to stabilizing as of Aug 2026.
#   - rust-lang/rust#130676 is the tracking issue.
#   - rust-lang/rust#152902, the stabilization PR.

set -e

# Delete any old docs
rm -rf doc/rustdoc

# Use copy-on-write cp if available
touch _COW
if `cp -c _COW _COW2 2> /dev/null`; then
    # BSD (OS X) default
    CP_COW="cp -c"
elif `cp --reflink=auto _COW _COW2 2> /dev/null`; then
    # Coreutils (unix) default
    CP_COW="cp --reflink=auto"
else
    echo "$(tput bold)Warn: No copy-on-write cp available. Doc build will be slower.$(tput sgr0)"
    CP_COW="cp"
fi
rm -f _COW _COW2

# arch/* and select chips/* crates have no host-target implementation (no
# `doc`-mock cfg branch to fall back to), so they're built here against
# their real target triple and merged into the unified doc tree via
# rustdoc's --write-doc-meta-dir/--read-doc-meta-dir.
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
# branch in the per-crate build loop below for what that requires.
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

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
META="$WORK/meta"
mkdir -p "$META/host"

# 1. Host pass: every crate not in CRATE_TARGETS, plus dependencies.
# RUSTDOCFLAGS makes each crate write its own non-colliding <crate>.json
# into $META/host. `--no-deps` skips generating docs for third-party
# dependencies, which takes a long time and we don't link to anyway.
CARGO_TARGET_DIR="$WORK" \
    RUSTDOCFLAGS="${RUSTDOCFLAGS:-} -Z unstable-options --write-doc-meta-dir=$META/host" \
    cargo doc --no-deps

# CRATE_TARGETS crates get pulled into the host pass too, as dependencies
# of the boards/chips that use them. Prune their host-pass meta so the
# finalize step only sees each crate's dedicated real-target meta.
for entry in "${CRATE_TARGETS[@]}"; do
    crate="${entry%%:*}"
    rm -f "$META/host/${crate//-/_}.json"
done

# 2. One independent real-target pass per CRATE_TARGETS crate. No
# --read-doc-meta-dir: only each build's own <crate>/ and src/<crate>/
# output is used later, the rest is discarded.
for entry in "${CRATE_TARGETS[@]}"; do
    crate="${entry%%:*}"
    target="${entry#*:}"
    echo "--- documenting $crate for $target ---"
    if [ "${target%.json}" != "$target" ]; then
        # A custom JSON target spec (no builtin triple exists) needs
        # cargo's own -Z json-target-spec just to accept a `--target
        # *.json` path, and -Z build-std since custom targets have no
        # prebuilt std/core component to fall back on. All -Z flags have
        # to be grouped together before the subcommand for cargo to
        # accept them.
        CARGO_TARGET_DIR="$WORK" \
            cargo -Z json-target-spec -Z build-std=core,compiler_builtins -Z unstable-options \
            rustdoc -p "$crate" --target "$target" -- \
            --write-doc-meta-dir="$META/$crate"
    else
        CARGO_TARGET_DIR="$WORK" \
            cargo rustdoc -p "$crate" --target "$target" -- \
            -Z unstable-options --write-doc-meta-dir="$META/$crate"
    fi
done

# 3. Finalize: a single bare `rustdoc` invocation (no cargo, and no
# crate source -- finalize mode doesn't take any, see the note up top)
# reads every meta directory written above and writes the fully-merged
# crates.js/trait.impl/search.index/src-files.js/static.files to its own
# --out-dir.
FINAL_DIR="$WORK/final"
read_flags=(--read-doc-meta-dir="$META/host")
for entry in "${CRATE_TARGETS[@]}"; do
    crate="${entry%%:*}"
    read_flags+=(--read-doc-meta-dir="$META/$crate")
done
echo "--- finalizing merged cross-crate info ---"
rustdoc -Z unstable-options --out-dir="$FINAL_DIR" "${read_flags[@]}"

# help.html/settings.html are static UI chrome, not derived from any
# crate's content -- generate them with an ordinary, non-merge-mode
# rustdoc call on a throwaway empty crate rather than trust the finalize
# step above for them. On at least one nightly seen in the wild, finalize
# mode silently didn't write these two files at all (rust-lang/rust#159473
# fixes this upstream, but not every nightly has that fix yet); a plain
# rustdoc invocation always writes them correctly and isn't exposed to
# whatever else changes about finalize mode's behavior in the future.
echo '//! empty' > "$WORK/empty.rs"
rustdoc --out-dir="$WORK/chrome" "$WORK/empty.rs"

# 4. Final assembly, three overlaid layers:
#   a) Host pass output as the base: real per-crate page content for
#      every host-buildable crate (the finalize step only carries
#      cross-crate glue, not a crate's own rendered pages).
#   b) The finalize step's merged glue files, over the host pass's own
#      (unmerged) versions of the same files.
#   c) Each CRATE_TARGETS crate's own real-target content, over
#      whatever the host pass produced for it as a dependency.
OUT="$WORK/merged"
cp -r "$WORK/doc" "$OUT"
for f in crates.js src-files.js; do
    cp "$FINAL_DIR/$f" "$OUT/$f"
done
cp "$WORK/chrome/help.html" "$OUT/help.html"
cp "$WORK/chrome/settings.html" "$OUT/settings.html"
rm -rf "$OUT/trait.impl" "$OUT/search.index" "$OUT/static.files"
cp -r "$FINAL_DIR/trait.impl" "$OUT/trait.impl"
cp -r "$FINAL_DIR/search.index" "$OUT/search.index"
cp -r "$FINAL_DIR/static.files" "$OUT/static.files"
for entry in "${CRATE_TARGETS[@]}"; do
    crate="${entry%%:*}"
    target_dir=$(target_dir_name "${entry#*:}")
    # rustdoc substitutes '-' with '_' in output directory names, since
    # crate names there are Rust identifiers (e.g. package "esp32-c3" is
    # documented under a directory named "esp32_c3").
    docname="${crate//-/_}"
    rm -rf "$OUT/$docname" "$OUT/src/$docname"
    cp -r "$WORK/$target_dir/doc/$docname" "$OUT/$docname"
    if [ -d "$WORK/$target_dir/doc/src/$docname" ]; then
        cp -r "$WORK/$target_dir/doc/src/$docname" "$OUT/src/$docname"
    fi
done

# Replace the default rust logo with our own Tock logo and the favicon with
# our own favicon. Note, it is also possible to set this using a `#[doc]`
# attribute
# (https://doc.rust-lang.org/rustdoc/the-doc-attribute.html#html_logo_url) but
# doing it this way avoids having to set the attribute for every crate.
curl https://www.tockos.org/assets/img/tocklogo.png --output "$OUT/rust-logo.png"
curl https://www.tockos.org/assets/img/icons/favicon-32x32.png --output "$OUT/favicon-32x32.png"
curl https://www.tockos.org/assets/img/icons/favicon-16x16.png --output "$OUT/favicon-16x16.png"
curl https://www.tockos.org/assets/img/icons/safari-pinned-tab.svg --output "$OUT/favicon.svg"

# Temporary redirect rule
# https://www.netlify.com/docs/redirects/
cat > "$OUT/_redirects" << EOF
# While we don't have a home page :/
/            /kernel            302
EOF

# Move the docs to doc/rustdoc.
$CP_COW -r "$OUT" doc/rustdoc
