#!/usr/bin/env python3

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

"""
Check if all of the available capsules are documented in the README.
"""

import os
import re
import sys

SKIP = [
    "README.md",
    "src/lib.rs",
    "/test",
    "src/rf233_const.rs",
    "extra/src/tutorials",
]


documented_capsules = []
implemented_capsules = []


def parse_capsule_readme(readme_filename, documented_list):
    root = "/".join(readme_filename.split("/")[:-1])

    # Find all documented capsules
    with open(readme_filename) as f:
        for l in f:
            items = re.findall(r".*\((src/.*?)\).*", l)
            if len(items) > 0:
                for item in items:
                    documented_capsules.append("{}/{}".format(root, item))


def find_implemented_capsules(root_path, implemented_list):
    # Find all capsule source files.
    for subdir, dirs, files in os.walk(os.fsencode(root_path)):
        for file in files:
            filepath = os.fsdecode(os.path.join(subdir, file))

            # Include the directory on behalf of `mod.rs` files.
            if os.fsdecode(file) == "mod.rs":
                # If we document any file in this folder, we must document them
                # all.
                document_within_this_folder = False
                for doc_capsule in documented_capsules:
                    fp = os.fsdecode(subdir)
                    if doc_capsule.startswith(fp) and doc_capsule != fp:
                        document_within_this_folder = True
                        break
                if document_within_this_folder:
                    # Skip the mod.rs
                    continue
                else:
                    # Use the mod.rs as the entire folder
                    filepath = os.fsdecode(subdir)

            # Skip some noise.
            for skip in SKIP:
                if skip in filepath:
                    break
            else:
                # Skip files where the directory (e.g. extra/src/net) is
                # documented.
                for doc_capsule in documented_capsules:
                    if filepath.startswith(doc_capsule) and filepath != doc_capsule:
                        break
                else:
                    implemented_list.append(filepath)


# Find links in readmes.
parse_capsule_readme("capsules/core/README.md", documented_capsules)
parse_capsule_readme("capsules/extra/README.md", documented_capsules)

# Find actual files of capsules.
find_implemented_capsules("capsules/core/src", implemented_capsules)
find_implemented_capsules("capsules/extra/src", implemented_capsules)

# Calculate what doesn't seem to be documented.
missing = list(set(implemented_capsules) - set(documented_capsules))

# Calculate what has been removed
removed = list(set(documented_capsules) - set(implemented_capsules))


if len(missing) > 0:
    print("The following capsules do not seem to be documented:")
    for m in sorted(missing):
        print(" - {}".format(m))

if len(removed) > 0:
    print("The following capsules seem to have been removed:")
    for m in sorted(removed):
        print(" - {}".format(m))


if len(missing) > 0:
    print("ERROR: Capsules missing documentation")
    sys.exit(-1)

if len(removed) > 0:
    print("ERROR: Capsules documented that are missing")
    sys.exit(-1)

print("Capsule documentation up to date.")
