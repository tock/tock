#!/usr/bin/env bash

# Post commit statuses to github indicating how a PR affects flash and RAM use for different boards.
# This script is run by Travis after successful PR builds. It reports resource differences between
# the target branch before and after merging in the PR.
# This script also prints more detailed size analysis to the Travis build log.
# This script only reports updates for boards whose size have changed as a result of the PR being
# tested, and does not currently support analyzing size differences in RISC-V boards.
# This file relies on a travis enviroment variable to post to github, which is the value of a
# Github OAuth personal token associated with @hudson-ayers Github identity.

set -e

# Bench the current commit that was pushed. Requires navigating back to build directory
make allboards > /dev/null 2>&1
for elf in $(find . -maxdepth 8 | grep 'release' | egrep '\.elf$' | grep -v 'riscv'); do
    tmp=${elf#*release/}
    b=${tmp%.elf}
    ./tools/print_tock_memory_usage.py -s ${elf} > current-benchmark-${b}
done

git remote set-branches origin master  > /dev/null 2>&1
git fetch --depth 1 origin master > /dev/null 2>&1
git checkout master > /dev/null 2>&1
make allboards > /dev/null 2>&1

# Find elfs compiled for release (for use in analyzing binaries in CI),
# ignore riscv binaries for now because size tool does not support RISC-V
for elf in $(find . -maxdepth 8 | grep 'release' | egrep '\.elf$' | grep -v 'riscv'); do
    tmp=${elf#*release/}
    b=${tmp%.elf}
    ./tools/print_tock_memory_usage.py -s ${elf} > previous-benchmark-${b}
done

# now calculate diff for each board, and post status to github for each non-0 diff
for elf in $(find . -maxdepth 8 | grep 'release' | egrep '\.elf$' | grep -v 'riscv'); do
    tmp=${elf#*release/}
    b=${tmp%.elf}
    ./tools/diff_memory_usage.py previous-benchmark-${b} current-benchmark-${b} size-diffs-${b}.txt ${b}
    if [ -s "size-diffs-${b}.txt" ]; then
        RES="$( grep -hs ^ size-diffs-${b}.txt )" #grep instead of cat to prevent errors on no match
        #if [ -n "${TRAVIS_GITHUB_TOKEN}" ]; then
            # Only attempt to post statuses if the token is available (will not post for PRs from forks)
        #    curl -X POST -H "Content-Type: application/json" --header "Authorization: token ${TRAVIS_GITHUB_TOKEN}" --data '{"state": "success", "context": "'"${b}"'", "description": "'"${RES}"'"}' https://api.github.com/repos/tock/tock/statuses/${TRAVIS_PULL_REQUEST_SHA}
        #fi
        echo "SIZE CHANGE DETECTED: ${b}: ${RES}"
    fi
done
