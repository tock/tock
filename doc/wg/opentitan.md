OpenTitan Tock Working Group (OT)
=================================

- Working Group Charter
- Adopted 2/14/2020

## Goals

The goals of the OpenTitan Tock Working Group (OT) are to:

- complete and test RISC-V architectural support for OpenTitan,
- design, implement and test peripheral drivers for the earlgrey platform,
- document APIs and major design decisions, and
- collaborate with the core group on Tock APIs that may need to change to better
  support RISC-V/OT.

## Members

- Laura Abbott, Oxide Computer
- Brad Campbell (Chair),  University of Virginia
- Jon Flatley, Google
- Alistair Francis, Western Digital
- Garret Kelly, Google
- Philip Levis, Stanford University/Google
- Patrick Mooney, Oxide Computer

## Code Purview

The OT working group will be in charge of (responsible for reviewing and
approving pull requests for) the following code directories:

- `chips/ibex`
- `chips/lowrisc`
- `boards/opentitan`

In addition, while it is not the focus of the group, because of the tight
relationship in early stages of development, the OT working group is also
responsible for:

- `arch/rv32i`

As interest and breadth of efforts into RISC-V change, it may no longer be
necessary for OT group to be responsible for the RISC-V architecture-level
support, or it may be moved to a different working group. In general, changes to
`arch/rv32i` should be in consultation with other RISC-V chip and board
maintainers.

Also, the set of directories under the purview of the OT group may change or
grow as code is re-organized or new boards emerge.
