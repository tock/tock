#!/bin/bash

set -ex

${OBJCOPY} --output-target=binary ${OBJCOPY_FLAGS} ${1} redboard-artemis-nano-tests.bin
python ambiq/ambiq_bin2board.py --bin redboard-artemis-nano-tests.bin --load-address-blob 0x20000 -b 115200 -port ${PORT} -r 2 -v --magic-num 0xCB --version 0x0 --load-address-wired 0xc000 -i 6 --options 0x1

# If we connect too quickly the UART doesn't work, so add a small delay
sleep 1
screen ${PORT} 115200
