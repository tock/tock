#! /usr/bin/env python3

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

from audioop import add
from embedded_data_visualizer import create_html_file
from print_tock_memory_usage import parse_mangled_name
import argparse
from collections import defaultdict, namedtuple
import os
import sys
import re
HELP_STRING = '''
Tock embedded data analysis tool

Generates a report of a Tock binary's embedded data usage in the form of a static HTML webpage.

Prerequisite:
    - The binary must be compiled for the RISC-V architecture.
    - Turn off LTO. To do so, set lto to false in the top-level Cargo.toml file.

To run the tool, in the Tock home directory invoke:

    $ tools/embedded_data_analyzer.py -e <elf_path> -n <name> -o <output_directory> [-d <objdump>]

arguments:
    elf_path         -- the relative path of the compiled elf file
    name             -- a name for the binary to be displayed in the HTML page
    output_directory -- where the user wishes the output to be stored
    objdump          -- (optional) path of the riscv objdump binary to use

You can also run the script with -h flag for help on what the arguments are.
'''


def get_args():
    # To allow for a custom help message, the default help option is turned off,
    # this means that required arguments must be checked manually
    ap = argparse.ArgumentParser(add_help=False)
    ap.add_argument('-e', '--elf_path', required=False)
    ap.add_argument('-o', '--output_directory', required=False)
    ap.add_argument('-n', '--name', required=False)
    ap.add_argument('-h', '--help', required=False,
                    action='store_true', default=False)
    ap.add_argument('-d', '--objdump', required=False)
    return vars(ap.parse_args())


def find_objdump():
    objdump = os.popen('which riscv32-unknown-elf-objdump').read().strip()
    if objdump != '':
        return objdump

    objdump = os.popen('which riscv64-unknown-elf-objdump').read().strip()
    if objdump != '':
        return objdump

    objdump = os.popen('which riscv32-none-elf-objdump').read().strip()
    if objdump != '':
        return objdump

    objdump = os.popen('which riscv64-none-elf-objdump').read().strip()
    if objdump != '':
        return objdump

    critical_error(
        'Objdump not found, please specify a riscv objdump path manually with -d or --objdump.')


FunctionInfo = namedtuple(
    'FunctionInfo', 'embedded_data_size_actual embedded_data_size_estimated embedded_data_count addresses')
SymbolInfo = namedtuple('SymbolInfo', 'actual_size estimated_size name')


def get_srodata_address(disasm):
    pattern = '<_srodata>:'
    for i in range(len(disasm)):
        match = re.findall(pattern, disasm[i])
        if match:
            return disasm[i].split()[0], i
    critical_error('Invalid binary, cannot find <_srodata> section')


def map_adress_to_data(disasm):
    _, start_index = get_srodata_address(disasm)
    sro_address, sro_end = get_sro_range(disasm)
    address_pattern = r'^\s*([0-9a-f]+):'
    data_pattern = r'[ |\t][0-9a-f]{4}'

    sro_data = ""
    for i in range(start_index, len(disasm)):
        match = re.search(address_pattern, disasm[i])
        if match:
            address = int(match.group(1), 16)
            if (sro_address <= address < sro_end):
                data = re.findall(data_pattern, disasm[i])
                if data:
                    # remove the \t character from first byte
                    data[0] = re.sub('\s+', '', data[0])
                    for j in range(len(data)):
                        data[j] = data[j].strip()
                        # seperate the bytes
                        sro_data += bytes.fromhex(data[j][2:]
                                                  ).decode("utf-8", errors='replace')
                        sro_data += bytes.fromhex(data[j][0:2]
                                                  ).decode("utf-8", errors='replace')

    return sro_data


def get_sro_range(disasm):
    data_pattern = r'[ |\t][0-9a-f]{4}'
    address_pattern = r'^[0-9a-f]{1,16}'
    sro_string, start_index = get_srodata_address(disasm)
    sro_address = int(sro_string, 16)
    previous_blank = False
    last_address_seen = 0
    last_line = -1
    for i in range(start_index, len(disasm)):
        if (disasm[i] == '\n'):
            previous_blank = True
        else:
            match = re.findall(address_pattern, disasm[i])
            if match:
                last_address_seen = int(match[0], 16)
                last_line = i
                previous_blank = False
            elif previous_blank:
                # if the last line was blank and current line has no address data has ended
                break

    # address points to the beginning of the line of data
    # must also add the length of data in the line itself
    if last_line != -1:
        data = re.findall(data_pattern, disasm[last_line])
        last_address_seen += len(data)*2

    return sro_address, last_address_seen


def filter_symbol_table(disasm, symbols_table):
    """
    Given the path to a symbols table,
    trim the table to only contain embedded data objects,
    save the trimeed table to a file,
    and also return the trimmed table as a list of strings.
    """
    sro_start, sro_end = get_sro_range(disasm)

    # trim garbage lines at the beginning and end
    trimmed_lines = symbols_table[4:-4]
    filtered = []
    for line in trimmed_lines:
        # if it is an object, and it's in the code,
        # and the address is within the range of srodata
        address = int(line[:8], 16)
        if address >= sro_start and address < sro_end:
            filtered.append(line)

    return filtered


def estimate_empty_symbols(symbols_table):
    items_list = list(symbols_table.items())
    items_list.sort()
    res = {}
    for i in range(len(items_list)):
        address = items_list[i][0]
        actual_size = items_list[i][1][0]
        name = items_list[i][1][1]
        estimated_size = actual_size
        if (estimated_size == 0) and (i != len(items_list) - 1):
            next_address = items_list[i+1][0]
            estimated_size = next_address - address
        res[address] = SymbolInfo(
            actual_size=actual_size, estimated_size=estimated_size, name=name)
    return res


def build_symbols_dict(symbols_table):
    mapping = {}  # address of symbol -> (size, name)
    for line in symbols_table:
        if '.debug' in line:
            continue

        matches = re.search(
            '^([0-9a-f]+)\s+[lg]\s+O?\s+\.text\s+([0-9a-f]+)\s+(.*)', line)
        address = int(matches.group(1), 16)
        size = int(matches.group(2), 16)
        name = matches.group(3)

        if address not in mapping:
            mapping[address] = (size, name)
        else:
            # replace zero-sized with non-zero-sized
            if size == 0:
                continue
            if mapping[address][0] == 0:
                mapping[address] = (size, name)
                continue

            critical_error(
                f'address 0x{address:x} mapped already: {mapping[address]}')

    return estimate_empty_symbols(mapping)


def trace_function(disasm, line_num):
    # pattern to detect if there's a function in a line
    func_pattern = r'^[0-9a-f]{1,16} <.*>:\n'
    for i in range(line_num, -1, -1):
        match = re.findall(func_pattern, disasm[i])
        if match:
            func_name = disasm[i][10:-3]
            if func_name[0] == '_':
                return parse_mangled_name(func_name)


def account_symbols(disasm, symbols):
    total_actual = 0
    total_estimated = 0
    found = set()
    func_to_address = defaultdict(set)
    address_to_func = defaultdict(set)

    # pattern to detect if there's an embedded data reference in a line
    line_pattern = r'# [0-9a-f]{1,16} <.*>'
    # pattern to extract address from a line
    address_pattern = r' [0-9a-f]{1,16} '

    for i in range(len(disasm)):
        match = re.findall(line_pattern, disasm[i])
        if match:
            address = int(re.findall(address_pattern, match[0])[0].strip(), 16)
            if address in symbols:
                func_name = trace_function(disasm, i)
                func_to_address[func_name].add(address)
                address_to_func[address].add(func_name)
                if address not in found:
                    total_actual += symbols[address].actual_size
                    total_estimated += symbols[address].estimated_size
                    found.add(address)

    return total_actual, total_estimated, func_to_address, address_to_func


def add_size_information(func_to_address, symbols):
    for func in func_to_address.keys():
        addresses = func_to_address[func]
        actual_size = 0
        estimated_size = 0
        for address in addresses:
            actual_size += symbols[address].actual_size
            estimated_size += symbols[address].estimated_size
        func_to_address[func] = FunctionInfo(embedded_data_size_actual=actual_size,
                                             embedded_data_size_estimated=estimated_size,
                                             embedded_data_count=len(
                                                 addresses),
                                             addresses=addresses)


def colored_string(msg, color):
    RED = '\033[31m'
    YELLOW = '\033[93m'
    END_COLOR = '\033[0m'
    if color == 'red':
        return RED + msg + END_COLOR
    return YELLOW + msg + END_COLOR


def critical_error(msg):
    print(colored_string(msg, 'red'))
    print('use option --help or -h for more information')
    sys.exit()


def check_architecture(elf_path):
    readelf = os.popen(f'readelf -h {elf_path} | grep Machine').read()
    if 'RISC-V' not in readelf:
        critical_error('Error: elf file must be compiled for RISC-V')


def main():
    args = get_args()

    if args['help']:
        print(HELP_STRING)
        sys.exit()
    if args['objdump'] is None:
        objdump = find_objdump()
    else:
        objdump = args['objdump']
    if args['elf_path'] is None:
        critical_error(
            'please provide an elf file with the --elf_path/-e option')
    if args['name'] is None:
        critical_error(
            'please provide a name to be displayed in the HTML page with the --name/-n option')
    if args['output_directory'] is None:
        critical_error(
            'please provide an output directory with the --output_directory/-o option')

    check_architecture(args['elf_path'])

    disasm = os.popen(f'{objdump} -d {args["elf_path"]}').readlines()
    symbols = os.popen(f'{objdump} -t {args["elf_path"]}').readlines()

    symbols_table = filter_symbol_table(disasm, symbols)
    symbols_dict = build_symbols_dict(symbols_table)
    total_actual, total_estimated, func_to_address, address_to_func = account_symbols(
        disasm, symbols_dict)
    add_size_information(func_to_address, symbols_dict)

    sro_data = map_adress_to_data(disasm)
    sro_start, sro_end = get_sro_range(disasm)

    create_html_file(args['name'], func_to_address, symbols_dict,
                     args['output_directory'], sro_data, sro_start)


if __name__ == '__main__':
    main()
