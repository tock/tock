#!/usr/bin/env python3

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

"""Splice a TBF app image into an elf2hex memory image.

The SHAKTI C-Class test SoC loads a single flat image (``code.mem``) into its
main memory at the RAM base. The kernel ELF supplies that image, but the app
region (``_sapps``) sits inside the same memory, so the app's TBF has to be
written into the image rather than loaded separately.

``elf2hex`` emits a fixed-size image of one row per bus word, most significant
byte first: bytes ``b0..b7`` at an 8-byte-aligned address render as the hex
string ``b7 b6 b5 b4 b3 b2 b1 b0``, i.e. the lowest address is the
least-significant lane. This rewrites exactly the rows covering
``[addr, addr + len(tbf))`` and leaves the rest of the image untouched.

Usage:
    splice_tbf.py <image> <app.tbf> <base-addr> <splice-addr> [--width N]

    splice_tbf.py code.mem app.tbf 0x80000000 0x80100000
"""

import argparse
import sys


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("image", help="elf2hex output to modify in place")
    p.add_argument("tbf", help="TBF app image to splice in")
    p.add_argument("base", help="physical address of the first row (e.g. 0x80000000)")
    p.add_argument("addr", help="physical address to splice at (e.g. 0x80100000)")
    p.add_argument(
        "--width",
        type=int,
        default=8,
        help="bytes per row, matching elf2hex's width argument (default: 8)",
    )
    args = p.parse_args()

    width = args.width
    base = int(args.base, 0)
    addr = int(args.addr, 0)

    if addr < base:
        sys.exit(f"splice address {addr:#x} is below the image base {base:#x}")
    if (addr - base) % width != 0:
        sys.exit(f"splice address {addr:#x} is not {width}-byte aligned relative to {base:#x}")

    with open(args.tbf, "rb") as f:
        tbf = f.read()
    if not tbf:
        sys.exit(f"{args.tbf} is empty")

    # Pad up to a whole row so the final partial word is well defined.
    if len(tbf) % width:
        tbf += b"\x00" * (width - len(tbf) % width)

    with open(args.image, "r") as f:
        rows = f.read().splitlines()

    start = (addr - base) // width
    count = len(tbf) // width
    if start + count > len(rows):
        sys.exit(
            f"splice would run past the end of the image: needs {start + count} "
            f"rows, image has {len(rows)}. Increase elf2hex's row count."
        )

    for i in range(count):
        chunk = tbf[i * width : (i + 1) * width]
        # Ascending address -> descending significance.
        rows[start + i] = "".join(f"{b:02x}" for b in reversed(chunk))

    with open(args.image, "w") as f:
        f.write("\n".join(rows) + "\n")

    print(
        f"spliced {len(tbf)} bytes ({count} rows) into {args.image} at "
        f"{addr:#x} (rows {start}..{start + count - 1})"
    )


if __name__ == "__main__":
    main()
