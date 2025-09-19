Tock Cryptography Working Group (crypto)
======================================

- Working Group Charter
- Adopted 09/18/2025

## Goals

The goals of the Tock Working Group (crypto) are to:

- Improve and standardize HIL and system call interfaces for cryptographic
  primitives
- Maintain and improve cryptography support in the Tock kernel, libtock-rs, and
  libtock-c.
- Review changes to the Tock kernel, libtock-rs, and libtock-c that
  affect cryptography support.

## Members

- Amit Levy (Chair), Tock Foundation
- Hussain Miyaziwala, Microsoft/Pluton
- Kat Fox, zeroRISC
- Tyler Potyondy, UCSD

## Code Purview

The crypto working group is in responsible for reviewing, approving, and
merging pull requests for the following HILs:

- `kernel::hil::symmetric_encryption`
- `kernel::hil::public_key_crypto`
- `kernel::hil::digest`
- `kernel::hil::crc`
- `kernel::hil::rng`

It will also be responsible for any additional HILs that are specific to
cryptography functionality, as well as for reviewing, approving, and merging
pull requests in `libtock-c` and `libtock-rs` that are specific to cryptography
architectures, including libraries for interacting with cryptography system
call drivers.

It does not have exclusive purview over other kernel or userspace subsystems
that use cryptography but whose primary purpose is other functionality (such as
networking stacks that encrypt traffic).
