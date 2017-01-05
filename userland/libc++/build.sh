#!/bin/bash

GCC_SRC_DIR=$1

NEWLIB_VERSION=2.2.0.20150423

SCRIPTPATH=$( cd $(dirname $0) ; pwd -P )
NEWLIB_INCLUDE_PATH=$SCRIPTPATH/../newlib/newlib-$NEWLIB_VERSION/newlib/libc/include

export CFLAGS_FOR_TARGET="-g -Os -ffunction-sections -fdata-sections -fPIC -msingle-pic-base -mno-pic-data-is-text-relative -mthumb -mcpu=cortex-m0 -isystem $NEWLIB_INCLUDE_PATH"
export CXXFLAGS_FOR_TARGET="-g -Os -ffunction-sections -fdata-sections -fPIC -msingle-pic-base -mno-pic-data-is-text-relative -mthumb -mcpu=cortex-m0 -isystem $NEWLIB_INCLUDE_PATH"

# 6.2.0:
$GCC_SRC_DIR/configure \
  --build=x86_64-linux-gnu \
  --host=x86_64-linux-gnu \
  --target=arm-none-eabi \
  --with-cpu=cortex-m0 \
  --disable-fpu \
  --with-newlib \
  --with-headers=$NEWLIB_INCLUDE_PATH \
  --enable-languages="c c++" \

make -j
