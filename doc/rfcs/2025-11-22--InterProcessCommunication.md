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
Each IPC mechanism can expose its own syscall driver interface, which userspace
applications can make requests to. Applications can use multiple IPC capsules
in combination to fulfill their needs.

Some portions of enabling IPC will still need to exist in the kernel. These
will take the form of capability-limited interfaces and will be limited to
necessary features for managing processes and memory.

The initial IPC mechanisms will primarily focus on client-server interactions.
A review of users' use cases has shown that enabling service applications is
a primary goal for a new IPC system. Servers will register with the OS and wait
for communications. Clients will discover servers and trigger communication.
Some mechanisms may also support peer-to-peer communication, but this is not a
first-order priority.

<!--

Leon: I don't agree with the above. I think that some of the use-cases we
discussed (such as a "network stack service") will have more complex
communication patterns than mere request-response. For instance, to avoid
entirely blocking a client that waits for the next request, they should have at
least a notification mechanism to indicate that a new packet arrived. This is
closer to what I remember from our previous discussions. Also, I don't know why
the document needs to set this premise explicitly, given that we want to design
an ecosystem of mechanisms instead of a single one, implementing a single
pattern.

-->


Basic mechanisms in the IPC ecosystem will include: registration and discovery
of services, message passing both synchronous and asynchronous, and shared
memory.


### Goals

**Fulfill common application-scenario requirements.**
The IPC mechanisms provided should support common application design patterns.
A particular focus is on client-server communication as that has shown to be
common among many use cases. We should ensure the IPC system has some capability
for all common use cases, rather than being optimized for a select set of use
cases.

**Enable extensions with alternative IPC mechanisms.**
While Tock should provide a variety of mechanisms to cover common use cases, it
seems unlikely that those mechanisms will suffice for all use cases. Instead,
the IPC system should be extensible through the creation of additional
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
conflict with other issues such as usability. However, the IPC system should
be extensible to enable performance-focused designs for downstream users with
stricter performance requirements.

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

<!--

I think we should do a better job of deliminating what "peer-to-peer"
communication means.

When we talk about the simplest form of client--server communication, it's clear
how to distinguish it: in P2P, you can have the "server" initate requests to the
client. However, when we're adding features like asynchronous messages being
delivered from a server to a client (as we'd need for a network stack exposing
an "IPC server"), then you kind of have all primitives required for "P2P
communication", in the sense that both sides can initiate communication by
sending a type of signal (either a client->server request, or a server->client
signal).

I think that we should instead define that the IPC system assumes that there are
two entities with different roles: there is a client to initate communication,
and a server offering a discoverable service.

This says little about the communication patterns that these entities will
engage in, once communication is established. And I think that's good, because
those patterns are ultimately going to be governed by the interfaces offered by
the IPC capsules.

-->


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

Registration and discovery is performed by matching "names", which are
twenty-byte arrays. Allowed buffers must be exactly twenty bytes in length
or commands using them will fail. The byte values can be anything, but are
typically ASCII values, which do not need to be null-terminated. The default
name value of all-zero-values cannot be used for discovery. Values will be
copied from the allowed buffer into a fixed-size allocation in the grant
region.

An alternative mechanism of using package identifiers was considered, but was
less flexible than having arbitrary process-specified identifiers. Package
identifiers have some advantages that they are 1) arbitrary length, and 2) are
unable to be modified at runtime. A downside is that they are encoded as part
of the build system, and not part of the application.

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
sends a response message back to the client. Clients may end up waiting
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
the response to be written to. Upcalls occur on request or response. If a
client has not allowed sufficient memory to hold  a server's reply, the reply
is only partially copied and the client upcall will indicate this error.

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
specific client. Each message in the buffer is prepended the process ID of the
server that sent the message, and the message length. If the client
StreamingProcessSlice lacks space to receive a server's message, the next
upcall will indicate this error condition.

In the case that a server in the allowlist of a client has a state change (faults),
the client will be notified in a separate callback.

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
* Client, async message received
* Client, error with allowlist ProcessID


## Shared Memory Capsule
TBD


## Kernel Functionality

IPC capsule mechanisms will need to rely on the kernel for some functionality.
The goal is for this functionality to be as minimal as possible. Access to
these functions will be controlled with capabilities, which can be provided to
the IPC capsules by the Board at initialization time.


### Process Descriptor Tables

IPC capsules rely on an opaque identifier for each process being communicated
with. These identifiers will be stored in userspace by clients and servers to
enable future communication. In the kernel, these identifiers will be used to
look up allowed data and grant-stored configurations.

The kernel already has such a mechanism: ProcessID. The Process Standard
implementation requests process identifiers from the kernel, which returns a
monotonically increasing number. ProcessIDs are unique to a running instance of
a process, with a process receiving a new ProcessID if it restarts. The `usize`
identifier within ProcessIDs can be given to userspace, which capsules can hold
on to a ProcessID reference.

Nothing in particular should be required to use ProcessIDs for IPC, in fact
they were designed with IPC in mind. That they are not identical across process
restarts is a boon, as clients will almost certainly need to restart
communication if a server restarts.

One remaining challenge is that ProcessIDs as designed are insufficient as an
access control mechanism. Nothing is stops an application from crafting their
own identifier value to refer to attempt to refer to another process. This
is helpful for testing and debugging, but also means that access control of
clients through a system like the IPC Manager capsule is insecure.

A secure implementation for client authentication would require some type of
process descriptor, which is used to access a kernel-managed table of other
processes a client has been given permission to communicate with. The table
would map a process-specific descriptor number into a ProcessID. A fixed-size
table would limit the number of other applications communicated with, but could
be configured at initialization time. A dynamic-size table would require
dynamic allocation of grant space and could fail at runtime.

Another alternative access control implementation could be to push
authentication into userspace. This would extend the allowlist idea from
Asynchronous Mailbox to other capsules. To add ProcessIDs to the allowlist,
they would need to be determined via some other mechanism. For example, a
"knock" mechanism could request access from a server which would either
permanently allow or deny that ProcessID. This would avoid kernel effort at the
cost of additional userspace complexity.

The initial design will likely skip an implementation of any client access
control, and instead utilize ProcessIDs directly. Systems desiring client
authentication could use standard process authentication mechanisms that
already exist in Tock.


### Process Status Callback

IPC capsules may need to receive a callback if the state of a process changes.
This would require the addition of a process status callback mechanism to the
kernel, where IPC mechanism(s) could register as clients.

As a specific example, the Synchronous Mailbox capsule allows clients to send a
request to a server, which may not immediately have room for the client's
message data. Instead, the data is left in the client's allow buffer until the
server requests it. But if the client successfully registers data to be sent,
and the server status changes after this point, the client would remain in limbo
as the message would never be fully received and responded to. Similarly, the
Asynchronous Mailbox needs a mechanism to notify clients that no further packets
will arrive from a process.

One way around this issue is to have userspace timeouts for Synchronous Mailbox
requests. The client could then cancel the request, or possibly take some
action to determine if the server ID it has is still valid. However, this seems
difficult to tune properly and can lead to runtime faults that are complex to
debug.

An alternative method would be for the capsule itself to realize that the
server is not valid and notify the client. The client could then reset its
state machine and start discovery of the server again. To do this, the capsule
would need notice that the process state has changed.

A proposed implementation of this feature would be to add it to the
KernelResources trait, with the ContextSwitchCallback being a good example. The
[Read Only State
capsule](https://github.com/tock/tock/blob/release-2.2/capsules/extra/src/read_only_state.rs)
uses ContextSwitchCallback to act whenever a context switch occurs. After
adding ProcessStateCallback to kernel resources, board files would then use
their implementation to call into one or more IPC capsules which require this
information. Finally, the kernel would need to be modified to call into this
hook whenever a process state change occurs.


### Memory Protection Configuration
TBD

### Process State Configuration
TBD

### Allow Changes
TBD


## Use Case Examples

The following are example scenarios that use IPC mechanisms or adapt them in
some way. The goal with these examples is to increase confidence that the
provided mechanisms are sufficient.

### Thread Network Server

In this example, a Thread network is managed by one application (the server).
The server provides IPC access to the Thread network, allowing clients to send
messages and register to receive incoming messages on a certain port or IP
address. Other applications act as clients to this server.

The mechanisms used are:
 * IPC Manager - registration and discovery
 * Synchronous Mailbox - requests to Thread server, mostly outgoing Thread packets from a client
 * Asynchronous Mailbox - incoming Thread packets for a client

The Thread server would first register with the IPC Manager, initialize the
Thread network, and then yield until either Thread work occurs or an incoming
Synchronous Mailbox request arrives. Requests would take the form of either
outgoing Thread packets, to be sent and then confirmed in a response, or
opening an incoming port/address for packets to arrive for the client. In the
later case, the server stores information about the registration, including the
client's ProcessID. Later, if an incoming packet destined for that client
arrives, the server uses the Asynchronous Mailbox to append it to the client's
StreamingProcessSlice.

Thread clients would first discover the server with the IPC Manager. They could
make requests of the server over the Synchronous Mailbox. A local queue in the
client could be used to store multiple outgoing packets while the first request
is still outstanding. Typically clients would yield on their own mechanisms
such as timers or sensor data arrival.

A thread client wishing to receive incoming packets would create a
StreamingProcessSlice which it would allow to the Asynchronous Mailbox for
packet reception. It would also send a different request via the Synchronous
Mailbox to the Thread server, registering for them. Finally, it would yield on
Asynchronous Mailbox callbacks to handle packet arrivals.

If the server faults and restarts, most clients would determine this upon next
Synchronous Mailbox request, which would fail due to the non-existent
ProcessID. Outstanding Synchronous Mailbox or Asynchronous Mailbox requests
would result in an error callback.


### Dynamic Application Loading
TBD

### Automotive IPC Extension
TBD


## Alternative Mechanisms Considered
TBD

