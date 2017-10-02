#!/bin/bash

NEWLIB_SRC_DIR=$1

$NEWLIB_SRC_DIR/configure --target=arm-none-eabi \
  --disable-newlib-supplied-syscalls \
  --disable-nls \
  --enable-newlib-reent-small \
  --disable-newlib-fvwrite-in-streamio \
  --disable-newlib-fseek-optimization \
  --disable-newlib-wide-orient \
  --enable-newlib-nano-malloc \
  --disable-newlib-unbuf-stream-opt \
  --enable-lite-exit \
  --enable-newlib-global-atexit \
  --disable-newlib-io-float

# --enable-newlib-nano-formatted-io

# Okay.. I cannot puzzle out a good way to add a flag to newlib's `ar` or `ranlib` invocation,
# so we're going to hack around this in a somewhat creative fashion:
ar=$(which arm-none-eabi-ar)
ranlib=$(which arm-none-eabi-ranlib)

rm -rf binhack
mkdir binhack

cat > binhack/arm-none-eabi-ar << EOF
#!/usr/bin/env sh
$ar "\$@" --plugin=$(arm-none-eabi-gcc --print-file-name=liblto_plugin.so)
EOF
chmod +x binhack/arm-none-eabi-ar

cat > binhack/arm-none-eabi-ranlib << EOF
#!/usr/bin/env sh
$ranlib "\$@" --plugin=$(arm-none-eabi-gcc --print-file-name=liblto_plugin.so)
EOF
chmod +x binhack/arm-none-eabi-ranlib

PATH="$PWD/binhack:$PATH" make -j CFLAGS_FOR_TARGET='-g -Os -ffunction-sections -fdata-sections -fPIC -flto -ffat-lto-objects -msingle-pic-base -mno-pic-data-is-text-relative'
