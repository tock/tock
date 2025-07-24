#!/usr/bin/env bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

# Post github check indicating how a PR affects flash and RAM use for different boards.
# This script is run by Github actions after successful PR builds. It reports resource differences between
# the target branch before and after merging in the PR.
# This script only reports updates for boards whose size have changed as a result of the PR being
# tested, and does not currently support analyzing size differences in RISC-V boards.

UPSTREAM_REMOTE_NAME="${UPSTREAM_REMOTE_NAME:-origin}"
GITHUB_BASE_REF="${GITHUB_BASE_REF:-master}"

set -e

# Bench the current commit that was pushed. Requires navigating back to build directory
make allboards > /dev/null 2>&1
for elf in $(find . -maxdepth 8 | grep 'release' | grep -E '\.elf$'); do
    tmp=${elf#*release/}
    b=${tmp%.elf}
    ./tools/print_tock_memory_usage.py -w ${elf} > current-benchmark-${b}
done

git remote set-branches "$UPSTREAM_REMOTE_NAME" "$GITHUB_BASE_REF"  > /dev/null 2>&1
git fetch --depth 1 "$UPSTREAM_REMOTE_NAME" "$GITHUB_BASE_REF" > /dev/null 2>&1
git checkout "$UPSTREAM_REMOTE_NAME"/"$GITHUB_BASE_REF" > /dev/null 2>&1
make allboards > /dev/null 2>&1

# Find elfs compiled for release (for use in analyzing binaries in CI),
for elf in $(find . -maxdepth 8 | grep 'release' | grep -E '\.elf$'); do
    tmp=${elf#*release/}
    b=${tmp%.elf}
    ./tools/print_tock_memory_usage.py -w ${elf} > previous-benchmark-${b}
done

DIFF_DETECTED=0

# now calculate diff for each board, and post status to github for each non-0 diff
for elf in $(find . -maxdepth 8 | grep 'release' | grep -E '\.elf$'); do
    tmp=${elf#*release/}
    b=${tmp%.elf}
    # Compute a summary suitable for GitHub.
    ./tools/diff_memory_usage.py previous-benchmark-${b} current-benchmark-${b} size-diffs-${b}.txt ${b}
    if [ -s "size-diffs-${b}.txt" ]; then
	DIFF_DETECTED=1
        RES="$( grep -hs ^ size-diffs-${b}.txt )" #grep instead of cat to prevent errors on no match
        echo "${b}: ${RES}"
    fi
    # Print a detailed by raw line-by-line diff. Can be useful to
    # understand where the size differences come from.
    git diff --no-index previous-benchmark-${b} current-benchmark-${b} || true #Supress exit code
done

if [ $DIFF_DETECTED -eq 0 ]; then
    echo "-> No size difference on any board detected"
fi
