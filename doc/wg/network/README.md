Network Working Group (NWG)
=================================

- Working Group Charter
- Adopted 8/11/2023

## Goals

The goals of the Network Tock Working Group (NWG) are to:

- design a set of interfaces (traits) for networking
- design the user land interfaces for networking
- define how buffer management should be implemented
- determine other kernel infrastructure useful for networking

## Members

- Alexandru Radovici (Chair), Politehnica University of Bucharest
- Branden Ghena, Northwestern University
- Leon Schuermann, Princeton University
- Tyler Potyondy, UCSD
- Cristian Rusu, University of Bucharest
- Felix Mada, OxidOS Automotive

## Membership and Communication

The networking working group membership is open to Tock developers interested
in the design of network interfaces. It is
intended to be a smaller group that represents the major perspectives
and issues, rather than a complete group. Group membership is decided by
the group: the exact process is not yet determined and may organically
evolve as the group gains momentum.

The group has a teleconference call once every two weeks. All working group members
participate in the call. Other people may be invited to participate to
help contribute to particular topics or on-going discussions. The
working group chair decides who beyond the working group members may
participate in the call.
Those looking to engage with the working group are encouraged to join the `#network-working-group` channel on the [Tock slack](https://github.com/tock/tock/#keep-up-to-date).
The working group publishes detailed notes of its calls. These will be
posted within a week of a call. This delay is to give participants an
opportunity to correct any errors or better explain points that came up.
They are intended to be a communication mechanism of the group, its
discussions, the technical issues, and decisions, not a literal
transcription of what is said.

## Code Purview

At this time, the network working group does not have responsibility for code in any particular directories of Tock. That may change as work progresses, with the possibility of a subdirectory within `capsules/` being created under the purview of this working group.
