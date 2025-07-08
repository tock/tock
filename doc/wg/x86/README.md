Tock x86 Working Group (X86)
======================================

- Working Group Charter
- Adopted 06/19/2025

## Goals

The goals of the Tock x86 Working Group (x86) are to:

- maintain and improve x86 support in the Tock kernel, libtock-rs, and
  libtock-c.
- review changes to the Tock kernel, libtock-rs, and libtock-c that
  affect x86 support.

## Members

- Amit Levy (Chair), Tock Foundation
- Hussain Miyaziwala, Microsoft/Pluton
- Alexandru Radovici, OxidOS

## Code Purview

The x86 working group is in responsible for reviewing, approving, and
merging pull requests for the following crates in the kernel:

- `arch/x86`
- `chips/x86_q35`
- `boards/qemu_i486_q35`

It is also responsible for reviewing, approving, and merging pull
requests in `libtock-c` and `libtock-rs` that are specific to x86
architectures, including startup assembly and system call adaptation
layers.
