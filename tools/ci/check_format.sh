#!/usr/bin/env bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.
#
# Runs `rustfmt --check` on the workspace and check for tabs in Rust files.
# Must be run from root Tock directory.
#
# Author: Leon Schuermann <leon@is.currently.online>
# Author: Pat Pannuto <pat.pannuto@gmail.com>
# Author: Brad Campbell <bradjc5@gmail.com>
#
set -e

# Verify that we're running in the base directory
if [ ! -x tools/ci/check_format.sh ]; then
	echo ERROR: $0 must be run from the tock repository root.
	echo ""
	exit 1
fi

set +e
let FAIL=0
set -e

# Run `cargo fmt --check` in the workspace root, which will check all
# crates in the workspace.
echo "Running \`cargo fmt --check\`..."
CARGO_FMT_CHECK_EXIT_CODE=0
cargo fmt --check || CARGO_FMT_CHECK_EXIT_CODE=$?
if [[ $CARGO_FMT_CHECK_EXIT_CODE -ne 0 ]]; then
	let FAIL=FAIL+1
else
	echo "\`cargo fmt --check\` suceeded."
fi
printf "\n"

# Check for tab characters in Rust source files that haven't been
# removed by rustfmt
echo "Checking for Rust source files with tab characters..."
RUST_FILES_WITH_TABS="$(git grep --files-with-matches $'\t' -- '*.rs' || grep -lr --include '*.rs' $'\t' . || true)"
if [ "$RUST_FILES_WITH_TABS" != "" ]; then
	echo "ERROR: The following files contain tab characters, please use spaces instead:"
	echo "$RUST_FILES_WITH_TABS" | sed 's/^/    -> /'
	let FAIL=FAIL+1
else
	echo "No Rust source file containing tab characters found."
fi

if [[ $FAIL -ne 0 ]]; then
	printf "\n"
	echo "$(tput bold)Formatting errors.$(tput sgr0)"
	echo "See above for details. You can try running \`cargo fmt\` to correct them."
fi
exit $FAIL
