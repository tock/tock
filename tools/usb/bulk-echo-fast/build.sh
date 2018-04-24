#!/bin/sh

OPTIONS=
# Uncomment the line below to enable logging
# OPTIONS="-DLOGGING"

$CC $OPTIONS -I/usr/include/libusb-1.0/ main.c -lusb-1.0
