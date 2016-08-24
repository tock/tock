#!/usr/bin/env python
# Generate chip specific code from CMSIS SVD definitions.
from __future__ import print_function
import sys

try:
    from cmsis_svd.parser import SVDParser
except ImportError:
    print("ERROR: Could not import cmsis_svd. You can install it using:\n\n" +
            "\tpip install -U cmsis-svd\n", file=sys.stderr)
    # Print original traceback, just in case it provides more information
    import traceback
    traceback.print_exc(file=sys.stderr)

    sys.exit(1)

# A subset of keywords that may appear as register names
RUST_KEYWORDS = ["in"]

def dump_json(parser):
    """Dump the SVD model as JSON."""
    import json
    svd_dict = parser.get_device().to_dict()
    print(json.dumps(svd_dict, sort_keys=True, indent=4,
        separators=(',', ': ')))

def get_peripheral_interrupts(parser):
    # Cortex M0 supports up to 32 external interrupts
    # Source: See ARMv6-M Architecture Reference Manual,
    # Table C-2 "Programmers' model feature comparison"
    interrupts = [""] * 32

    for peripheral in parser.get_device().peripherals:
        for intr in peripheral.interrupts:
            if interrupts[intr.value]:
                assert interrupts[intr.value] == intr.name
            else:
                interrupts[intr.value] = intr.name

    return interrupts

def compat_get_derived_from(parser, peripheral):
    try:
        # Will work when a future cmsis-svd version exposes it.
        return peripheral.derived_from
    except AttributeError:
        # FIXME: workaround for getting into "derivedFrom" for the
        # peripheral (which is missing from the model). Issue being tracked
        # on https://github.com/posborne/cmsis-svd/pull/18
        return parser._root.findall('.//peripheral/[name=\'%s\']' %
                peripheral.name)[0].get('derivedFrom')

def get_peripheral_registers(parser, peripheral_names=[]):
    peripherals = []
    for peripheral in parser.get_device().peripherals:
        if peripheral_names and peripheral.name not in peripheral_names:
            continue
        registers = peripheral.registers
        derived_from = compat_get_derived_from(parser, peripheral)
        if not registers and derived_from:
            registers = filter(lambda p: p.name == derived_from,
                    parser.get_device().peripherals)[0].registers
        # Makes no sense to continue in this case
        assert registers, "SVD does not contain any registers for peripheral " + \
                "'%s'" % peripheral.name
        cur_ofs = 0
        reserved_id = 1
        reg_model = []
        for register in registers:
            offset = register.address_offset
            if offset != cur_ofs:
                assert offset > cur_ofs
                # This should always be true due to ARM alignment requirements
                assert (offset - cur_ofs) % 4 == 0
                reserved_size = (offset - cur_ofs) / 4
                reg_model.append({
                    "name": "_reserved%d" % reserved_id,
                    "array_size": reserved_size,
                    "reserved": True,
                })
                reserved_id += 1
            if register.dim:
                assert register.dim_increment == 4
                assert register.dim_index == range(0, register.dim)
                array_size = register.dim
            else:
                array_size = 1
            rname = register.name.replace("[%s]", "").lower()
            if rname in RUST_KEYWORDS:
                rname += "_"
            reg_model.append({
                "name": rname,
                "array_size": array_size,
                "reserved": False,
            })
            cur_ofs = offset + 4 * array_size

        peripherals.append({
            "name": peripheral.name,
            "base_address": peripheral.base_address,
            "registers": reg_model,
        })

    return peripherals

def gen_file(env, outfile, variables):
    # To generate a template for file.c, create a file.c.jinja
    template = env.get_template(outfile + '.jinja')
    template.stream(**variables).dump(outfile)

def main():
    from jinja2 import Environment, FileSystemLoader
    from os.path import relpath

    parser = SVDParser.for_packaged_svd('Nordic', 'nrf51.svd')
    #dump_json(parser)
    interrupts = get_peripheral_interrupts(parser)
    dump_macros(interrupts,
            open("src/chips/nrf51822/src/peripheral_interrupts.h", "w"))
    peripherals = get_peripheral_registers(parser, ["GPIO"])

    env = Environment(loader=FileSystemLoader('src/chips/nrf51822/src'))
    template = env.get_template('peripheral_registers.rs.jinja')
    template.stream(program=PROGRAM, peripherals=peripherals).dump(
            'src/chips/nrf51822/src/peripheral_registers.rs')

if __name__ == "__main__":
    main()
