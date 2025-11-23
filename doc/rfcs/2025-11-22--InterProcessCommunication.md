# Interprocess Communication

- Initial Proposal: TBD
- RFC PR: TBD

## Summary

This document proposes a redesign of interprocess communication in Tock. The
goal is to document the system before implementation.


## Overview

The current (Tock 2.2) IPC mechanism is based on shared memory implemented as a
special kernel driver:
https://github.com/tock/tock/blob/release-2.2/kernel/src/ipc.rs It was
initially implemented early in the Tock effort and has shown to have several
issues in practice: https://github.com/tock/tock/issues/1993 Primarily, while
shared memory is a useful mechanism, some microcontrollers are quite limited in
their ability to share memory chunks. An alternative system that uses message
passing has been long-discussed but never implemented.

This RFC proposes replacing the current IPC kernel driver entirely with a new
IPC ecosystem of capsules which will provide various mechanisms for
communication between processes. Basing a significant portion of the work in
capsules should allow for more secure and extensible IPC mechanisms.
Each capsule is also a syscall driver, which userspace applications can make
requests to. Most applications will use multiple IPC capsules in combination
to fulfill their needs.

Some portions of enabling IPC will still need to exist in the kernel. These
will take the form of capability-limited interfaces and will be limited to
necessary features for managing processes and memory.

The IPC design will primarily focus on client-server interactions. A review of
users' use cases has shown that enabling service applications is a primary goal
for a new IPC system. Servers will register with the OS and wait for
communications. Clients will discover servers and trigger communication. Some
mechanisms may also support peer-to-peer communication, but this is not a
first-order priority.

Basic mechanisms in the IPC ecosystem will include: registration and discovery
of services, message passing both synchronous and asynchronous, and shared
memory.


### Goals

**Fulfill common application-scenario requirements.**
The IPC mechanisms provided should support common application design patterns.
A particular focus is on client-server communication as that has shown to be
common among many use cases. We should ensure the IPC system has some
capability for all common use cases, rather than optimal cpability for a select
use cases.

**Enable extensions with alternative IPC mechanisms.**
While Tock should provide a variety of mechanisms to cover common use cases, it
seems unlikely that those mechanisms will suffice for all use cases. Instead,
the IPC system should enable extension through the creation of additional
capsules. These could be created by downstream users, possibly without
additional kernel interfaces, and the most useful could be upstreamed into
mainline Tock.

**Support a wide variety of microcontrollers.**
Any microcontroller that Tock supports should be able to use some IPC
mechanisms. However, it is possible that some mechanisms will be more capable
or more performant on some platforms and less capable or less performant on
others. One example of this is shared memory, where the number of memory
protection regions a microcontroller is capable of will limit the number of
memory regions IPC can support. When possible, IPC mechanisms should scale
their capability based on the microcontroller rather than reject a platform
altogether.

**Mechanisms should primarily be implemented in capsules.**
Capsules should be used to implement IPC mechanisms, with as much of the
functionality as possible provided within the capsule. This is valuable as
capsules are easier to extend for downstream users and cannot use unsafe Rust.
Some functionality by necessity will need to exist in the kernel. This
functionality should be as minimal as possible and should be protected from
access with capabilities.


### Non-goals

**Optimal performance.**
Much of IPC research focuses on optimal performance to enable microkernel
designs. Our focus is instead of client-server interactions between userspace
applications which may not require the best possible performance. Where
possible, performance should be valued, but we will weigh that value when in
conflict with other issues such as usability.

**One perfect mechanism.**
We do not believe that there is any single best IPC mechanism that would
support all application needs. Instead, we focus on an ecosystem of capsule
implementations, with some basic mechanisms provided initially and others added
as-needed. Additional capsules should be creatable downstream which could
revise the interface of existing mechanisms, optimize existing mechanisms for a
particular use case, or provide altogether new mechanisms.

**Peer-to-peer communication.**
The primary focus of the IPC design is to support client-server interactions.
The current belief is that true peer-to-peer communication is not as commonly
needed by Tock applications. However, the design should not preclude the
future addition of peer-to-peer communication mechanisms if possible.


## IPC Manager Capsule

This capsule provides registration for services and discovery for clients.
Upon discovery, it provides opaque process IDs which can be used to refer to
processes in other IPC mechanisms.

If validation of services or clients is desired, this capsule could perform
that operation at registration-time. The initial design will likely skip that
feature but the design should enable it to be possible and provide clear
locations where it could later be connected if implemented. Callbacks will be
given on registration and discovery, allowing asynchronous operations to take
place before they are completed.

**Commands**:
* Existence
* Register as service with allowed string name
* Discover service with allowed string name

**Allows**:
* Read-only, string name of service

**Subscribes**:
* Registration complete (success or failure)
* Discovery complete (provides process ID)


## Synchronous Mailbox Capsule

This capsule provides synchronous client-to-server request-and-response
messages. Clients can send a request to any process ID. After receiving a
request message, the server performs whatever action it chooses and eventually
sends a response message back to the client. Clients may wait end up waiting
for an arbitrary duration until the server responds to them, but may also
cancel their request at any time.

Servers do not need to be aware of clients in advance, as they will receive a
process ID with each request. Clients do need to have previously discovered a
process ID for the server, possibly via the IPC Manager or possibly via a
separate mechanism (for example, they could be hard-coded).

Each client can only have one outstanding request at a time. However, they can
cancel that request if a server is not responsive. A response callback will
occur once the request has been serviced. If a server application stops
existing (faults, restarts, etc.) after a request is made but before it is
responded to, the client will receive a fault callback instead.

Servers are never required to wait on clients, with responses either being
accepted immediately or dropped. Typical behavior for a server will be to yield
until a request is waiting. Then it can service that request and check for any
more before yielding again.

The implementation for synchronous mailbox should use a single copy from
allowed memory to allowed memory. No message is ever stored within the capsule
itself. Clients will allow two buffers, one containing the request and one for
the response to be written to. Upcalls occur on request or response.

**Commands**:
* Existence
* Client, send request to process ID
* Client, cancel request
* Server, get any next request
* Server, get next request from process ID
* Server, send response to process ID

**Allows**:
* Read-only, buffer to read from (client-request or server-response)
* Read-write, buffer to write into (client-response or server-request)

**Subscribes**:
* Client, response received
* Client, error received
* Server, request waiting


## Asynchronous Mailbox Capsule

The capsule provides asynchronous one-directional messages. The intent is for
these messages to be from servers to clients and this documentation will assume
that flow, but the interface does not require that behavior and any two
processes could use this mechanism to communicate, including bi-directionally.
Messages are appended to a
[StreamingProcessSlice](https://github.com/tock/tock/blob/release-2.2/kernel/src/utilities/streaming_process_slice.rs)
if space is available.

Servers must know the process ID of the client they are sending to. This could
have previously been received through a request via the Synchronous Mailbox
mechanism or possibly via a separate mechanism (IPC Manager or hard-coded).
Clients also need to be aware of the process ID for any server they wish to
receive from, which they will specify in an allowlist. Again, this could
previously have been received from the IPC Manager mechanism or alternatively.

Clients provide an allowlist of services which can send messages to them and a
buffer containing a StreamingProcessSlice. Clients receive a callback when one
or more messages have been appended for them.

Servers allow a buffer to be appended to the StreamingProcessSlice for a
specific client. The client will also receive the process ID of the server that
sent the message.

**Commands**:
* Existence
* Client, enable async reception
* Client, disable async reception
* Server, send async message to process ID

**Allows**:
* Read-only, server, buffer to send from
* Read-write, client, StreamingProcessSlice to receive into
* Read-only, client, allowlist of which process IDs to accept messages from

**Subscribes**:
* Client, Async message received


## Shared Memory Capsule
TBD

## Kernel Functionality

IPC capsule mechanisms will need to rely on the kernel for some functionality.
The goal is for this functionality to be as minimal as possible. Access to
these functions will be controlled with capabilities, which can be provided to
the IPC capsules by the Board at initialization time.


### Process IDs



### Application Status Client


### Memory Protection Configuration
TBD

### Process State Configuration
TBD

### Allow Changes
TBD


## Use Case Examples

### Thread Network Server

### Dynamic Application Loading

### Automotive IPC Extension


