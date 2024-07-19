#!/usr/bin/env python3

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2024.

"""
git clone git@github.com:taiki-e/syn-serde.git
cd syn-serde/examples/rust2json
cargo run -- ../../tock/capsules/core/src/button.rs > ../../tock/button.json

then run this

./generate_syscall_library.py ../button.json
"""


import json
import sys
import pathlib
from pathlib import Path

sys.setrecursionlimit(10**6)

syntax_filename = sys.argv[1]
capsule = pathlib.Path(syntax_filename).stem


def search(tree, goal):
    if isinstance(tree, list):
        found = []
        for i in tree:
            r = search(i, goal)
            if r != None:
                found.extend(r)
        return found

    elif isinstance(tree, dict):
        found = []
        for k, v in tree.items():
            if k == goal:
                found.append(v)
                continue
            r = search(v, goal)
            if r != None:
                found.extend(r)
        return found
    else:
        if goal == tree:
            return [tree]
        return None

    return None


def find_matching_fn(fns, search_fn):
    for fn in fns:
        function_name = fn["ident"]
        if function_name == search_fn:
            return fn


commands = []
fns = None


with open(syntax_filename) as f:
    s = json.load(f)

    fns = search(s, "fn")

    fn = find_matching_fn(fns, "command")

    function_name = fn["ident"]

    if function_name == "command":
        fn_body = fn["stmts"]

        match_stmt = search(fn_body, "match")[0]

        match_arms = match_stmt["arms"]

        for arm in match_arms:
            try:
                command_index = arm["pat"]["lit"]["int"]

                if command_index == 0:
                    continue

                op = arm["body"]["method_call"]["method"]

                commands.append((command_index, op))
            except:
                pass

c_header = """
#pragma once

#include "../../tock.h"

#ifdef __cplusplus
extern "C" {
#endif

"""
print(c_header)

c_fn_format = """
{comment}
returncode_t libtock_{capsule}_{fn_name}();
"""

for commandnum, fn_name in commands:
    fn = find_matching_fn(fns, fn_name)

    c_fn = c_fn_format.format(comment="", capsule=capsule, fn_name=fn_name)

    print(c_fn)

c_footer = """

#ifdef __cplusplus
}
#endif
"""

print(c_footer)
