#!/usr/bin/env bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

FAIL=0

# Find folders with Cargo.toml files but no README.md file.
for f in $(find . | grep Cargo.toml); do
	dir=$(dirname $f)
	readme=${dir}/README.md

	if [ ! -f "$readme" ]; then
	    echo "$readme does not exist!"
	    let FAIL=FAIL+1
	fi
done

exit $FAIL
