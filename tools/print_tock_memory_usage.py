#!/usr/bin/env python3

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

# Prints out the memory usage of a Tock kernel binary ELF.
# Currently only works on ARM binaries.
#
# Usage: print_tock_memory_usage.py ELF
#
# Author: Philip Levis <pal@cs.stanford.edu>

# pylint: disable=superfluous-parens
"""
Script to print out the memory usage of a Tock kernel binary ELF.

Usage: print_memory_usage.py ELF
Options:
  -dn, --depth=n      Group symbols at depth n or greater. E.g.,
                      depth=2 will group all h1b::uart:: symbols
                      together. Default: 1
  -v, --verbose       Print verbose output.
  -s, --show-waste    Show where RAM is wasted (due to padding)
"""

import os
import re
import sys
import getopt
import cxxfilt  # Demangling C++/Rust symbol names
import copy

OBJDUMP = "llvm-objdump"

print_all = False
verbose = False
show_waste = False
symbol_depth = 1
sort_by_size = False  # Otherwise lexicographic order

# A map of section name -> size
sections = {}

# These lists store 5-tuples:
#    (name, start address, reported size, total size, type)
# The "reported size" is the size that the symbol table says this
# symbol has. As a symbol table can have overlapping symbols (e.g.,
# two different symbols with the same address and size) and have space
# that is not covered by symbols (e.g., data embedded after a symbol),
# the reported size is not an accurate accounting of how a symbol impacts
# the size of a binary.
#
# The script calculates the "real" size of symbol and stores it in
# "total size", with the rule that the sum of the total size of all of
# the symbols in a section equal the size of the section. The "total
# size" of a symbol is defined to be the space until the next
# symbol. If there are multiple symbols with the same address, the
# script considers the one with the largest reported size as the last
# one. This means that all of these symbols except the one with the
# largest reported size will have total size 0 (the next symbol has
# the same address). The one with the largest reported size will have
# a total size of the space until the next (different) symbol address.
#
# The "type" field is used to distinguish between variables and
# functions.  This is useful in code sections as it allows the script
# to distinguish constants and embedded data from instructions. As Rust
# can insert a lot of embedded data (e.g., panic strings), distinguishing
# the two is useful. Three types are used:
#   - "variable" denotes an initialized or uninitialized variable.
#   - "data" denotes embedded/constant data in a text section.
#   - "function" denotes instructions/executable code in a text section.

# Uninitialized variables are zeros and zeroed out at kernel boot; they
# exist solely in RAM (the .bss section).
kernel_uninitialized = []
# Initialized variables start with non-zero values and are initialized
# at kernel boot by copying their initial values from flash into RAM
# (the .data section).
kernel_initialized = []
# Kernel symbols are first stored in kernel_text to perform whole
# kernel text calculations. Then they are split into kernel_functions
# and kernel_data to separately account for symbols of these types.
kernel_text = []
kernel_functions = []
kernel_data = []


def usage(message):
    """Prints out an error message and usage"""
    if message != "":
        print("error: " + message)
    print(
        """Usage: print_memory_usage.py ELF
Options:
  -a                  Print all symbols (overrides -d)
  -dn, --depth=n      Group symbols at depth n or greater. E.g.,
                      depth=2 will group all h1b::uart:: symbols
                      together. Default: 1
  -s, --size          Sort symbols by size (normally lexicographic)
  -v, --verbose       Print verbose output (RAM waste and embedded flash data)
  -w, --show-waste    Show where RAM is wasted (due to padding)
      --objdump       Path to the llvm-objdump executable

Note: depends on llvm-objdump to extract symbols"""
    )

def is_private_symbol(symbol):
    """Returns whether a symbol is a private symbol. Private symbols
    are inserted by the compiler and denote internal structure: they
    are not included when attributing space to symbols."""
    if symbol[0:4] == ".LBB":
        return True
    elif symbol[0:4] == ".LBE":
        return True
    elif symbol[0:5] == ".Ltmp":
        return True
    else:
       return False


def process_section_line(line):
    """Parses a line from the Sections: header of an ELF objdump,
    inserting it into a data structure keeping track of the sections."""
    # pylint: disable=anomalous-backslash-in-string,line-too-long
    match = re.search("^\S+\s+\.(text|relocate|sram|stack|app_memory)\s+(\S+).+", line)
    if match != None:
        sections[match.group(1)] = int(match.group(2), 16)


# Take a Rust-style symbol of '::' delineated names and trim the last
# one if it is a hash.  Many symbols have hashes appended which just
# hurt readability; they take the form of h[16-digit hex number].
def trim_hash_from_symbol(symbol):
    """If the passed symbol ends with a hash of the form h[16-hex number]
    trim this and return the trimmed symbol."""
    # Remove the hash off the end
    tokens = symbol.split("::")
    last = tokens[-1]
    if last[0] == "h":
        tokens = tokens[:-1]  # Trim off hash if it exists
        trimmed_name = "::".join(tokens)  # reassemble
        return trimmed_name
    else:
        return symbol


escape_sequences = [
    ["$C$", ","],
    ["$SP$", "@"],
    ["$BP$", "*"],
    ["$RF$", "&"],
    ["$LT,GT$", "<>"],
    ["$LT$", "<"],
    ["$GT$", ">"],
    ["$LP$", "("],
    ["$RP$", ")"],
    ["$u20$", " "],
    ["$u27$", "'"],
    ["$u5b$", "["],
    ["$u5d$", "]"],
    ["..", "::"],
    [".", "-"],
]


def parse_mangled_name(name):
    """Take a potentially mangled symbol name and demangle it to its
    name, removing the trailing hash. This is not just a simple
    demangling: for methods, it outputs the structure + method
    as a :: separated name, eliding the trait (if any)."""

    # Not a mangled name, just return it unchanged.
    if name[0:3] != "_ZN":
        return name

    # Trim a trailing . number (e.g., ".71") which breaks demangling
    match = re.search("\.\d+$", name)
    if match != None:
        name = name[: match.start()]

    # Trim a trailing ".llvm", which breaks demangling
    match = re.search("\.llvm", name)
    if match != None:
        name = name[: match.start()]

    demangled = ""
    try:
        demangled = cxxfilt.demangle(name, external_only=False)
    except cxxfilt.InvalidName:
        demangled = name

    corrected_name = trim_hash_from_symbol(demangled)
    for escape in escape_sequences:
        corrected_name = corrected_name.replace(escape[0], escape[1])

    # Need to separate the name of the structure from the name of
    # the method. If it starts with a _, then it's of the form
    # _<structure as trait>::method otherwise it's
    # structure::method. So first carve off the method name, then
    # figure out the structure.

    structure_end = corrected_name.rfind("::")
    full_structure_name = ""

    if structure_end >= 0:
        method = corrected_name[structure_end + 2 :]
        full_structure_name = corrected_name[0:structure_end]
    else:
        method = corrected_name

    structure = full_structure_name
    if corrected_name[0:1] == "_":
        split = full_structure_name.split(" as ")
        structure = split[0]
        # trim the _<
        structure = structure[2:]

    symbol = structure
    if len(symbol) > 0:
        symbol = symbol + "::" + method
    else:
        # No structure, just a method
        symbol = method

    if symbol[0:2] == "-L" or symbol[0:2] == "-l" or symbol[0:4] == "anon":
        symbol = "Anonymous"
    if symbol[0:7] == "-hidden":
        symbol = "Hidden"

    return symbol


def process_symbol_line(line):
    """Parse a line the SYMBOL TABLE section of the objdump output and
    insert its data into one of the three kernel_ symbol lists.
    Because Tock executables have a variety of symbol formats,
    first try to demangle it; if that fails, use it as is."""
    # pylint: disable=line-too-long,anomalous-backslash-in-string
    global kernel_text
    global kernel_initialized
    global kernel_uninitialized
    match = re.search(
        "^(\S+)\s+(\w*)\s+(\w*)\s+\.(text|relocate|sram|stack|app_memory)\s+(\S+)\s+(.+)",
        line,
    )
    if match != None:
        addr = int(match.group(1), 16)
        linkage = match.group(2)
        symbol_type = match.group(3)
        segment = match.group(4)
        size = int(match.group(5), 16)
        name = match.group(6)

        # Compiler embeds these symbols, ignore them
        if name[0:3] == "$t." or name[0:3] == "$d.":
            return

        # Special case end of kernel RAM, given that there is padding
        # between it and application RAM.
        if name == "_ezero":
            name = "Padding at end of kernel RAM"

        # Initialized data: part of the flash image, then copied into RAM
        # on start. The .data section in normal hosted C.
        if segment == "relocate":
            demangled = parse_mangled_name(name)
            kernel_initialized.append((demangled, addr, size, 0, "variable"))

        # Uninitialized data, stored in a zeroed RAM section. The
        # .bss section in normal hosted C.
        elif segment == "sram":
            demangled = parse_mangled_name(name)
            kernel_uninitialized.append((demangled, addr, size, 0, "variable"))

        # Code and embedded data.
        elif segment == "text":
            match = re.search("\$(((\w+\.\.)+)(\w+))\$", name)
            # It's a function
            if is_private_symbol(name):
                # Skip this symbol
                return
            if symbol_type == "F" or symbol_type == "f":
                try:
                    symbol = parse_mangled_name(name)
                    kernel_text.append((symbol, addr, size, 0, "function"))
                except cxxfilt.InvalidName:
                    kernel_text.append((name, addr, size, 0, "function"))
            else:
                try:
                    symbol = parse_mangled_name(name)
                    kernel_text.append((symbol, addr, size, 0, "data"))
                except cxxfilt.InvalidName:
                    kernel_text.append((name, addr, size, 0, "data"))


def print_section_information():
    """Print out the ELF's section information (RAM and Flash use)."""
    text_size = sections["text"]
    stack_size = sections["stack"]
    relocate_size = sections["relocate"]
    sram_size = sections["sram"]
    app_size = 0
    if "app_memory" in sections:  # H1B-style linker file, static app section
        app_size = sections["app_memory"]
    else:  # Mainline Tock-style linker file, using APP_MEMORY
        for (name, addr, size, tsize, desc) in kernel_uninitialized:
            if name.find("APP_MEMORY") >= 0:
                app_size = size

    flash_size = text_size + relocate_size
    ram_size = stack_size + sram_size + relocate_size

    print("Kernel occupies " + str(flash_size) + " bytes of flash")
    print("  " + "{:>6}".format(text_size) + "\tcode and constant strings")
    print("  " + "{:>6}".format(relocate_size) + "\tvariable initializers")
    print("Kernel occupies " + str(ram_size) + " bytes of RAM")
    print("  " + "{:>6}".format(stack_size) + "\tstack")
    print("  " + "{:>6}".format(sram_size) + "\tuninitialized variables")
    print("  " + "{:>6}".format(relocate_size) + "\tinitialized variables")
    print("  " + "{:>6}".format(sram_size + relocate_size) + "\tvariables total")
    print("Applications allocated " + str(app_size) + " bytes of RAM")


# Take a list of 'symbols' and group them into in 'groups' as aggregates
# for condensing. Names are '::' delimited hierarchies. The aggregate
# sizes are determined by the global symbol depth, which indicates how
# many levels of the naming heirarchy to display. A depth of 0 means
# group all symbols together into one category; a depth of 1 means
# aggregate symbols into top level categories (e.g, 'h1b::*'). A depth
# of 100 means aggregate symbols only if they have the same first 100
# name levels, so effectively print every symbol individually.
#
# The 'waste' and 'section' parameters are used to specify whether detected
# waste should be printed and the name of the section for waste information.
#
# Returns a string representation of any detected waste. This is returned
# as a string to it can be later output.
def group_symbols(groups, symbols, waste, section):
    """Take a list of symbols and group them into 'groups' for reporting
    aggregate flash/RAM use."""
    global symbol_depth
    output = ""
    expected_addr = 0
    waste_sum = 0
    prev_symbol = ""
    for (symbol, addr, size, real_size, _) in symbols:
        # If we find a gap between symbol+size and the next symbol, we might
        # have waste. But this is only true if it's not the first symbol and
        # this is actually a variable and just just a symbol (e.g., _estart)
        if (
            addr != expected_addr
            and expected_addr != 0
            and size != 0
            and (waste or verbose)
        ):
            output = (
                output
                + "   ! "
                + str(addr - expected_addr)
                + " bytes of data or padding after "
                + prev_symbol
                + "\n"
            )
            waste_sum = waste_sum + (addr - expected_addr)
        tokens = symbol.split("::")
        key = symbol[0]  # Default to first character (_) if not a proper symbol
        name = symbol

        if len(tokens) == 1:
            # The symbol isn't a standard mangled Rust name. These rules are
            # based on observation.
            # .Lanon* and str.* are embedded string.
            if (
                symbol[0:6] == ".Lanon"
                or symbol[0:5] == "anon."
                or symbol[0:4] == "str."
            ):
                key = "Constant strings"
            elif symbol[0:8] == ".hidden ":
                key = "ARM aeabi support"
            elif symbol[0:3] == "_ZN":
                key = "Unidentified auto-generated"
            elif symbol == "Padding at end of kernel RAM":
                key = symbol
                name = symbol
            else:
                key = "Unmangled globals (C-like code)"
                name = symbol
        else:
            # Packages have a trailing :: while other categories don't;
            # this allows us to disambiguate when * is relevant or not
            # in printing.
            key = "::".join(tokens[0:symbol_depth])
            name = ""

            if len(tokens[symbol_depth:]) > 0:
                key = key + "::"
                name = "::".join(tokens[symbol_depth:])
        if key in groups.keys():
            groups[key].append((name, real_size))
        else:
            groups[key] = [(name, real_size)]

        # Set state for next iteration
        expected_addr = addr + size
        prev_symbol = symbol

    if waste and waste_sum > 0:
        output = (
            output + "Total of " + str(waste_sum) + " bytes wasted in " + section + "\n"
        )

    return output


def string_for_group(key, padding_size, group_size, num_elements):
    """Return the string for a group of variables, with padding added on the
    right; decides whether to add a * or not based on the name of the group
    and number of elements in it."""
    if key[-2:] == "::":
        key = key + "*"
        key = key.ljust(padding_size + 2, " ")
        return "  " + key + str(group_size) + " bytes\n"
    else:
        key = key + ""
        key = key.ljust(padding_size + 2, " ")
        return "  " + key + str(group_size) + " bytes\n"


def print_groups(title, groups):
    """Print title, then all of the variable groups in groups."""
    group_sum = 0
    output = ""
    max_string_len = 0
    if len(groups.keys()) > 0:
        max_string_len = len(max(groups.keys(), key=len))
    group_sizes = {}

    for key in groups.keys():
        symbols = groups[key]

        group_size = 0
        for (name, size) in symbols:
            group_size = group_size + size
        group_sizes[key] = group_size

    if sort_by_size:
        for k, v in sorted(group_sizes.items(), key=lambda item: item[1], reverse=True):
            group_size = v
            symbols = groups[key]
            output = output + string_for_group(
                k, max_string_len, group_size, len(symbols)
            )
            group_sum = group_sum + group_size
    else:
        for key in sorted(group_sizes.keys()):
            group_size = group_sizes[key]
            symbols = groups[key]
            output = output + string_for_group(
                key, max_string_len, group_size, len(symbols)
            )
            group_sum = group_sum + group_size

    print(title + ": " + str(group_sum) + " bytes")
    print(output, end="")

def split_text_into_data_and_functions():
    global kernel_text
    global kernel_functions
    global kernel_data
    for (name, addr, reported_size, real_size, desc) in kernel_text:
        if desc == "function":
            kernel_functions.append((name, addr, reported_size, real_size, desc))
        elif desc == "data":
            kernel_data.append((name, addr, reported_size, real_size, desc))


def print_symbol_information():
    """Print out all of the variable and function groups with their flash/RAM
    use."""
    split_text_into_data_and_functions()
    if print_all:
        print_all_symbol_information()
    else:
        print_grouped_symbol_information()

def print_all_symbols(title, symbols):
    """Print out all of the symbols passed as a list of 4-tuples,
    prefaced by the title and total size of the of symbols."""
    max_string_len = 0
    max_byte_len = 5
    if len(symbols) > 0:
        max_string_len = max(len(s) for (s, _, _, _, _) in symbols)
#        max_byte_len = max(len(str(size)) for (_, _, _, size, _) in symbols)
    output = ""
    symbol_sum = 0
    if sort_by_size:
        symbols = sorted(symbols, key=lambda item: item[3], reverse=True)
    for (name, addr, reported_size, real_size, desc) in symbols:
        name = name.ljust(max_string_len + 2, " ")
        symbol_sum = symbol_sum + real_size
        size_str = str(real_size).rjust(max_byte_len, " ")
        output = output + "  " + name + size_str + " bytes\n"
    print(title + ": " + str(symbol_sum) + " bytes")
    print(output, end="")


def print_all_symbol_information():
    """Print out the size of every symbol."""
    print_all_symbols("Initialized variable groups (Flash+RAM)", kernel_initialized)
    print()
    print_all_symbols("Variable groups (RAM)", kernel_uninitialized)
    print()
    print_all_symbols("Function groups (flash)", kernel_functions)
    print()
    print_all_symbols("Embedded data (flash)", kernel_data)
    print()
    print_all_symbols("Padding within functions and embedded data (flash)", padding_text)
    print()


def print_grouped_symbol_information():
    """Print out the size taken up by symbols, with symbols grouped
    by their names"""
    initialized_groups = {}
    gaps = group_symbols(initialized_groups, kernel_initialized, show_waste, "Flash+RAM")
    print_groups("Initialized variable groups (Flash+RAM)", initialized_groups)
    print()

    uninitialized_groups = {}
    gaps = gaps + group_symbols(
        uninitialized_groups, kernel_uninitialized, show_waste, "RAM"
    )
    print_groups("Variable groups (RAM)", uninitialized_groups)
    print(gaps)

    function_groups = {}
    gaps = group_symbols(function_groups, kernel_functions, show_waste, "Flash")
    print_groups("Function groups (flash)", function_groups)
    print(gaps)

    embedded_data = {}
    gaps = group_symbols(embedded_data, kernel_data, show_waste, "Flash")
    print_groups("Embedded data (flash)", embedded_data)
    print(gaps)

    padding = {}
    gaps = group_symbols(padding, padding_text, False, "Flash")
    print_groups("Padding within functions and embedded padding (flash)", padding)
    print(gaps)


def sort_value(symbol_entry):
    """Helper function for sorting symbols by start address and size."""
    value = (symbol_entry[1] << 16) + symbol_entry[2]
    return value

def get_addr(symbol_entry):
    """Helper function for sorting symbols by start address."""
    return symbol_entry[1]

def get_name(symbol_entry):
    """Helper function for fetching symbol names to calculate longest name."""
    return symbol_entry[0]

text_total = 0
def compute_padding(symbols, text):
    """Calculate how much padding is in a list of symbols by comparing their
    reporting size with the spacing with the next function and return
    the total differences."""
    global text_total
    symbols.sort(key=sort_value)
    elements = []
    func_count = len(symbols)
    diff = 0
    size_sum = 0
    for i in range(1, func_count):
        (esymbol, eaddr, esize, _, edesc) = symbols[i - 1]
        (symbol, laddr, _, _, desc) = symbols[i]
        total_size = laddr - eaddr
#        print("PROCESSED: ", esymbol, "has size", esize, "but real size is", total_size, "total is", text_total, "comes before", symbol)
        symbols[i - 1] = (esymbol, eaddr, esize, total_size, edesc)

        size_sum = size_sum + total_size
        padding_size = (total_size - esize)
        # Padding represents when there is space after the end of a symbol's
        # defined size. Sometimes, when there is embedded data, it is in space
        # after a symbol, i.e., there is space after symbol+length and the
        # next symbol. Other times, the embedded data is within a symbol's
        # region.
        if total_size != esize and total_size > 0 and padding_size > 0:
            elements.append((esymbol, 0, padding_size, padding_size, edesc))
            diff = diff + padding_size
        #if total_size != 0:
        #    print(esymbol, total_size)
    return elements


def parse_options(opts):
    """Parse command line options."""
    global print_all, symbol_depth, verbose, show_waste, sort_by_size, OBJDUMP
    valid = "ad:vsw"
    long_valid = ["depth=", "verbose", "show-waste", "size", "objdump="]
    optlist, leftover = getopt.getopt(opts, valid, long_valid)
    for (opt, val) in optlist:
        if opt == "-a":
            print_all = True
        elif opt == "-d" or opt == "--depth":
            symbol_depth = int(val)
        elif opt == "-v" or opt == "--verbose":
            verbose = True
        elif opt == "-w" or opt == "--show-waste":
            show_waste = True
        elif opt == "-s" or opt == "--size":
            sort_by_size = True
        elif opt == "--objdump":
            OBJDUMP = val
        else:
            usage("unrecognized option: " + opt)
            return []

    return leftover

# Script starts here ######################################
if __name__ == "__main__":
    arguments = sys.argv[1:]
    if len(arguments) < 1:
        usage("no ELF specified")
        sys.exit(-1)

    # The ELF is always the last argument; pull it out, then parse
    # the others.
    elf_name = ""
    options = arguments
    try:
        remaining = parse_options(options)
        if len(remaining) != 1:
            usage("")
            sys.exit(-1)
        else:
            elf_name = remaining[0]
    except getopt.GetoptError as err:
        usage(str(err))
        sys.exit(-1)

    header_lines = os.popen(OBJDUMP + " --section-headers " + elf_name).readlines()

    print("Tock memory usage report for " + elf_name)
    arch = "UNKNOWN"

    for hline in header_lines:
        # pylint: disable=anomalous-backslash-in-string
        hmatch = re.search("file format (\S+)", hline)
        if hmatch != None:
            arch = hmatch.group(1)

    if arch == "UNKNOWN":
        usage("could not detect architecture of ELF")
        sys.exit(-1)

    objdump_lines = os.popen(OBJDUMP + " -t --section-headers " + elf_name).readlines()
    objdump_output_section = "start"

    for oline in objdump_lines:
        oline = oline.strip()
        # First, move to a new section if we've reached it; use continue
        # to break out and reduce nesting.
        if oline == "Sections:":
            objdump_output_section = "sections"
            continue
        elif oline == "SYMBOL TABLE:":
            objdump_output_section = "symbol_table"
            continue
        elif objdump_output_section == "sections":
            process_section_line(oline)
        elif objdump_output_section == "symbol_table":
            process_symbol_line(oline)

    padding_init = compute_padding(kernel_initialized, False)
    padding_uninit = compute_padding(kernel_uninitialized, False)
    padding_text = compute_padding(kernel_text, True)

    print_section_information()
    print()
    print_symbol_information()
