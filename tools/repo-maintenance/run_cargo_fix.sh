#!/usr/bin/env bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

# Runs cargo fix (which removes lint warnings) on every subdirectory from .
# that has a Cargo.toml file.
#
# Author: Brad Campbell <bradjc5@gmail.com>

FAIL=0

set -e

# Verify that we're running in the base directory
if [ ! -x tools/repo-maintenance/run_cargo_fix.sh ]; then
	echo ERROR: $0 must be run from the tock repository root.
	echo ""
	exit 1
fi

for f in $(find . | grep Cargo.toml); do
	pushd $(dirname $f) > /dev/null
	cargo fix --allow-dirty || let FAIL=FAIL+1
	popd > /dev/null
done

