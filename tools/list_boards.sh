#!/usr/bin/env bash

boards=`ls -l boards | egrep '^d' | awk '{print $9}'`
echo Supported Tock boards: $boards
echo ""
echo To build the kernel for a particular board, run:
echo "    make TOCK_BOARD=<board name>"
