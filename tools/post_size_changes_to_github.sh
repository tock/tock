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
set -x

# Only run for PR builds, not push builds
if [ -n "$TRAVIS_PULL_REQUEST_BRANCH" ]; then
    REMOTE_URL="$(git config --get remote.origin.url)"

    # Bench the current commit that was pushed. Requires navigating back to build directory
    cd ${TRAVIS_BUILD_DIR}
    make allboards
    for elf in $(find . -maxdepth 8 | grep 'release' | egrep '\.elf$' | grep -v 'riscv'); do
        tmp=${elf#*release/}
        b=${tmp%.elf}
        ${TRAVIS_BUILD_DIR}/tools/print_tock_memory_usage.py -s ${elf} | tee ${TRAVIS_BUILD_DIR}/current-benchmark-${b}
    done

    # The Travis environment variables behave like so:
    # TRAVIS_BRANCH
    #   - if PR build, this is the pr base branch
    #   - if push build, this is the branch that was pushed
    # TRAVIS_PULL_REQUEST_BRANCH
    #   - if PR build, this is the "target" of the pr, i.e. not the base branch
    #   - if push build, this is blank
    #
    # Example:
    # You open a PR with target `master`, and PR branch `foo`
    # During a PR build:
    #     TRAVIS_BRANCH=master
    #     TRAVIS_PULL_REQUEST_BRANCH=foo

    # Travis-ci uses a shallow clone, so to checkout target branch you must fetch it
    git remote set-branches origin "${TRAVIS_BRANCH}"
    git fetch --depth 1 origin "${TRAVIS_BRANCH}"
    git checkout -f "${TRAVIS_BRANCH}"
    make allboards

    # Find elfs compiled for release (for use in analyzing binaries in CI),
    # ignore riscv binaries for now because Phil's tool does not support RISC-V
    for elf in $(find . -maxdepth 8 | grep 'release' | egrep '\.elf$' | grep -v 'riscv'); do
        tmp=${elf#*release/}
        b=${tmp%.elf}
        ${TRAVIS_BUILD_DIR}/tools/print_tock_memory_usage.py -s ${elf} | tee ${TRAVIS_BUILD_DIR}/previous-benchmark-${b}
    done

    # now calculate diff for each board, and post status to github for each non-0 diff
    cd ${TRAVIS_BUILD_DIR}
    for elf in $(find . -maxdepth 8 | grep 'release' | egrep '\.elf$' | grep -v 'riscv'); do
        tmp=${elf#*release/}
        b=${tmp%.elf}
        ${TRAVIS_BUILD_DIR}/tools/diff_memory_usage.py previous-benchmark-${b} current-benchmark-${b} size-diffs-${b}.txt ${b}
        if [ -s "size-diffs-${b}.txt" ]; then
            RES="$( grep -hs ^ size-diffs-${b}.txt )" #grep instead of cat to prevent errors on no match
            if [ -s "${TRAVIS_GITHUB_TOKEN}"]; then
                # Only attempt to post statuses if the token is available (will not post for PRs from forks)
                curl -X POST -H "Content-Type: application/json" --header "Authorization: token ${TRAVIS_GITHUB_TOKEN}" --data '{"state": "success", "context": "'"${b}"'", "description": "'"${RES}"'"}' https://api.github.com/repos/tock/tock/statuses/${TRAVIS_PULL_REQUEST_SHA}
            fi
            echo "SIZE CHANGE DETECTED: ${b}: ${RES}"
        fi
    done
fi
