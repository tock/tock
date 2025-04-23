#!/usr/bin/env bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.
#
# Script used to install additional requirements to the base Netlify image.
#
# Should not be used or relied on outside of Netlify context
# (exception: the docs-ci GitHub actions workflow, see issue #3428).
#
#  Author: Pat Pannuto <pat.pannuto@gmail.com>


set -e
set -u
set -x

# Install rust stuff that we need
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2025-05-19

# And fixup path for the newly installed rust stuff
export PATH="$PATH:$HOME/.cargo/bin"

# Do the actual work
make ci-runner-netlify
