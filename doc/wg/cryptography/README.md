Tock Cryptography Working Group (crypto)
========================================

- Working Group Charter
- Adopted 09/30/2025

## Goals

The goals of the Tock Working Group (crypto) are to:

- Improve and standardize HIL and system call interfaces for cryptographic
  primitives.
- Maintain and improve cryptography support in the Tock kernel, libtock-rs, and
  libtock-c.
- Review changes to the Tock kernel, libtock-rs, and libtock-c that
  affect cryptography support.

## Members

- Tyler Potyondy (Lead), UCSD
- Hussain Miyaziwala, Microsoft/Pluton
- Kat Fox, zeroRISC
- Amit Levy (Core WG), Tock Foundation

## Membership and Communication

The cryptography working group membership is open to Tock developers
interested in the design of cryptography interfaces. Group membership
is decided by the group: the exact process is not yet determined and
may organically evolve as the group gains momentum.

The group primarily coordinates via the Cryptography WG channel in the
[Tock Matrix space](https://matrix.to/#/#tock:tockos.org). Those
looking to engage with the working group are encouraged to join the
channel.

## Code Purview

The crypto working group is responsible for reviewing, approving, and
merging pull requests for the following HILs:

- `kernel::hil::symmetric_encryption`
- `kernel::hil::public_key_crypto`
- `kernel::hil::digest`
- `kernel::hil::crc`
- `kernel::hil::rng`

capsules:

- `capsules_extra::symmetric_encryption`
- `capsules_extra::public_key_crypto`
- `capsules_extra::hmac`
- `capsules_extra::hmac_sha256`
- `capsules_extra::sha`
- `capsules_extra::sha256`
- `capsules_aes_gcm`
- `ecdsa_sw`

It will also be responsible for any additional HILs and capsules that
are specific to cryptography functionality, reviewing and maintaining
external dependencies on crypographic crates, as well as for
reviewing, approving, and merging pull requests in `libtock-c` and
`libtock-rs` that are specific to cryptography architectures,
including libraries for interacting with cryptography system call
drivers.

The working group's scope does not apply to subsystems that merely
_use_ cryptography interfaces.
