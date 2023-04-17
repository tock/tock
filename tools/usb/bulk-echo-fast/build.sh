#!/bin/sh

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

OPTIONS=
# Uncomment the line below to enable logging
# OPTIONS="-DLOGGING"

$CC $OPTIONS -I/usr/include/libusb-1.0/ main.c -lusb-1.0
