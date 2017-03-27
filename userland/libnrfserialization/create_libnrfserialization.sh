#!/usr/bin/env bash

set -e
set -x

rm -rf build/ headers/
rm -f headers.tar.gz
mkdir -p headers

# Commit hash of the https://github.com/lab11/nrf5x-base repository
# to use to make the library.
NRF5X_BASE_SHA=6fb0730f2e976d9b85f8dddc6a873cb58d85d810

if [ ! -f $NRF5X_BASE_SHA.zip ]; then
	wget https://github.com/lab11/nrf5x-base/archive/$NRF5X_BASE_SHA.zip
fi

if [ ! -d "nrf5x-base-$NRF5X_BASE_SHA" ]; then
	unzip $NRF5X_BASE_SHA.zip
fi

make -j NRF_BASE_PATH=nrf5x-base-$NRF5X_BASE_SHA
mkdir -p cortex-m0 cortex-m4
cp build/cortex-m0/libnrfserialization.a cortex-m0/libnrfserialization.a
cp build/cortex-m4/libnrfserialization.a cortex-m4/libnrfserialization.a

make -j headers NRF_BASE_PATH=nrf5x-base-$NRF5X_BASE_SHA


cat build/headers/*.headers | awk '{$1=$1};1' | awk '{print $1}' | sort | grep '\.h' | grep -v libtock | uniq | xargs -IFOO cp FOO headers/
tar czf headers.tar.gz headers

set +x

echo ""
echo "Done."
