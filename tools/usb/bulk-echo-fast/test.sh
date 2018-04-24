#!/bin/sh

dd if=/dev/urandom of=input.dat bs=1 count=99999

time ./a.out <input.dat >output.dat

diff -q input.dat output.dat && echo 'Success!'
