#!/bin/bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

OBJCOPY=$(find "$(rustc --print sysroot)" -name llvm-objcopy | head -1)
${OBJCOPY} --output-target=binary --strip-sections --strip-all --remove-section .apps ${1} ${1}.bin
esptool.py --port /dev/ttyUSB0 --chip esp32c3 write_flash 0x0 ${1}.bin
