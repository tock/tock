#!/usr/bin/env python3

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

# Prints out the source locations of panics in a Tock kernel ELF
#
# This tool attempts to trace all panic locations in a Tock kernel ELF by
# tracing calls to panic functions in the core library, using the debug information
# embedded in the ELF file. This tool requires an ELF which includes debug information.
# In its current state, cannot accurately provide the source locations
# corresponding to each panic, but tries to be honest about its confidence in
# each guess. In general, each guess is usually enough to locate the relevant panic.
# More creative analysis might be able to increase
# the accuracy with which this tool can identify source locations of panics. For now,
# this tool is useful for:
#
# - obtaining a rough count of the number of panics in a Tock kernel binary
#
# - finding and removing panics in a Tock kernel binary
#
# - roughly determining which components of a Tock kernel binary contain the most panic
# paths
#
# There are several assumptions built into this tool which may not always hold. For one,
# the list of panic_functions are assumed to not match any strings in the actual
# codebase, despite the fact they are incomplete function names and overlap is possible.
# I could solve this by using full names of these functions, but I am unsure how often
# the name mangling of these functions will change as the rust compiler changes so this
# approach felt potentially more stable.
#
# Several assumptions are made about DWARF locations that do not always hold, so source
# locations are not always accurate -- sometimes, the printed location just points to
# the function containing a panic, rather than the actual line on which the panic
# occurs.  Some assumptions about which panics are in the core library and will be
# caught by grepping for other calls may also not always hold. The best way to inspect
# these is by manually inspecting the panics in the `within_core_panic_list`.
#
# This script stores panics which it cannot trace out of the core library in the
# `no_info_panic_list`. If this list contains some panics, that is a sign that some
# panics have not been identified. You can manually look at the addresses stored in
# this list, attempt to find the core library function which leads to these instrucitons
# being called, and then add those core library functions to the list of panic functions.
#
# The output of this script is *not* stable.
#
# Usage: find_panics.py ELF [--riscv]
#
# Requires Python 3.7+
#
# Author: Hudson Ayers <hayers@.stanford.edu>

import argparse
import platform
import re
import subprocess
import sys


if platform.system() == 'Darwin':
    DWARFDUMP = "dwarfdump"
elif platform.system() == 'Linux':
    DWARFDUMP = "llvm-dwarfdump"
else:
    raise NotImplementedError("Unknown platform")
# Note: In practice, GCC objdumps are better at symbol resolution than LLVM objdump
ARM_OBJDUMP = "arm-none-eabi-objdump"
RISCV_OBJDUMP = "riscv64-unknown-elf-objdump"

# TODO: For all functions below the initial batch, it would like be preferable to
# automatically populate the list with additional functions in the core library using
# debug info. For now, however, I do this manually.
panic_functions = [
    "expect_failed",
    "unwrap_failed",
    "panic_bounds_check",
    "slice_index_order_fail",
    "slice_end_index_len_fail",
    "slice_start_index_len_fail",
    "slice17len_mismatch_fail",
    "str16slice_error_fail",
    "copy_from_slice17len_mismatch_fail",
    "copy_from_slice17",
    "panicking5panic",
    # below are functions I have manually traced up from the above, more "core" panics, on a riscv binary with a low inline threshold
    "6unwrap17",
    "6expect17",
    "11copy_within17",
    "core..fmt..builders..PadAdapter",  # calls slice_error_fail
    "11copy_within17",  # calls panicking::panic
    "write_char",  # calls PadAdapter one above
    "write_str",  # calls write_char
    "printable5check",  # calls slice_index_order_fail
    "char$u20$as$u20$core..fmt..Debug",  # calls printable5check
    "GenericRadix7fmt_int",  # calls slice_start_index_len_fail
    # below are functions I manually traced on an arm binary,
    # with a somewhat higher inline threshold.
    "10unwrap_err17h6",
    "13is_whitespace17",
    "$u20$core..slice..index..SliceIndex$LT",
    "core..iter..adapters..filter..Filter$LT$I$C$P$GT$$u20$as$u20$core..iter",
    "_ZN4core5slice5index74_$LT$impl$u20$core..ops..index..Index$LT$I$GT$$u20$for$u20$$u5b$T$u5d$$GT$5index17h4c77379bd26a525bE",
    "_ZN4core5slice5index74_$LT$impl$u20$core..ops..index..Index$LT$I$GT$$u20$for$u20$$u5b$T$u5d$$GT$5index17hfe7e43aa2388c47bE",
]

# Pre-compiled regex lookups
dw_at_file_re = re.compile(r""".*(?:DW_AT_call_file|DW_AT_decl_file).*""")
dw_at_line_re = re.compile(r""".*(?:DW_AT_call_line|DW_AT_decl_line).*""")
line_info_re = re.compile(r""".*Line info.*""")
abstract_origin_re = re.compile(r""".*DW_AT_abstract_origin.*""")
dw_at_linkage_name_re = re.compile(r""".*DW_AT_linkage_name.*""")
dw_at_name_re = re.compile(r""".*DW_AT_name.*""")


def matches_panic_funcs(name):
    """If the passed name contains one of the known panic_functions,
    return the match
    """
    for func in panic_functions:
        if func in name:
            return func
    return ""


def linkage_or_origin_all_parents(elf, addr, linkage=False):
    """Returns a list of the abstract origin or linkage of all parents of the dwarf
    location for the passed address
    """
    result = subprocess.run(
        (DWARFDUMP, "--lookup=0x" + addr, "-p", elf), capture_output=True, text=True
    )
    dwarfdump = result.stdout
    regex = abstract_origin_re
    if linkage:
        regex = dw_at_linkage_name_re
    matches = re.findall(regex, dwarfdump)

    def getFunction(line):
        return line.strip().split('"')[1]

    origins = list(map(getFunction, matches))
    return origins


def any_origin_matches_panic_func(elf, addr):
    """returns name if any origin for the passed addr matches one
    of the functions in the panic_functions array
    """
    origins = linkage_or_origin_all_parents(elf, addr)
    for origin in origins:
        name = matches_panic_funcs(origin)
        if name:
            return name
    return ""


def any_linkage_matches_panic_func(elf, addr):
    """returns True + name if any linkage for the passed addr matches one
    of the functions in the panic_functions array
    """
    linkages = linkage_or_origin_all_parents(elf, addr, True)
    for linkage in linkages:
        name = matches_panic_funcs(linkage)
        if name:
            return name
    return ""


def check_for_source_in_parent(elf, addr):
    """Takes in a dwarfdump lookup including parents of the source DWARF
    location, returns the first parent with a call file not in
    the core library. If found, this often indicates the source of the panic
    in the Tock source code.
    """
    result = subprocess.run(
        (DWARFDUMP, "--lookup=0x" + addr, "-p", elf), capture_output=True, text=True
    )
    dwarfdump = result.stdout
    matches = re.findall(dw_at_file_re, dwarfdump)

    def getFile(line):
        return line.strip().split('"')[1]

    source_files = list(map(getFile, matches))
    for (i, f) in enumerate(source_files[::-1]):
        if "/core/" not in f:
            line_matches = re.findall(dw_at_line_re, dwarfdump)

            def getLine(line):
                return line.strip().split("(")[1].split(")")[0]

            source_lines = list(map(getLine, line_matches))
            source_line = source_lines[::-1][i]
            return (f, source_line)
    return ("", "")


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("ELF", help="ELF file for analysis")
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="Output additional DWARF info for each panic location in the binary",
    )
    parser.add_argument("--riscv", action="store_true", help="Use risc-v based objdump")
    return parser.parse_args()


# Find all addresses that panic, and get basic dwarf info on those addresses
def find_all_panics(objdump, elf, is_riscv):
    panic_list = []
    within_core_panic_list = []
    no_info_panic_list = []
    result = subprocess.run((objdump, "-d", elf), capture_output=True, text=True)
    objdump_out = result.stdout
    for function in panic_functions:
        function_re = re.compile(".*:.*#.*" + function + ".*")
        if not is_riscv:
            # Arm-none-eabi-objdump uses ';' for comments instead of '#'
            function_re = re.compile(".*:.*<.*" + function + ".*")
            # TODO: arm elfs include loads of offsets from symbols in such a way that these lines
            # are matched by this regex. In general, these loads occur within the instruction stream
            # associated with the symbol at hand, and will usually be excluded by logic later in
            # this function. This leads to `within_core_panic_list` and `no_info_panic_list`
            # containing more "panics" than when analyzing a risc-v binary. We could fix this
            # by matching *only* on functions with instructions that actually jump to a new symbol,
            # but this would require a list of such instructions for each architecture. However
            # as written it actually lets us identify panics which are jumped to via addresses
            # stored in registers, which may actually catch additional valid panics.
        matches = re.findall(function_re, objdump_out)

        def getAddr(line):
            return line.strip().split(":")[0]

        addrs = list(map(getAddr, matches))
        for addr in addrs:
            result = subprocess.run(
                (DWARFDUMP, "--lookup=0x" + addr, elf), capture_output=True, text=True
            )
            dwarfdump = result.stdout
            dw_at_file = re.search(dw_at_file_re, dwarfdump)
            dw_at_line = re.search(dw_at_line_re, dwarfdump)
            line_info = re.search(line_info_re, dwarfdump)
            abstract_origin = re.search(abstract_origin_re, dwarfdump)
            linkage_name = re.search(dw_at_linkage_name_re, dwarfdump)
            file_string = ""
            line_string = ""
            line_info_string = ""
            abstract_origin_string = ""
            linkage_name_string = ""
            if dw_at_file:
                file_string = dw_at_file.group(0).strip()
                line_string = dw_at_line.group(0).strip()
            panicinfo = {}
            panicinfo["addr"] = addr
            panicinfo["function"] = function
            if line_info:
                line_info_string = line_info.group(0).strip()
                panicinfo["line_info"] = line_info_string
            if abstract_origin:
                abstract_origin_string = abstract_origin.group(0).strip()
            if linkage_name:
                linkage_name_string = linkage_name.group(0).strip()
            if "DW_AT_call_file" in file_string and "DW_AT_decl_file" in file_string:
                raise RuntimeError("I misunderstand DWARF")
            if "DW_AT_call_file" in file_string or "DW_AT_decl_file" in file_string:
                filename = file_string.split('"')[1]
                line_num = line_string.split("(")[1].split(")")[0]
                if "DW_AT_call_file" in file_string:
                    panicinfo["call_file"] = filename
                    panicinfo["call_line"] = line_num
                if "DW_AT_decl_file" in file_string:
                    panicinfo["decl_file"] = filename
                    panicinfo["decl_line"] = line_num
                if not "/core/" in filename:
                    if not "closure" in abstract_origin_string:
                        panicinfo["best_guess_source"] = "call/decl"
                    else:
                        panicinfo["best_guess_source"] = "call-closure-line-info"
                    panic_list.append(panicinfo)
                    continue
                else:  # 'core' in filename
                    (parent_file, parent_line) = check_for_source_in_parent(elf, addr)
                    if parent_file:
                        panicinfo["parent_call_file"] = parent_file
                        panicinfo["parent_call_line"] = parent_line
                        panicinfo["best_guess_source"] = "parent"
                        panic_list.append(panicinfo)
                        continue
                    elif not abstract_origin and not linkage_name:
                        no_info_panic_list.append(panicinfo)
                        continue
                    elif abstract_origin:
                        if "core" in abstract_origin_string:
                            name = matches_panic_funcs(abstract_origin_string)
                            if name:
                                within_core_panic_list.append(panicinfo)
                                continue
                            else:
                                name2 = any_origin_matches_panic_func(elf, addr)
                                name3 = any_linkage_matches_panic_func(elf, addr)
                                if name2:
                                    within_core_panic_list.append(panicinfo)
                                    continue
                                elif name3:
                                    within_core_panic_list.append(panicinfo)
                                    continue
                                else:
                                    no_info_panic_list.append(panicinfo)
                                    continue
                        elif "closure" in abstract_origin_string:
                            # not in core, in closure, line info is probably sufficient
                            panicinfo["best_guess_source"] = "lineinfo"
                            panic_list.append(panicinfo)
                            continue
                        else:
                            # i have not seen this happen -- core in file, not closure, origin not core
                            raise RuntimeError("Unhandled")
                    if linkage_name:
                        name = matches_panic_funcs(linkage_name_string)
                        if name:
                            within_core_panic_list.append(panicinfo)
                            continue
                        else:
                            no_info_panic_list.append(panicinfo)
                            print(
                                "Failed to match panic but we probably have enough info to trace it up. Linkage name: {}, addr: {}".format(
                                    linkage_name_string, addr
                                )
                            )
                            continue
                    no_info_panic_list.append(panic_info)
                    print("did not find source for panic: {}".format(addr))
                    continue
            elif abstract_origin:
                origin = abstract_origin_string.split('"')[1]
                panicinfo["abstract_origin"] = origin
                if "core" in origin:
                    if matches_panic_funcs(origin):
                        within_core_panic_list.append(panicinfo)
                        continue
                    no_info_panic_list.append(panicinfo)
                    print(
                        "Probably could add this origin or one of its parents to the panic function list: {}".format(
                            abstract_origin_string
                        )
                    )
                    continue
                else:
                    panicinfo["best_guess_source"] = "abstract_origin + line"
                    panic_list.append(panicinfo)
                    continue
            else:
                # This gets hit for OUTLINED_FUNCTION_XX a bunch on ARM
                try:
                    dw_at_name_string = re.findall(dw_at_name_re, dwarfdump)[
                        -1
                    ].strip()  # see multiple matches for this string sometimes
                    function_name = dw_at_name_string.split('"')[1]
                    if "OUTLINED_FUNCTION_" in function_name:
                        # This is a common pattern where panicing paths are repeated in many
                        # places throughout the binary, and LLVMs optimizer outlines the repeated code.
                        # Let's add these to the list of panicing functions, dynamically so this is resilient to
                        # changes in the binary.
                        if function_name not in panic_functions:
                            # don't double insert
                            panic_functions.append(
                                function_name + ">"
                            )  # so FUNCTION_22 does not catch FUNCTION_222
                        within_core_panic_list.append(panicinfo)
                        continue
                    no_info_panic_list.append(panicinfo)
                    continue
                except:
                    # There seem to be a places where lookup fails completely
                    # Not easy to recover, log these and continue on.
                    no_info_panic_list.append(panicinfo)
                    continue
            raise RuntimeError("BUG: Should not reach here")
    return (panic_list, within_core_panic_list, no_info_panic_list)


def pretty_print(panicinfo):
    if panicinfo["best_guess_source"] == "call/decl":
        try:
            print(
                "\t{} -- {}:{}".format(
                    panicinfo["addr"], panicinfo["call_file"], panicinfo["call_line"]
                )
            )
        except:
            print(
                "\t{} -- in function starting at {}:{}".format(
                    panicinfo["addr"], panicinfo["decl_file"], panicinfo["decl_line"]
                )
            )
    elif panicinfo["best_guess_source"] == "parent":
        print(
            "\t{} -- at or in function starting at {}:{}".format(
                panicinfo["addr"],
                panicinfo["parent_call_file"],
                panicinfo["parent_call_line"],
            )
        )
    elif panicinfo["best_guess_source"] == "lineinfo":
        print(
            "\t{} -- in closure, try: {}".format(
                panicinfo["addr"], panicinfo["line_info"]
            )
        )
    elif panicinfo["best_guess_source"] == "abstract_origin + line":
        print(
            "\t{} -- line_info: {} from origin :{}".format(
                panicinfo["addr"], panicinfo["line_info"], panicinfo["abstract_origin"]
            )
        )
    elif panicinfo["best_guess_source"] == "call-closure-line-info":
        print(
            "\t{} -- in closure starting on line_info: {}".format(
                panicinfo["addr"], panicinfo["line_info"]
            )
        )
    else:
        raise RuntimeError("Missing best guess source: {}".format(panicinfo))


def main():
    args = parse_args()
    if sys.version_info.minor < 7:
        print("This tool requires Python 3.7+")
        return -1
    print("Tock panic report for " + args.ELF)

    objdump = ARM_OBJDUMP
    if args.riscv:
        objdump = RISCV_OBJDUMP

    (panic_list, within_core_panic_list, no_info_panic_list) = find_all_panics(
        objdump, args.ELF, args.riscv
    )
    print("num_panics: {}".format(len(panic_list)))
    buckets_list = {}
    for f in panic_functions:
        buckets_list[f] = []
    for panic in panic_list:
        buckets_list[panic["function"]].append(panic)
    for f, l in buckets_list.items():
        if len(l) > 0:
            print("{}: {}".format(f, len(l)))
        for p in l:
            pretty_print(p)
            if args.verbose:
                print(p)
                print()

    print("num panics in core ignored: {}".format(len(within_core_panic_list)))
    print("num panics for which no info available: {}".format(len(no_info_panic_list)))
    if args.verbose:
        print(
            "If more debug info is needed, run dwarfdump directly on the address in question."
        )


if __name__ == "__main__":
    main()
