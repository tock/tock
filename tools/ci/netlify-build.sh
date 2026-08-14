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

# Netlify automatically restores ~/.rustup, ~/.cargo/{registry,bin}, and
# ./target from the previous build's cache when Cargo.toml/Cargo.lock are
# present at the repo root (see run-build-functions.sh in
# netlify/build-image), so put the cache's bin dir on PATH first and only
# run the installer if that didn't already give us a cargo, instead of
# unconditionally reinstalling over a cache hit every time.
export PATH="$PATH:$HOME/.cargo/bin"

if ! command -v cargo > /dev/null 2>&1; then
    curl https://sh.rustup.rs -sSf | sh -s -- -y
fi

# Do the actual work
make ci-runner-netlify
