#!/usr/bin/env bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

# Fast rustdoc warning/error check across every crate, including arch/chip
# crates that only compile for a real embedded target (no host-buildable
# `doc`-mock cfg branch). Same crate/target coverage as build_all_docs.sh,
# but skips HTML rendering and cross-crate merging -- this only cares
# about rustdoc's own diagnostics (-D warnings via deny_warnings.toml),
# not a browsable doc/rustdoc tree. Much faster for local iteration; CI's
# published docs still come from build_all_docs.sh.
#
# Usage: tools/build/check_docs.sh

set -e

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
source "$ROOT/tools/build/doc_crate_targets.sh"

cd "$ROOT"
DENY_WARNINGS_CONFIG="$ROOT/boards/cargo/deny_warnings.toml"

# 1. Host pass: every crate not in CRATE_TARGETS. --no-deps skips
# generating docs for third-party dependencies. --output-format=json
# skips HTML rendering (rustdoc's diagnostics run regardless of output
# format, so this doesn't lose any warnings/errors).
echo "--- checking host-buildable crates ---"
cargo doc --no-deps -Z unstable-options --output-format=json --config "$DENY_WARNINGS_CONFIG"

# 2. One independent pass per CRATE_TARGETS crate, against its real
# target -- these have no host-buildable fallback, so the host pass above
# never actually type-checks their embedded-only code.
echo "--- checking arch-specific crates ---"
for entry in "${CRATE_TARGETS[@]}"; do
    crate="${entry%%:*}"
    target="${entry#*:}"
    echo "--- checking $crate for $target ---"
    if [ "${target%.json}" != "$target" ]; then
        # A custom JSON target spec (no builtin triple exists) needs
        # cargo's own -Z json-target-spec just to accept a `--target
        # *.json` path, and -Z build-std since custom targets have no
        # prebuilt std/core component to fall back on.
        cargo -Z json-target-spec -Z build-std=core,compiler_builtins -Z unstable-options \
            doc -p "$crate" --no-deps --target "$target" --output-format=json --config "$DENY_WARNINGS_CONFIG"
    else
        cargo doc -p "$crate" --no-deps --target "$target" -Z unstable-options --output-format=json --config "$DENY_WARNINGS_CONFIG"
    fi
done

echo "--- all doc checks passed ---"
