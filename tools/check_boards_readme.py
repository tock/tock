#!/usr/bin/env python3

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2024.

"""
Check if all of the available boards are documented in the README.
"""

import os
import re
import sys

SKIP = [
    "boards/components",
    "boards/nordic/nrf52_components",
    "boards/configurations",
    "boards/tutorials",
]


documented_boards = []
implemented_boards = []

# Find all documented capsules
with open("boards/README.md") as f:
    for l in f:
        items = re.findall(r".*\((.*?/README.md)\).*", l)
        if len(items) > 0:
            for item in items:
                documented_boards.append("boards/{}".format(item))


# Find all capsule source files.
for subdir, dirs, files in os.walk(os.fsencode("boards/")):
    # Skip some directories we do not consider.
    for skip in SKIP:
        if skip in os.fsdecode(subdir):
            break
    else:
        for file in files:
            if os.fsdecode(file) == "Cargo.toml":
                # Create the filepath to the board readme.
                filepath = os.fsdecode(
                    os.path.join(subdir, "README.md".encode("utf-8"))
                )
                implemented_boards.append(filepath)


# Calculate what doesn't seem to be documented.
missing = list(set(implemented_boards) - set(documented_boards))

# Calculate what has been removed
removed = list(set(documented_boards) - set(implemented_boards))


if len(missing) > 0:
    print("The following boards do not seem to be documented:")
    for m in sorted(missing):
        print(" - {}".format(m))

if len(removed) > 0:
    print("The following boards seem to have been removed:")
    for m in sorted(removed):
        print(" - {}".format(m))


if len(missing) > 0:
    print("ERROR: Boards missing documentation in the main boards/README.md")
    sys.exit(-1)

if len(removed) > 0:
    print("ERROR: Boards that do not exist are documented in boards/README.md ")
    sys.exit(-1)

print("Board documentation up to date.")
