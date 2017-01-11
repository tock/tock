#!/usr/bin/env sh

set -e

mkdir ar_hack
cp libc.a libm.a ar_hack/

pushd ar_hack
arm-none-eabi-ar -xv libc.a --plugin=$(arm-none-eabi-gcc --print-file-name=liblto_plugin.so)
arm-none-eabi-ar -xv libm.a --plugin=$(arm-none-eabi-gcc --print-file-name=liblto_plugin.so)
rm libc.a libm.a
