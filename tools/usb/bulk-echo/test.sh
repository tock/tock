#!/bin/sh

bin=target/debug

dd if=/dev/urandom of=input.dat bs=1 count=99999

time $bin/bulk-echo <input.dat >output.dat

diff -q input.dat output.dat && echo 'Success!'
