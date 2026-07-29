# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

import binascii
import os
from collections import defaultdict

HTML_HEADER = '''
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Tock Embedded Data</title>
    <style>
        /* table styling */
        .styled-table {
            border-collapse: collapse;
            margin: 0 auto;
            width: 80%;
            padding: 10px;
            font-size: 0.9em;
            font-family: sans-serif;
            min-width: 400px;
            box-shadow: 0 0 20px rgba(0, 0, 0, 0.15);
            text-align: left;
        }
        .styled-table thead tr {
            background-color: #009879;
            color: #ffffff;
            text-align: left;
        }
        .styled-table th,
        .styled-table td {
            padding: 12px 15px;
        }

        .styled-table tbody tr:nth-of-type(even) {
            background-color: #f3f3f3;
        }

        .styled-table tbody tr.active-row {
            font-weight: bold;
            color: #009879;
        }

        a {
            text-decoration: none;
            color: black;
        }

        button {
            background-color: #009879;
            border: none;
            border-radius: 25px;
            color: white;
            padding: 0.3em 0.8em;
            text-align: center;
            text-decoration: none;
            font-size: smaller;
        }
    </style>
</head>
<body>
'''

HTML_FOOTER = '''
        </tbody>
    </table>
</body>

</html>
'''

ENTRY_TABLE_HEADER = '''
    <table class="styled-table">
        <thead>
        <tr>
            <th>De-mangled Function Name</th>
            <th style="text-align: right;">Embedded Data Size (Estimated)</th>
            <th style="text-align: right;">Embedded Data Size (Reported)</th>
            <th style="text-align: right;">Embedded Data Count</th>
        </tr>
        </thead>
        <tbody>
'''

FUNC_TABLE_HEADER = '''
    <table class="styled-table">
        <thead>
        <tr>
            <th>Symbol Name</th>
            <th style="text-align: right;">Size (Estimated)</th>
            <th style="text-align: right;">Size (Reported)</th>
            <th>Data</th>
        </tr>
        </thead>
        <tbody>
'''


def create_function_page(func_name, addresses, symbols_dict, sro_data, sro_start):
    symbol_infos = []
    for address in addresses:
        estimated_size = symbols_dict[address].estimated_size
        name = symbols_dict[address].name
        actual_size = symbols_dict[address].actual_size
        start_index = address - sro_start
        data = sro_data[start_index:start_index+estimated_size]
        symbol_infos.append((
            name,
            estimated_size,
            actual_size,
            data
        ))

    symbol_infos.sort(key=lambda x: x[1], reverse=True)

    rows = []
    for i in range(len(symbol_infos)):
        info = symbol_infos[i]

        # info[3] holds the raw section bytes. Hexlify them directly rather
        # than re-encoding a decoded string, which would turn each U+FFFD
        # replacement character into its own three bytes (ef bf bd) and show
        # data that is not what the binary actually contains.
        raw_data = info[3]
        hex_data = binascii.hexlify(raw_data).decode('utf-8')
        formatted_hex_data = ''
        for j in range(0, len(hex_data), 4):
            formatted_hex_data += f'{hex_data[j:j+4]} '

        # .srodata mixes string literals with binary constants, so decoding it
        # as text is only ever partly meaningful. Render it the way a hex dump
        # does: printable ASCII as itself, everything else as '.'. Decoding
        # with errors='replace' instead would litter the pane with U+FFFD for
        # every byte >= 0x80 and emit raw control characters for the low ones,
        # which is the garbled output this pane was showing.
        ascii_data = ''.join(
            chr(b) if 0x20 <= b < 0x7f else '.' for b in raw_data)

        hex_font = ' style=\"font-family: monospace, monospace;\"'

        # This goes into a JS template literal, so backslash, backtick and the
        # ${...} interpolation marker all have to be escaped -- .srodata holds
        # arbitrary strings from the firmware, and an unescaped ${...} would be
        # evaluated as an expression when the pane is toggled.
        escaped_ascii_data = (ascii_data
                              .replace('\\', '\\\\')
                              .replace('`', '\\`')
                              .replace('${', '\\${'))

        data_html = f'''
                <th>
                    <script>
                        function toggle{i}() {{
                            var x = document.getElementById("{i}");
                            if (x.dataset.mode === "hex") {{
                                x.dataset.mode = "ascii";
                                x.innerHTML = `{escaped_ascii_data}`;
                            }} else {{
                                x.dataset.mode = "hex";
                                x.innerHTML = "{formatted_hex_data}";
                            }}
                        }}
                    </script>
                    <button onclick="toggle{i}()">ASCII/HEX</button>
                    <div{hex_font} data-mode="ascii" id="{i}">{ascii_data}</div>
                </th>
        '''

        entry_string = f'''
            <tr>
                <th>{info[0]}</th>
                <th style="text-align: right;">{info[1]}</th>
                <th style="text-align: right;">{info[2]}</th>
                {data_html}
            </tr>
        '''
        rows.append(entry_string)

    header = HTML_HEADER.replace(
        '<title>Tock Embedded Data</title>', f'<title>{func_name}</title>')

    name_header = f'''
    <h1 style="text-align: center;">{func_name}</h1>
    '''

    return header + FUNC_TABLE_HEADER + name_header + ''.join(rows) + HTML_FOOTER


def get_table_entry_string(function_name, info, index):
    html_string = f'''
        <tr>
            <th><a href="funcs/{index}.html">{function_name}</a></th>
            <th style="text-align: right;">{info.embedded_data_size_estimated}</th>
            <th style="text-align: right;">{info.embedded_data_size_actual}</th>
            <th style="text-align: right;">{info.embedded_data_count}</th>
        </tr>
    '''
    return html_string


def sort_functions(func_to_address_list):
    grouped_by_crate_dict = defaultdict(list)
    for func_to_address in func_to_address_list:
        func_name = func_to_address[0]
        found = func_name.find(':')
        crate = '' if found == -1 else func_name[:found]
        grouped_by_crate_dict[crate].append(func_to_address)

    grouped_by_crate_list = []
    for crate in grouped_by_crate_dict.keys():
        func_list = grouped_by_crate_dict[crate]
        crate_size = sum(x[1].embedded_data_size_estimated for x in func_list)
        func_list.sort(
            key=lambda x: x[1].embedded_data_size_estimated, reverse=True)
        grouped_by_crate_list.append((crate_size, func_list))

    grouped_by_crate_list.sort(key=lambda x: x[0], reverse=True)

    return [func for crate in grouped_by_crate_list for func in crate[1]]


def create_html_strings(name, func_to_address, symbols_dict, sro_data, sro_start):
    func_to_address_list = list(func_to_address.items())
    func_to_address_list = sort_functions(func_to_address_list)

    table_rows = []
    function_pages = []
    for i in range(len(func_to_address_list)):
        function_name, info = func_to_address_list[i]
        html = get_table_entry_string(function_name, info, i)
        table_rows.append(html)
        function_pages.append(create_function_page(
            function_name, info.addresses, symbols_dict, sro_data, sro_start))

    header = HTML_HEADER.replace(
        '<title>Tock Embedded Data</title>', f'<title>{name}</title>')

    name_header = f'''
    <h1 style="text-align: center;">{name}</h1>
    '''

    index_page = header + name_header + ENTRY_TABLE_HEADER + \
        ''.join(table_rows) + HTML_FOOTER

    return index_page, function_pages


def write_file(path, str):
    with open(path, 'w') as f:
        f.truncate(0)
        f.write(str)


def create_html_file(name, func_to_address, symbols_dict, file_path, sro_data, sro_start):
    funcs_path = os.path.join(file_path, 'funcs')
    os.makedirs(funcs_path, exist_ok=True)

    index, funcs = create_html_strings(
        name, func_to_address, symbols_dict, sro_data, sro_start)
    write_file(os.path.join(file_path, f'{name}.html'), index)
    for i in range(len(funcs)):
        write_file(os.path.join(funcs_path, f'{i}.html'), funcs[i])

    print(
        f'html file containing embedded data information written to {file_path}{os.sep}{name}.html')
