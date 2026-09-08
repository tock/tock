# Interprocess Communication Capsules

These capsules provide implementations of interprocess communication mechanisms
that boards can choose to provide to processes. They are loosely bound by the
shared [IPC Identifier](ipc_identifier.rs) which can be used to identify
destinations and sources for communication. The goal of this design is to
provide an ecosystem for IPC which can be extended in the future as desired,
including downstream with application-specific or even board-specific
mechanisms.

## Registry

**Registry** capsules provide a means for processes (services) to list
themselves for discovery by other processes (clients). Discovery results in the
client having a IPC Identifier for the specified service. Different capsules
provide different underlying implementations for registration and/or discovery.
Current capsules allow for registration/discovery using string names or package
names.

## Relay

**Relay** capsules provide a means for single-copy, allow-to-allow
communication between a pair of processes. The current implementation only
includes request-and-response paired messages.
