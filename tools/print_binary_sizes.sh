#!/usr/bin/env bash

set -e
set -x

#TODO: Test below line only runs for Pull request builds
if [ -n "$TRAVIS_PULL_REQUEST_BRANCH" ]; then
#if [ "${TRAVIS_PULL_REQUEST_BRANCH:-$TRAVIS_BRANCH}" != "master" ]; then
    REMOTE_URL="$(git config --get remote.origin.url)"

    # Clone the repository fresh..for some reason checking out master fails
    # from a normal PR build's provided directory
    cd ${TRAVIS_BUILD_DIR}/..
    #cd ~/ #TODO: Bring me back above
    git clone ${REMOTE_URL} "${TRAVIS_REPO_SLUG}_tock_bench" #TODO: Bring me back
    cd  "${TRAVIS_REPO_SLUG}_tock_bench"
    #git checkout master

    # The Travis environment variables behave like so:
    # TRAVIS_BRANCH
    #   - if PR build, this is the pr base branch
    #   - if push build, this is the branch that was pushed
    # TRAVIS_PULL_REQUEST_BRANCH
    #   - if PR build, this is the "target" of the pr, i.e. not the base branch
    #   - if push build, this is blank
    #
    # Example:
    # You open a PR with base `master`, and PR branch `foo`
    # During a PR build:
    #     TRAVIS_BRANCH=master
    #     TRAVIS_PULL_REQUEST_BRANCH=foo
    # During a push build:
    #     TRAVIS_BRANCH=foo
    #     TRAVIS_PULL_REQUEST_BRANCH=

    #TODO: Restore below lines for when running on actual Travis
    # Bench the pull request base or master
    #if [ -n "$TRAVIS_PULL_REQUEST_BRANCH" ]; then
    git checkout -f "$TRAVIS_BRANCH"
    #else # this is a push build
    #  # This could be replaced with something better like asking git which
    #  # branch is the base of $TRAVIS_BRANCH
    #  git checkout -f master
    #fi
    make allboards > /dev/null
    #cp ~/tock/tools/print_tock_memory_usage.py tools/ #TODO: REMOVE ME
    #cp ~/tock/tools/diff_memory_usage.py tools/ #TODO: REMOVE ME
    # Find elfs compiled for release (for use in analyzing binaries in CI),
    # ignore riscv binaries for now because Phil's tool does not support RISC-V
    for elf in $(find boards -maxdepth 8 | grep 'release' | egrep '\.elf$' | grep -v 'riscv'); do
        tmp=${elf#*release/}
        b=${tmp%.elf}
        ./tools/print_tock_memory_usage.py ${elf} > previous-benchmark-${b}
    done
    # Bench the current commit that was pushed
    git checkout -f "${TRAVIS_PULL_REQUEST_BRANCH}"
    #git checkout layered_net_caps
    make allboards > /dev/null
    for elf in $(find boards -maxdepth 8 | grep 'release' | egrep '\.elf$' | grep -v 'riscv'); do
        tmp=${elf#*release/}
        b=${tmp%.elf}
        ./tools/print_tock_memory_usage.py ${elf} > current-benchmark-${b}
    done
    #cargo benchcmp previous-benchmark current-benchmark
    for elf in $(find boards -maxdepth 8 | grep 'release' | egrep '\.elf$' | grep -v 'riscv'); do
        tmp=${elf#*release/}
        b=${tmp%.elf}
        ./tools/diff_memory_usage.py previous-benchmark-${b} current-benchmark-${b} size-diffs.txt ${b}
    done
    echo SIZE CHANGES \(if any\):
    grep -hs ^ size-diffs.txt # Used instead of cat to prevent errors on no match
fi
