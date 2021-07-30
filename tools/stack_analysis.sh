#!/usr/bin/env bash

# This script requires that the .elf under analysis includes stack
# size information, and is thus most easily called using the `make stack-analysis`
# rule.

bold=$(tput bold)
normal=$(tput sgr0)

# Get a list of all stack frames and their sizes.
frames=`$(find $(rustc --print sysroot) -name llvm-readobj) --elf-output-style GNU --stack-sizes $1`

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
    printf "\n"
fi
