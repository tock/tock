# tock-test-harness
## Description

This directory stores the necessary files to enable the Raspberry Pi to act as a runner for hardware continuous integration.

This directory contains:
 - File to configure the Raspberry Pi to act as runner for a given board. `runner_init.py`
 - Scripts in which to build the board operating system for the build, install the configuration, and tests to be ran on the board. `main.py`

Check out the [Here](https://github.com/AnthonyQ619/tock/blob/aq-config-updated/doc/CI_Hardware.md#looking-in-the-workflow) for more info about the workflow of this directory. (Contains the explanation of the files `main.py` and `Runner.py`)

For developers, go to [Getting Started](https://github.com/AnthonyQ619/tock/blob/aq-config-updated/doc/CI_Hardware.md).

## File structure:

For configuring the Raspberry Pi, look into: `runner_init.py`

For better understanding of the workflow on how boards are tested, look into: `main.py` in the `lib` folder and 'Runner.py` for the functions called to build, install, and test the board.

