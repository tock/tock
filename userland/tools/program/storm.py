#!/usr/bin/env python2

import time
import stormloader
import sys
from stormloader import sl_api
from sys import argv

APP_BASE_ADDR = 0x30000

argv.pop(0)

img = ""

for filename in argv:
    f = open(filename)
    img = img + f.read()
    f.close()

print("Writing %d files totaling %d bytes" % (len(argv), len(img)))

# Padding zeroes at the end
img = img + '\x00\x00\x00\x00\x00\x00\x00\x00'


try:
    sl = sl_api.StormLoader(None)
    sl.enter_bootload_mode()
    then = time.time()
    sl.write_extended_irange(APP_BASE_ADDR, img)
    now = time.time()
    print("Wrote %d bytes in %.3f seconds" %(len(img), now-then))
    expected_crc = sl.crc32(img)
    written_crc = sl.c_crcif(APP_BASE_ADDR, len(img))
    if expected_crc != written_crc:
        print("CRC failure: expected 0x%04x, got 0x%04x" % (expected_crc, written_crc))
        sys.exit(1)
    else:
        print("CRC pass")
    sl.enter_payload_mode()
except sl_api.StormloaderException as e:
    print("Fatal error:", e)
    sys.exit(1)

