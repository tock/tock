#!/usr/bin/env python
#
# usage: svd2regs.py [-h] [--group] (--mcu VENDOR MCU | --svd [SVD])
#                    [--save FILE] [--fmt ['ARG ..']] [--path PATH]
#                    peripheral
#
# positional arguments:
#   peripheral        Name of the Peripheral
#
# optional arguments:
#   -h, --help        show this help message and exit
#   --group, -g       Peripheral is a group with several instances
#   --mcu VENDOR MCU  Vendor and MCU (Database from cmsis-svd)
#   --svd [SVD]         Path to SVD-File
#   --save FILE       Save generated Code to file
#
# rustfmt:
# Format with rustfmt
#
#   --fmt ['ARG ..']  enable rustfmt with optional arguments
#   --path PATH       path to rustfmt
#
# Examples:
#   SIM peripheral from Database
#     svd2regs.py SIM --mcu Freescale MK64F12
#
#   Format with rustfmt
#     svd2regs.py SIM --svd mcu.svd --fmt
#
#   Format with rustfmt --force
#     svd2regs.py SIM --svd mcu.svd --fmt '--force'
#
#   Format with rustfmt not in PATH
#     svd2regs.py SIM --svd mcu.svd --fmt --path /home/tock/bin/
#
#   Save to file
#     svd2regs.py SIM --svd mcu.svd --fmt '--force' --save src/peripherals.rs
#
#   With stdin pipe
#     cat mcu.svd | svd2regs.py SIM --svd --fmt '--force' | tee src/mcu.rs
#
# Required Python Packages:
#   cmsis-svd
#   pydentifier
#
# Author: Stefan Hoelzl <stefan.hoelzl@posteo.de>

import sys
import argparse
from subprocess import Popen, PIPE
from xml.etree import ElementTree as ET

try:
    from cmsis_svd.parser import SVDParser
except ImportError:
    print('Could not import CMSIS SVD library')
    print('pip install cmsis-svd')
    sys.exit(1)

try:
    import pydentifier
except ImportError:
    print('Could not import Pydentifier library')
    print('pip install pydentifier')
    sys.exit(1)

RUST_KEYWORDS = ["mod"]
COMMENT_MAX_LENGTH = 80


def comment(text):
    if text:
        return "/// {}".format(text[:COMMENT_MAX_LENGTH].strip())
    else:
        return ''


class CodeBlock(str):
    TEMPLATE = ""

    def __new__(cls, *args):
        return cls.TEMPLATE.format(**cls.fields(*args))

    @staticmethod
    def fields(*args):
        return {}


class Includes(CodeBlock):
    TEMPLATE = """
use kernel::common::StaticRef;
use kernel::common::registers::{{self, ReadOnly, ReadWrite, WriteOnly}};
    """


class PeripheralBaseDeclaration(CodeBlock):
    TEMPLATE = """
const {name}_BASE: StaticRef<{title}Registers> =
    unsafe {{ StaticRef::new(0x{base:8X} as *const {title}Registers) }};
"""

    @staticmethod
    def fields(base, peripheral):
        return {
            "name": peripheral.name,
            "title": base.title(),
            "base": peripheral.base_address,
        }


class PeripheralStruct(CodeBlock):
    TEMPLATE = """{comment}
#[repr(C)]
struct {name}Registers {{
{fields}
}}
"""

    @staticmethod
    def fields(name, peripheral, dev):
        def get_register_size(reg):
            size = reg._size
            if size is None and reg.parent:
                size = reg.parent.size
            if size is None and dev.size:
                size = dev.size
            if size is None:
                raise Exception(
                    "Cant figure out size of register {}".format(reg.name)
                )
            if size not in [8, 16, 32]:
                raise Exception(
                    "Invalid size {} of register {}".format(size, reg.name)
                )
            return size

        fields = []
        offset = 0
        cnt = 0
        for register in sorted(peripheral.registers,
                               key=lambda r: r.address_offset):
            if register.address_offset > offset:
                diff = (register.address_offset - offset)
                fields.append(ReservedStructField(cnt, diff))
                cnt += 1
                offset += diff
            if offset == register.address_offset:
                size = get_register_size(register)
                fields.append(PeripheralStructField(register, size))
                offset += size / 8
            else:
                # TODO: handle overlapping registers better (Unions?)
                print(
                    "Offset Mismatch at register {} ({} != {})".format(
                        register.name,
                        register.address_offset,
                        offset
                    )
                )

        return {
            "comment": comment(peripheral.description),
            "name": name.title(),
            "fields": "\n".join(fields)
        }


class PeripheralStructField(CodeBlock):
    TEMPLATE = """{comment}
{name}: {mode}<u{size}{definition}>,"""

    @staticmethod
    def fields(register, size):
        def identifier(name):
            identifier = pydentifier.lower_underscore(name)
            if identifier in RUST_KEYWORDS:
                identifier = "{}_".format(identifier)
            return identifier

        def definition(reg):
            if len(reg._fields) == 1:
                return ""
            return ", {}::Register".format(reg.name)

        mode_map = {
            "read-only": "ReadOnly",
            "read-write": "ReadWrite",
            "write-only": "WriteOnly",
        }

        return {
            "comment": comment(register.description),
            "name": identifier(register.name),
            "size": size,
            "mode": mode_map.get(register._access, "ReadWrite"),
            "definition": definition(register),
        }


class ReservedStructField(CodeBlock):
    TEMPLATE = """_reserved{cnt}: [u8; {size}],"""

    @staticmethod
    def fields(cnt, size):
        return {
            "cnt": cnt,
            "size": int(size),
        }


class BitfieldsMacro(CodeBlock):
    TEMPLATE = """register_bitfields![u{size},{bitfields}
];"""

    @staticmethod
    def fields(registers):
        bitfields = ",".join(Bitfield(register) for register in registers)
        return {
            "size": 32,
            "bitfields": bitfields
        }


class Bitfield(CodeBlock):
    TEMPLATE = """
{name} [
{fields}
]"""

    @staticmethod
    def fields(register):
        fields = ",\n".join(BitfieldField(field) for field in register._fields)
        return {
            "name": register.name,
            "fields": fields,
        }


class BitfieldField(CodeBlock):
    TEMPLATE = """    {comment}
    {name} OFFSET({offset}) NUMBITS({size}) {enums}"""

    @staticmethod
    def enumerated_values(field):
        values = []
        for value in field.enumerated_values:
            if value.description not in [v.description for v in values]:
                values.append(value)
        return values

    @staticmethod
    def fields(field):
        if not field.is_enumerated_type:
            enums = "[]"
        else:
            enums = ",\n".join(BitfieldFieldEnum(enum)
                               for enum in BitfieldField.enumerated_values(field))
            enums = "[\n{}\n    ]".format(enums)
        return {
            "comment": comment(field.description),
            "name": field.name,
            "offset": field.bit_offset,
            "size": field.bit_width,
            "enums": enums,
        }


class BitfieldFieldEnum(CodeBlock):
    TEMPLATE = """        {comment}
        {name} = {value}"""

    @staticmethod
    def fields(enum):
        def identifier(desc):
            if not desc:
                return None
            if any(desc.startswith(str(digit)) for digit in range(10)):
                desc = "_{}".format(desc)
            i = pydentifier.upper_camel(desc)
            return i if len(i) < 80 else None

        def enum_identifier(e):
            for t in [e.description, e.name, e.value]:
                i = identifier(t)
                if i:
                    return i

        return {
            "comment": comment(enum.description),
            "name": enum_identifier(enum),
            "value": enum.value,
        }


def get_parser(mcu, svd):
    try:
        if mcu:
            return SVDParser.for_packaged_svd(mcu[0], "{}.svd".format(mcu[1]))
        return SVDParser(ET.ElementTree(ET.fromstring(svd.read())))
    except IOError:
        print("No SVD file found")
        sys.exit()


def parse(peripheral_name, mcu, svd, group):
    svd_parser = get_parser(mcu, svd)
    dev = svd_parser.get_device()
    if group:
        flt = lambda p: p.group_name == peripheral_name
    else:
        flt = lambda p: p.name == peripheral_name
    return peripheral_name, filter(flt, dev.peripherals), dev


def generate(name, peripherals, dev):
    peripherals = list(peripherals)

    if len(peripherals) == 0:
        print('Error: no peripheral found.')
        return ''

    main_peripheral = peripherals[0]
    return Includes() \
           + PeripheralStruct(name, main_peripheral, dev) \
           + generate_bitfields_macro(filter(lambda r: len(r._fields) > 1,
                                             main_peripheral.registers)) \
           + "\n".join(PeripheralBaseDeclaration(name, peripheral)
                       for peripheral in peripherals)


def generate_bitfields_macro(registers):
    if registers:
        return BitfieldsMacro(registers)
    return ""


def rustfmt(code, path, *args):
    cmd = ["{}rustfmt".format(path)]
    cmd.extend(args)
    fmt = Popen(cmd, stdin=PIPE, stdout=PIPE, stderr=PIPE)
    out, err = fmt.communicate(code)
    if err:
        print(code)
        print(err)
        sys.exit()

    return out


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("peripheral", help="Name of the Peripheral")
    parser.add_argument("--group", "-g", action="store_true",
                        help="Peripheral is a group with several instances")
    xor = parser.add_mutually_exclusive_group(required=True)
    xor.add_argument('--mcu', nargs=2, metavar=('VENDOR', 'MCU'),
                     help='Vendor and MCU (Database from cmsis-svd)')
    xor.add_argument('--svd', type=argparse.FileType('r'), const=sys.stdin,
                     nargs="?", metavar="SVD", help='Path to SVD-File')
    parser.add_argument("--save", type=argparse.FileType('w'), metavar="FILE",
                        default=sys.stdout, help="Save generated Code to file")
    fmt = parser.add_argument_group('rustfmt',
                                    'Format with rustfmt')
    fmt.add_argument("--fmt", nargs="?", const='', metavar="'ARG ..'",
                     help="enable rustfmt with optional arguments")
    fmt.add_argument("--path", help="path to rustfmt", default="")
    return parser.parse_args()


def main():
    args = parse_args()
    code = generate(*parse(args.peripheral, args.mcu, args.svd, args.group))
    if args.fmt is not None:
        code = rustfmt(code, args.path, *args.fmt.strip("'").split(" "))
    args.save.write(code)


if __name__ == '__main__':
    main()
