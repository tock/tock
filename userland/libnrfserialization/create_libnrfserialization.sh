#!/usr/bin/env bash

# Commit hash of the https://github.com/lab11/nrf5x-base repository
# to use to make the library.
NRF5X_BASE_SHA=6fb0730f2e976d9b85f8dddc6a873cb58d85d810

if [ ! -f $NRF5X_BASE_SHA.zip ]; then
	wget https://github.com/lab11/nrf5x-base/archive/$NRF5X_BASE_SHA.zip
fi

if [ ! -d "nrf5x-base-$NRF5X_BASE_SHA" ]; then
	unzip $NRF5X_BASE_SHA.zip
fi

make NRF_BASE_PATH=nrf5x-base-$NRF5X_BASE_SHA
cp build/cortex-m4/libnrfserialization.a libnrfserialization.a

make headers NRF_BASE_PATH=nrf5x-base-$NRF5X_BASE_SHA
rm -rf headers
rm -f headers.tar.gz
mkdir -p headers
cat build/cortex-m4/*.headers | awk '{$1=$1};1' | awk '{print $1}' | sort | grep '\.h' | grep -v libtock | uniq | xargs -i cp {} headers/
tar czf headers.tar.gz headers
