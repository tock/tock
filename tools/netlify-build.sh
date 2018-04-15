#!/usr/bin/env bash

set -e
set -u
set -x

curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2018-04-19

export PATH="$PATH:$HOME/.cargo/bin"

wget -nv 'https://developer.arm.com/-/media/Files/downloads/gnu-rm/7-2017q4/gcc-arm-none-eabi-7-2017-q4-major-linux.tar.bz2?revision=375265d4-e9b5-41c8-bf23-56cbe927e156?product=GNU%20Arm%20Embedded%20Toolchain,64-bit,,Linux,7-2017-q4-major' -O arm.tgz

tar -xf arm.tgz
export PATH="$PATH:$(pwd)/gcc-arm-none-eabi-7-2017-q4-major/bin"

tools/build-all-docs.sh
