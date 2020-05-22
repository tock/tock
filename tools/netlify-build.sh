#!/usr/bin/env bash
#
# Script used to install additional requirements to the base Netlify image.
#
# Should not be used or relied on outside of Netlify context.
#
#  Author: Pat Pannuto <pat.pannuto@gmail.com>


set -e
set -u
set -x

# Install rust stuff that we need
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2021-01-07

# And fixup path for the newly installed rust stuff
export PATH="$PATH:$HOME/.cargo/bin"

# Do the actual work
make ci-runner-netlify
