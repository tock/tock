#!/usr/bin/env bash

DESIRED_FIXPOINT_VERSION="2009-15"
DESIRED_FIXPOINT_RELEASE_TAG="nightly"
DESIRED_FLUX_COMMIT="b0cec81c42bc6e210f675b46dd5b4b16774b0d0e"

########################################################

DO_INSTALL="false"
DO_PATH_UPDATE="false"
case "$1" in
  "check")
    ;;
  "install")
    DO_INSTALL="true"
    ;;
  "source")
    DO_PATH_UPDATE="true"
    ;;
  *)
    echo "Invalid argument: $1"
    echo " (expected one of 'check' 'install' or 'source')"
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


# Verify expectations
# https://stackoverflow.com/questions/59895/how-do-i-get-the-directory-where-a-bash-script-is-located-from-within-the-script
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
if [[ $SCRIPT_DIR != $(pwd) ]]; then
  echo Error: Must cd to dir containing this script before running.
  exit 1
fi

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

if [[ $(./fixpoint --numeric-version 2>/dev/null) == "$DESIRED_FIXPOINT_VERSION" ]]; then
  if $VERBOSE; then
    echo "fixpoint version: $(./fixpoint --numeric-version)"
  fi
else
  if $DO_INSTALL; then
    # Remove any old versions
    rm -f fixpoint fixpoint*.gz
    # Install prebuilt version
    curl -sSL https://github.com/ucsd-progsys/liquid-fixpoint/releases/download/$DESIRED_FIXPOINT_RELEASE_TAG/fixpoint-$PLATFORM.tar.gz | tar -x
    [[ $(./fixpoint --numeric-version) == "$DESIRED_FIXPOINT_VERSION" ]]
  else
    echo "Missing required dependency: fixpoint"
    exit 1
  fi
fi


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

# FIXME: Actually check the version of flux (--version doesn't match hash??)

if ! command -v flux > /dev/null; then
  if $DO_INSTALL; then
    if ! [ -d flux/ ]; then
      git clone --shallow-since=2025-01-01 https://github.com/flux-rs/flux
    fi
    pushd flux
    git pull
    git checkout $DESIRED_FLUX_COMMIT
    ## For the moment, just install globally; but ideally we pull that out to path-based runs later
    # cargo build
    # pushd crates/flux-bin
    # cargo build
    cargo xtask install
    popd
  fi
fi
