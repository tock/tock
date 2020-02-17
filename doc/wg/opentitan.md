OpenTitan Tock Working Group (OT)
=================================

- Working Group Charter
- Adopted 2/17/2020

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

## Membership and Communication

The OpenTitan working group membership is a subset of the people who have
commit (pull request merge) permissions on the Tock repository. It is
intended to be a smaller group that represents the major perspectives
and issues, rather than a complete group. Group membership is decided by
the group: the exact process is not yet determined and may organically
evolve as the group gains momentum.

The group has a weekly teleconference call. All working group members  
participate in the call. Other people may be invited to participate to
help contribute to particular topics or on-going discussions. The
working group chair decides who beyond the working group members may
participate in the call.

The working group publishes detailed notes of its calls. These will be
posted within a week of a call. This delay is to give participants an
opportunity to correct any errors or better explain points that came up.
They are intended to be a communication mechanism of the group, its 
discussions, the technical issues, and decisions, not a literal
transcription of what is said.

## Code Purview

The OT working group will be in charge of (responsible for reviewing,
approving, and merging pull requests for) the following code directories:

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

All members of the OpenTitan working group have the ability to merge pull
requests to code in these directories, following the working group's code
review process. The working group may also grant pull request merging
abilities to people outside the group. In cases when a pull request modifies
directories under the purview of more than one  working group, the approval of 
all responsible working groups is required. 

