#!/usr/bin/env python3

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

from embedded_data_visualizer import create_html_file
from print_tock_memory_usage import parse_mangled_name
import argparse
from collections import defaultdict, namedtuple
import os
import sys
import re
import cxxfilt  # Added for C++ demangling

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
    sys.exit(1)

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
    for candidate in [
        'riscv32-unknown-elf-objdump',
        'riscv64-unknown-elf-objdump',
        'riscv32-none-elf-objdump',
        'riscv64-none-elf-objdump',
    ]:
        objdump = os.popen(f'which {candidate}').read().strip()
        if objdump != '':
            return objdump
    critical_error(
        'Objdump not found, please specify a riscv objdump path manually with -d or --objdump.')

FunctionInfo = namedtuple(
    'FunctionInfo', 'embedded_data_size_actual embedded_data_size_estimated embedded_data_count addresses')
SymbolInfo = namedtuple('SymbolInfo', 'actual_size estimated_size name')

def get_srodata_address(disasm):
    pattern = '<_srodata>:'
    for i, line in enumerate(disasm):
        if pattern in line:
            return line.split()[0], i
    critical_error('Invalid binary, cannot find <_srodata> section')

def map_address_to_data(disasm):
    _, start_index = get_srodata_address(disasm)
    sro_address, sro_end = get_sro_range(disasm)
    address_pattern = r'^\s*([0-9a-f]+):'
    data_pattern = r'[ \t][0-9a-f]{4}'

    sro_data = ""
    for i in range(start_index, len(disasm)):
        match = re.search(address_pattern, disasm[i])
        if match:
            address = int(match.group(1), 16)
            if sro_address <= address < sro_end:
                data = re.findall(data_pattern, disasm[i])
                if data:
                    # remove the \t or spaces from first byte
                    data[0] = re.sub(r'\s+', '', data[0])
                    for d in data:
                        d = d.strip()
                        # separate bytes in swapped order
                        sro_data += bytes.fromhex(d[2:4]).decode("utf-8", errors='replace')
                        sro_data += bytes.fromhex(d[0:2]).decode("utf-8", errors='replace')

    return sro_data

def get_sro_range(disasm):
    data_pattern = r'[ \t][0-9a-f]{4}'
    address_pattern = r'^[0-9a-f]{1,16}'
    sro_string, start_index = get_srodata_address(disasm)
    sro_address = int(sro_string, 16)
    previous_blank = False
    last_address_seen = 0
    last_line = -1
    for i in range(start_index, len(disasm)):
        if disasm[i] == '\n':
            previous_blank = True
        else:
            match = re.findall(address_pattern, disasm[i])
            if match:
                last_address_seen = int(match[0], 16)
                last_line = i
                previous_blank = False
            elif previous_blank:
                # if the last line was blank and current line has no address data, the section has ended
                break

    if last_line != -1:
        data = re.findall(data_pattern, disasm[last_line])
        last_address_seen += len(data)*2

    return sro_address, last_address_seen

def filter_symbol_table(disasm, symbols_table):
    sro_start, sro_end = get_sro_range(disasm)

    # trim garbage lines at the beginning and end
    trimmed_lines = symbols_table[4:-4]
    filtered = []
    for line in trimmed_lines:
        try:
            address = int(line[:8], 16)
        except ValueError:
            continue
        if sro_start <= address < sro_end:
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
        if estimated_size == 0 and i != len(items_list) - 1:
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
            r'^([0-9a-f]+)\s+[lg]\s+O?\s+\.text\s+([0-9a-f]+)\s+(.*)', line)
        if not matches:
            continue
        address = int(matches.group(1), 16)
        size = int(matches.group(2), 16)
        name = matches.group(3).strip()

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
    # pattern to detect function header line like:
    # 10008 <function_name>:
    func_pattern = r'^[0-9a-f]{1,16} <.*>:$'
    for i in range(line_num, -1, -1):
        if re.match(func_pattern, disasm[i].strip()):
            # Extract function name between '<' and '>:'
            start = disasm[i].find('<') + 1
            end = disasm[i].find('>:')
            func_name = disasm[i][start:end]
            # Demangle if starts with '_'
            if func_name.startswith('_'):
                try:
                    return cxxfilt.demangle(func_name)
                except Exception:
                    # fallback to parse_mangled_name for other formats
                    return parse_mangled_name(func_name)
            return func_name
    return None

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
            addr_str = re.findall(address_pattern, match[0])[0].strip()
            address = int(addr_str, 16)
            if address in symbols:
                func_name = trace_function(disasm, i)
                if func_name is None:
                    func_name = "<unknown>"
                func_to_address[func_name].add(address)
                address_to_func[address].add(func_name)
                if address not in found:
                    total_actual += symbols[address].actual_size
                    total_estimated += symbols[address].estimated_size
                    found.add(address)

    return total_actual, total_estimated, func_to_address, address_to_func

def add_size_information(func_to_address, symbols):
    for func in list(func_to_address.keys()):
        addresses = func_to_address[func]
        actual_size = 0
        estimated_size = 0
        for address in addresses:
            actual_size += symbols[address].actual_size
            estimated_size += symbols[address].estimated_size
        func_to_address[func] = FunctionInfo(
            embedded_data_size_actual=actual_size,
            embedded_data_size_estimated=estimated_size,
            embedded_data_count=len(addresses),
            addresses=addresses
        )

def check_architecture(elf_path):
    readelf_output = os.popen(f'readelf -h {elf_path} | grep Machine').read()
    if 'RISC-V' not in readelf_output:
        critical_error('Error: elf file must be compiled for RISC-V')

def main():
    args = get_args()

    if args['help']:
        print(HELP_STRING)
        sys.exit(0)

    objdump = args['objdump'] if args['objdump'] else find_objdump()

    if not args['elf_path']:
        critical_error('Please provide an elf file with the --elf_path/-e option')
    if not args['name']:
        critical_error('Please provide a name to be displayed in the HTML page with the --name/-n option')
    if not args['output_directory']:
        critical_error('Please provide an output directory with the --output_directory/-o option')

    check_architecture(args['elf_path'])

    disasm = os.popen(f'{objdump} -d {args["elf_path"]}').readlines()
    symbols = os.popen(f'{objdump} -t {args["elf_path"]}').readlines()

    symbols_table = filter_symbol_table(disasm, symbols)
    symbols_dict = build_symbols_dict(symbols_table)
    total_actual, total_estimated, func_to_address, address_to_func = account_symbols(disasm, symbols_dict)
    add_size_information(func_to_address, symbols_dict)

    sro_data = map_address_to_data(disasm)
    sro_start, sro_end = get_sro_range(disasm)

    create_html_file(args['name'], func_to_address, symbols_dict,
                     args['output_directory'], sro_data, sro_start)

if __name__ == '__main__':
    main()

        
