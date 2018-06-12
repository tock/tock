#!/usr/bin/env bash

set -e
set -x

pushd $HOME

if [ ! -x gcc-arm-none-eabi-6_2-2016q4/bin/arm-none-eabi-gcc ]; then
  wget https://developer.arm.com/-/media/Files/downloads/gnu-rm/6-2016q4/gcc-arm-none-eabi-6_2-2016q4-20161216-linux.tar.bz2?product=GNU%20ARM%20Embedded%20Toolchain,64-bit,,Linux,6-2016-q4-major -O gcc.tar.bz2
  tar -xjf gcc.tar.bz2
fi

