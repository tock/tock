Network Working Group (NWG)
=================================

- Working Group Charter
- Adopted 8/11/2023

## Goals

The goals of the Network Tock Working Group (NWG) are to:

- design a set of interfaces (traits) for networking
- design the userland interfaces for networking
- define how buffer management should be implemented
- determine other kernel infrastructure useful for networking

## Members

- Alexandru Radovici, Politehnica University of Bucharest
- Branden Ghena (Chair), Northwestern University
- Leon Schuermann, Princeton University
- Tyler Potyondy, UCSD

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

The Network working group is responsible for reviewing, approving, and merging
pull requests related to networking in Tock. This includes the Tock kernel,
libtock-c, and libtock-rs. Within the Tock kernel, this includes but is not
limited to, capsules supporting BLE, CAN, Ethernet, IEEE 802.15.4, and WiFi.
This also includes chip drivers and kernel HILs related to similar subsystems.

