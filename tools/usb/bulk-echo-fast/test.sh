#!/bin/sh

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

dd if=/dev/urandom of=input.dat bs=1 count=99999

time ./a.out <input.dat >output.dat

diff -q input.dat output.dat && echo 'Success!'
