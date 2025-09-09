#!/usr/bin/env bash

DESIRED_FIXPOINT_VERSION="0.9.6.3.3"
DESIRED_FIXPOINT_RELEASE_TAG="nightly"
DESIRED_FLUX_COMMIT="b0cec81c42bc6e210f675b46dd5b4b16774b0d0e"
DESIRED_FLUX_VERSION="FIXME"

########################################################

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
    exit 1
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
pushd $SCRIPT_DIR > /dev/null

# Print what we're doing on CI
VERBOSE="false"
if [[ -v CI ]]; then
  VERBOSE="true"
  set -x
fi

# Error on failures
set -e
set -u


#################################
## fixpoint

pushd fixpoint > /dev/null

if [[ $(./fixpoint --numeric-version 2>/dev/null) == "$DESIRED_FIXPOINT_VERSION" ]]; then
  if $VERBOSE; then
    echo "fixpoint version: $(./fixpoint --numeric-version)"
  fi
else
  if $DO_INSTALL; then
    # Remove any old versions
    rm -f fixpoint fixpoint*.gz
    # Install prebuilt version
    curl -sSL https://github.com/ucsd-progsys/liquid-fixpoint/releases/download/$DESIRED_FIXPOINT_RELEASE_TAG/fixpoint-$PLATFORM.tar.gz | tar -xz
    [[ $(./fixpoint --numeric-version) == "$DESIRED_FIXPOINT_VERSION" ]]
  else
    echo "Missing required dependency: fixpoint"
    exit 1
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
    exit 1
  fi
fi
if $VERBOSE; then
  z3 --version
fi


#################################
## flux

PATH="${PATH}:$(pwd)/flux/target/release"
export FLUX_SYSROOT="$(pwd)/flux/target/release"

# FIXME: Equality is inverted to skip actually checking the version until
# upstream flux prints out the version honestly; this effectively reduces
# to just checking whether a runnable cargo-flux exists currently
#                                         ||
if [[ $(cargo flux --version 2>/dev/null) != "$DESIRED_FLUX_VERSION" ]]; then
  if $VERBOSE; then
    echo "flux version: $(cargo flux --version)"
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
  else 
    echo "Missing required dependency: flux"
    exit 1
  fi
fi

# Undo forcing the directory (pushd $SCRIPT_DIR)
popd > /dev/null
