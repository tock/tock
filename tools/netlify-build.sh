#!/usr/bin/env bash
#
#  Script used to build docs on netlify rather than travis.
#
#  Author: Pat Pannuto <pat.pannuto@gmail.com>


set -e
set -u
set -x

curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2019-09-19

export PATH="$PATH:$HOME/.cargo/bin"

tools/build-all-docs.sh
