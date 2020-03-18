#!/usr/bin/env python3

# Copyright 2019 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

#
# Prints out the memory usage of a Tock kernel binary ELF.
# Currently only works on ARM binaries.
#
# Usage: print_tock_memory_usage.py ELF
#
# Author: Philip Levis <philip.levis@gmail.com>

# pylint: disable=superfluous-parens
'''
Script to print out the memory usage of a Tock kernel binary ELF.

Usage: print_memory_usage.py ELF
Options:
  -dn, --depth=n      Group symbols at depth n or greater. E.g.,
                      depth=2 will group all h1b::uart:: symbols
                      together. Default: 1
  -v, --verbose       Print verbose output.
  -s, --show-waste    Show where RAM is wasted (due to padding)
'''

import os
import re
import sys
import getopt
import cxxfilt   # Demangling C++/Rust symbol names


OBJDUMP = "arm-none-eabi-objdump"

verbose = False
show_waste = False
symbol_depth = 1

# A map of section name -> size
sections = {}

# These lists store 4-tuples:
#    (name, start address, length of function, total size)
# The "length of function" is the size of the symbol as reported in
# objdump, which is the executable code. "Total size" includes any
# constants embedded, including constant strings, or padding.
# Initially the lists are populated with total_size=0; it is later
# computed by sorting the symbols and calculating their spacing.
kernel_uninitialized = []
kernel_initialized = []
kernel_functions = []

def usage(message):
    """Prints out an error message and usage"""
    if message != "":
        print("error: " + message)
    print("""Usage: print_memory_usage.py ELF
Options:
  -dn, --depth=n      Group symbols at depth n or greater. E.g.,
                      depth=2 will group all h1b::uart:: symbols
                      together. Default: 1
  -v, --verbose       Print verbose output (RAM waste and embedded flash data)
  -s, --show-waste    Show where RAM is wasted (due to padding)""")



def process_section_line(line):
    """Parses a line from the Sections: header of an ELF objdump,
       inserting it into a data structure keeping track of the sections."""
    # pylint: disable=anomalous-backslash-in-string,line-too-long
    match = re.search('^\S+\s+\.(text|relocate|sram|stack|app_memory)\s+(\S+).+', line)
    if match != None:
        sections[match.group(1)] = int(match.group(2), 16)

 # Take a Rust-style symbol of '::' delineated names and trim the last
 # one if it is a hash.  Many symbols have hashes appended which just
 # hurt readability; they take the form of h[16-digit hex number].
def trim_hash_from_symbol(symbol):
    """If the passed symbol ends with a hash of the form h[16-hex number]
       trim this and return the trimmed symbol."""
    # Remove the hash off the end
    tokens = symbol.split('::')
    last = tokens[-1]
    if last[0] == 'h':
        tokens = tokens[:-1] # Trim off hash if it exists
        trimmed_name = "::".join(tokens) # reassemble
        return trimmed_name
    else:
        return symbol

def parse_mangled_name(name):
    """Take a potentially mangled symbol name and demangle it to its
       name, removing the trailing hash. Raise a cxxflit.InvalidName exception
       if it is not a mangled symbol."""
    demangled = cxxfilt.demangle(name, external_only=False)
    corrected_name = trim_hash_from_symbol(demangled)
    # Rust-specific mangled names triggered by Tock Components, e.g.
    # ZN100_$LT$capsules..ieee802154..driver..RadioDriver$u20$as$u20$capsules..ieee802154..device..RxClient$GT$7receive
    # This name has two parts: the structure, then the trait method it is
    # implementing. This code parses only the structure name, so all
    # methods that are trait implementations are just clumped under the
    # name of the structure. -pal
    if corrected_name[0:5] == "_$LT$":
        # Trim off the _$LT$, then truncate at next $, this will extract
        # capsules..ieee802154..driver..RadioDriver
        corrected_name = corrected_name[5:]
        endpos = corrected_name.find("$")
        if endpos > 0:
            corrected_name = corrected_name[0:endpos]

    return corrected_name

def process_symbol_line(line):
    """Parse a line the SYMBOL TABLE section of the objdump output and
       insert its data into one of the three kernel_ symbol lists.
       Because Tock executables have a variety of symbol formats,
       first try to demangle it; if that fails, use it as is."""
    # pylint: disable=line-too-long,anomalous-backslash-in-string
    match = re.search('^(\S+)\s+\w+\s+\w*\s+\.(text|relocate|sram|stack|app_memory)\s+(\S+)\s+(.+)', line)
    if match != None:
        addr = int(match.group(1), 16)
        segment = match.group(2)
        size = int(match.group(3), 16)
        name = match.group(4)

        # Initialized data: part of the flash image, then copied into RAM
        # on start. The .data section in normal hosted C.
        if segment == "relocate":
            try:
                demangled = parse_mangled_name(name)
                kernel_initialized.append((demangled, addr, size, 0))
            except cxxfilt.InvalidName:
                kernel_initialized.append((name, addr, size, 0))

        # Uninitialized data, stored in a zeroed RAM section. The
        # .bss section in normal hosted C.
        elif segment == "sram":
            try:
                demangled = parse_mangled_name(name)
                kernel_uninitialized.append((demangled, addr, size, 0))
            except cxxfilt.InvalidName:
                kernel_uninitialized.append((name, addr, size, 0))

        # Code and embedded data.
        elif segment == "text":
            # pylint: disable=anomalous-backslash-in-string
            match = re.search('\$(((\w+\.\.)+)(\w+))\$', name)
            if match != None:
                symbol = match.group(1)
                symbol = symbol.replace('..', '::')
                symbol = trim_hash_from_symbol(symbol)
                kernel_functions.append((symbol, addr, size, 0))
            else:
                try:
                    symbol = parse_mangled_name(name)
                    kernel_functions.append((symbol, addr, size, 0))
                except cxxfilt.InvalidName:
                    kernel_functions.append((name, addr, size, 0))

def print_section_information():
    """Print out the ELF's section information (RAM and Flash use)."""
    text_size = sections["text"]
    stack_size = sections["stack"]
    relocate_size = sections["relocate"]
    sram_size = sections["sram"]
    app_size = 0
    if "app_memory" in sections:  # H1B-style linker file, static app section
        app_size = sections["app_memory"]
    else: # Mainline Tock-style linker file, using APP_MEMORY
        for (name, addr, size, tsize) in kernel_uninitialized:
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
    for (symbol, addr, size, _) in symbols:
        if size == 0:
            continue
        # If we find a gap between symbol+size and the next symbol, we might
        # have waste. But this is only true if it's not the first symbol and
        # this is actually a variable and just just a symbol (e.g., _estart)
        if addr != expected_addr and expected_addr != 0 and size != 0 and (waste or verbose):
            output = output + "   ! " + str(addr - expected_addr) + " bytes of data or padding after " + prev_symbol + "\n"
            waste_sum = waste_sum + (addr - expected_addr)
        tokens = symbol.split("::")
        key = symbol[0] # Default to first character (_) if not a proper symbol
        name = symbol

        if len(tokens) == 1:
            # The symbol isn't a standard mangled Rust name. These rules are
            # based on observation.
            # .Lanon* and str.* are embedded string.
            if symbol[0:6] == '.Lanon' or symbol[0:5] == "anon." or symbol[0:4] == 'str.':
                key = "Constant strings"
            elif symbol[0:8] == ".hidden ":
                key = "ARM aeabi support"
            elif symbol[0:3] == "_ZN":
                key = "Unidentified auto-generated"
            else:
                key = "Unmangled globals (C-like code)"
                name = symbol
        else:
            # Packages have a trailing :: while other categories don't;
            # this allows us to disambiguate when * is relevant or not
            # in printing.
            key = "::".join(tokens[0:symbol_depth]) + "::"
            name = "::".join(tokens[symbol_depth:])

            if key in groups.keys():
                groups[key].append((name, size))
            else:
                groups[key] = [(name, size)]

        # Set state for next iteration
        expected_addr = addr + size
        prev_symbol = symbol

    if waste and waste_sum > 0:
        output = output + "Total of " + str(waste_sum) + " bytes wasted in " + section + "\n"
        
    return output
        
def string_for_group(key, padding_size, group_size, num_elements):
    """Return the string for a group of variables, with padding added on the
       right; decides whether to add a * or not based on the name of the group
       and number of elements in it."""
    if num_elements == 1: # If there's a single symbol (a variable), print it.
        key = key[:-2]
        key = key + ":"
        key = key.ljust(padding_size + 2, ' ')
        return ("  " + key + str(group_size) + " bytes\n")
    else: # If there's more than one, print the key as a namespace
        if key[-2:] == "::":
            key = key + "*"
            key = key.ljust(padding_size + 2, ' ')
            return ("  " + key + str(group_size) + " bytes\n")
        else:
            key = key + ":"
            key = key.ljust(padding_size + 2, ' ')
            return ("  " + key + str(group_size) + " bytes\n")

def print_groups(title, groups):
    """Print title, then all of the variable groups in groups."""
    group_sum = 0
    output = ""
    max_string_len = len(max(groups.keys(), key=len))
    for key in sorted(groups.keys()):
        symbols = groups[key]

        group_size = 0
        for (_, size) in symbols:
            group_size = group_size + size

        output = output + string_for_group(key, max_string_len, group_size, len(symbols))
        group_sum = group_sum + group_size

    print(title + ": " + str(group_sum) + " bytes")
    print(output, end = '')

def print_symbol_information():
    """Print out all of the variable and function groups with their flash/RAM
       use."""
    variable_groups = {}
    gaps = group_symbols(variable_groups, kernel_initialized, show_waste, "RAM")
    gaps = gaps + group_symbols(variable_groups, kernel_uninitialized, show_waste, "Flash+RAM")
    print_groups("Variable groups (RAM)", variable_groups)
    print(gaps)
    
    print("Embedded data (flash): " + str(padding_text) + " bytes")
    print()
    function_groups = {}
    # Embedded constants in code (e.g., after functions) aren't counted
    # in the symbol's size, so detecting waste in code has too many false
    # positives.
    gaps = group_symbols(function_groups, kernel_functions, False, "Flash")
    print_groups("Function groups (flash)", function_groups)
    print(gaps)

def get_addr(symbol_entry):
    """Helper function for sorting symbols by start address."""
    return symbol_entry[1]

def compute_padding(symbols):
    """Calculate how much padding is in a list of symbols by comparing their
       reporting size with the spacing with the next function and return
       the total differences."""
    symbols.sort(key=get_addr)
    func_count = len(symbols)
    diff = 0
    for i in range(1, func_count):
        (esymbol, eaddr, esize, _) = symbols[i - 1]
        (_, laddr, _, _) = symbols[i]
        total_size = laddr - eaddr
        symbols[i - 1] = (esymbol, eaddr, esize, total_size)
        if total_size != esize:
            diff = diff + (total_size - esize)

    return diff

def parse_options(opts):
    """Parse command line options."""
    global symbol_depth, verbose, show_waste
    valid = 'd:vs' 
    long_valid = ['depth=', 'verbose', 'show-waste']
    optlist, leftover = getopt.getopt(opts, valid, long_valid)
    for (opt, val) in optlist:
        if opt == '-d' or opt == '--depth':
            symbol_depth = int(val)
        elif opt == '-v' or opt == '--verbose':
            verbose = True
        elif opt == '-s' or opt == '--show-waste':
            show_waste = True
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

    header_lines = os.popen(OBJDUMP + ' -f ' + elf_name).readlines()

    print("Tock memory usage report for " + elf_name)
    arch = "UNKNOWN"

    for hline in header_lines:
        # pylint: disable=anomalous-backslash-in-string
        hmatch = re.search('file format (\S+)', hline)
        if hmatch != None:
            arch = hmatch.group(1)
            if arch != 'elf32-littlearm':
                usage(arch + " architecture not supported, only elf32-littlearm supported")
                sys.exit(-1)

    if arch == "UNKNOWN":
        usage("could not detect architecture of ELF")
        sys.exit(-1)

    objdump_lines = os.popen(OBJDUMP + ' -x ' + elf_name).readlines()
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

    padding_init = compute_padding(kernel_initialized)
    padding_uninit = compute_padding(kernel_uninitialized)
    padding_text = compute_padding(kernel_functions)

    print_section_information()
    print()
    print_symbol_information()
