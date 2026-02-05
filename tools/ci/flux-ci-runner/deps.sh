#!/usr/bin/env bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

# Sets up the environment to execute a specific version of flux. As flux
# is currently fast-moving, this avoids any system-wide install. Thus,
# this script must be `source`d in any shell that wants to use flux.
#
# Author: Pat Pannuto <ppannuto@ucsd.edu>

DESIRED_FIXPOINT_VERSION="0.9.6.3.3"
DESIRED_FIXPOINT_RELEASE_TAG="nightly"

########################################################

# Force execution in a sourced context to avoid 'return vs exit' games
if ! (return 0 2>/dev/null); then
  source "$0" "$@"
  exit $?
fi

DO_INSTALL="false"
case "$1" in
  "check")
    ;;
  "install")
    DO_INSTALL="true"
    ;;
  *)
    echo "Invalid argument: $1"
    echo " (expected one of 'check' 'install')"
    return 1
    ;;
esac

########################################################

if [[ "$(uname -s)" == "Darwin" ]]; then
  INSTALL_CMD="brew install"
  if [[ "$(uname -m)" == "x86_64" ]]; then
    PLATFORM=x86_64-apple-darwin
  else
    PLATFORM=aarch64-apple-darwin
  fi
else
  # FIXME: Check for Windows, other stuff too
  INSTALL_CMD="sudo apt install"
  PLATFORM=x86_64-linux-gnu
fi


# Force execution from the script directory for simplicity
# https://stackoverflow.com/questions/59895/how-do-i-get-the-directory-where-a-bash-script-is-located-from-within-the-script
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
pushd "$SCRIPT_DIR" > /dev/null

# Print what we're doing on CI
VERBOSE="false"
if [ -n "$CI" ]; then
  VERBOSE="true"
  set -x
fi

# Error on failures
set -u
set -o pipefail

# Gracefully handle being `source`d or run.
#
# This replaces `set -e` such that someone `source`ing this script won't have
# their shell session terminated
handle_errror() {
  trap - ERR
  kill -INT $$
  # this sleep is important, as the kill effect is async; without it, the next
  # command non-deterministically might run before the INT is handled
  sleep 1
}
trap handle_errror ERR


#################################
## fixpoint

pushd fixpoint > /dev/null

need_new_fixpoint() {
  version=$(./fixpoint --numeric-version 2>/dev/null) || return 0
  $(../../../build/semver.sh "$version" "<" "$DESIRED_FIXPOINT_VERSION") && return 0
  if $VERBOSE; then
    echo "fixpoint version: $(./fixpoint --numeric-version)"
  fi
  return 1
}

if need_new_fixpoint; then
  if $DO_INSTALL; then
    # Remove any old versions
    rm -f fixpoint fixpoint*.gz
    # Install prebuilt version
    curl -sSL https://github.com/ucsd-progsys/liquid-fixpoint/releases/download/$DESIRED_FIXPOINT_RELEASE_TAG/fixpoint-$PLATFORM.tar.gz | tar -xz
    # Verify install
    if need_new_fixpoint; then
      echo "Failed to installed requested fixpoint"
      return 1
    fi
  else
    echo "Missing required dependency: fixpoint"
    return 1
  fi
fi

PATH="${PATH}:$(pwd)"

popd > /dev/null # fixpoint/

#################################
## z3

# TODO: Check/enforce version >= 4.15

if ! command -v z3 > /dev/null; then
  if $DO_INSTALL; then
    $INSTALL_CMD z3
  else
    echo "Missing required dependency: z3"
    return 1
  fi
fi
if $VERBOSE; then
  z3 --version
fi


#################################
## flux

PATH="${PATH}:$(pwd)/flux/target/release"
export FLUX_SYSROOT="$(pwd)/flux/target/release"

# Extract this from the project configuration to have a sole source of truth
DESIRED_FLUX_COMMIT=$(grep 'flux-rs.*rev' ../../../kernel/Cargo.toml | cut -d'"' -f4)

# nominal output is "cargo-flux SHORT_HASH (DATE)"
DESIRED_FLUX_VERSION="cargo-flux $DESIRED_FLUX_COMMIT"

if [[ $(cargo flux -Vv | cut -d' ' -f1,2 2>/dev/null) == "$DESIRED_FLUX_VERSION" ]]; then
  if $VERBOSE; then
    echo "flux version: $(cargo flux -Vv)"
  fi
else
  if $DO_INSTALL; then
    if ! [ -d flux/ ]; then
      git clone --shallow-since=2025-01-01 https://github.com/flux-rs/flux
    fi
    pushd flux
    git pull
    git checkout $DESIRED_FLUX_COMMIT
    cargo build --release
    popd
    # Verify install
    [[ $(cargo flux -Vv | cut -d' ' -f1,2 2>/dev/null) == "$DESIRED_FLUX_VERSION" ]]
  else
    echo "Missing required dependency: flux"
    return 1
  fi
fi

# Undo forcing the directory (pushd $SCRIPT_DIR)
popd > /dev/null
