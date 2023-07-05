#!/usr/bin/env bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

# This script requires that the .elf under analysis includes stack
# size information, and is thus most easily called using the `make stack-analysis`
# rule.

bold=$(tput bold)
normal=$(tput sgr0)

# Get a list of all stack frames and their sizes.
frames=`$(find $(rustc --print sysroot) -name llvm-readobj) --demangle --elf-output-style GNU --stack-sizes $1`

# Print the stack frame size of `main`
printf "   main stack frame: \n"
echo "$frames" | grep ' main' # Use a space before main to avoid functions with main in the name.
printf "\n"

# Print the 5 largest stack frames
printf "   5 largest stack frames: \n"
echo "$frames" | sort -n -r | head -5
printf "\n"

# Check if main is the largest stack frame
largest=`echo "$frames" | sort -n -r | head -1 | grep ' main'`
largest_ret_val=$?

# If it is, print a warning.
if [ $largest_ret_val -eq 0 ]; then
    echo "   ${bold}WARNING! main is the largest stack frame!${normal}"
    echo "   See https://github.com/tock/tock/issues/2425 for an explanation of"
    echo "   why this is an issue, and https://github.com/tock/tock/pull/2715 for"
    echo "   an example of how to fix it."
    printf "\n"
fi
