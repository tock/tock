#!/usr/bin/env bash

# This script requires that the .elf under analysis includes stack
# size information, and is thus most easily called using the `make stack-analysis`
# rule.

# Print the stack frame size of `main`
printf "main stack frame: \n"
$(find $(rustc --print sysroot) -name llvm-readobj) --elf-output-style GNU --stack-sizes $1 | grep 'main'

printf "\n"
printf "5 largest stack frames: \n"

# Print the 5 largest stack frames
$(find $(rustc --print sysroot) -name llvm-readobj) --elf-output-style GNU --stack-sizes $1 | sort -n -r | head -5
printf "\n"
