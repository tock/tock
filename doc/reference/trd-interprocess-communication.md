# Interprocess Communication

**TRD:** <br/>
**Working Group:** Network<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Branden Ghena, Leon Schuermann<br/>
**Draft-Created:** 2026/08/02 <br/>
**Draft-Version:** 1 <br/>
**Draft-Discuss:** <devel@lists.tockos.org><br/>

## Abstract

Interprocess communication (IPC) enables two or more processes running on an OS
to share information with each other. This is helpful in building userland
services and coordinating between multiple applications with a shared purpose.
The prior Tock IPC system has proven insufficient for user needs. This document
explains the design of a new ecosystem of IPC capsules meant to support general
needs while being extensible for special-purpose scenarios.

## 1 Introduction

The long-existing (Tock 2.2 and prior) IPC mechanism is based on shared memory
implemented as a [special kernel driver](https://github.com/tock/tock/blob/release-2.2/kernel/src/ipc.rs).
It was initially implemented early in the Tock effort and has shown to have
several issues in practice:
[https://github.com/tock/tock/issues/1993](https://github.com/tock/tock/issues/1993).
Primarily, while shared memory is a useful mechanism, some microcontrollers are
quite limited in their ability to share memory chunks. An alternative system
that uses message passing has been long-discussed but never implemented.

This document discusses a design for replacing the current IPC kernel driver
entirely with a new IPC ecosystem of capsules which provides various
mechanisms for communication between processes. Basing a significant portion of
the work in capsules should allow for more secure and extensible IPC mechanisms.
Each IPC mechanism can expose its own syscall driver interface, which userspace
applications can make requests to. Applications can use multiple IPC capsules
in combination to fulfill their needs.

Some portions of enabling IPC still needs to exist in the kernel. These take the
form of capability-limited interfaces and are limited to necessary features for
managing processes and memory.

The initial IPC mechanisms primarily focus on client-server pattern where a
server contains some resources that it provides to one or more clients. A
review of users' use cases has shown that enabling service applications is a
primary goal for a new IPC system. Servers register with the OS and wait for a
client to reach out to it. Clients discover servers and initiate communication.
After initiation, long-term communication can be triggered bidirectionally,
depending on which mechanisms are used.

Basic mechanisms in the IPC ecosystem include:

* **IPC Registry**: registration and discovery of services
* **IPC Relay**: single-copy message passing
* **IPC Share**: shared memory

Each of these mechanisms include several implementations which could be used
individually or in combination with each other.

### 1.1 Goals

**Fulfill common application-scenario requirements.**
The IPC mechanisms provided should support common application design patterns.
A particular focus is on client-server communication as that has shown to be
common among many use cases. We should ensure the IPC system has some capability
for all common use cases, rather than being optimized for a select set of use
cases.

**Enable extensions with alternative IPC mechanisms.**
While Tock should provide a variety of mechanisms to cover common use cases, it
seems unlikely that those mechanisms will suffice for all use cases. Instead,
the IPC system should be extensible through the creation of additional or
alternative capsules. These could be created by downstream users, possibly
without additional kernel interfaces, and the most useful could be upstreamed
into mainline Tock. This includes the registration and discovery system, which
can be substituted with other implementations as long as it provides IPC
identifiers usable by other capsules.

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

### 1.2 Non-goals

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
implementations, with some basic mechanisms provided upstream initially and
others added as needed. Additional capsules should be creatable downstream which
could revise the interface of existing mechanisms, optimize existing mechanisms
for a particular use case, or provide altogether new mechanisms.

**Peer-to-Peer Discovery and Initiation.**
The primary focus of the IPC design is to support client-server interactions
where clients discover servers and initiate a connection. The current belief is
that most applications will have a clear differentiation between servers which
have resources and clients which take actions to connect with them. As an
example of how this is applied to designs, in the current design servers
"discover" clients when the client first contacts it, usually via a Relay
mechanism. An alternative design where any process could discover and reach out
to any other process is possible, but is not the current focus. Note that this
does not restrain communication patterns after initiation. Both client requests
and asynchronous server notifications are supported in communication mechanisms.

### 1.3 Security

Two primary security models were in mind for this IPC design. The first is
a system without security concerns. While this is a trivial case, it does
include most development of Tock, some demos, and often Tock for use as a
research platform. In this system, processes are trusted to not impersonate
other processes or intentionally deny service to other processes.

The other security model is a system where all apps are signed by a single
developer and checked before being loaded. In this case, all applications
running on the system have been confirmed by the developer (either as a group or
individually). In practice, this has similar expectations for processes, where
they can be trusted to not impersonate other processes or intentionally deny
services.

A model not immediately targeted by this design is the "app store model" where
apps signed by multiple developers coexist on a single platform. In this case,
these apps can only be partially trusted for good behavior. They could
impersonate other processes or deny service to other processes.

Preventing impersonation would require server and client validation. The ability
to validate servers and clients is not part of the initial design, but we
believe that the design enables it to be implemented if desired. Particularly,
registration and discovery have been broken into an asynchronous operation, with
an upcall confirming the result. This allows validation, possibly asynchronous
validation, to be inserted into the operation without change to the process
implementations.

**Server validation** is the more straightforward of the two to implement. As
part of the registration command, the capsule would make a request to the kernel
to perform some type of validation. The exact details of this validation process
are left open to determination. Once validation is completed, a call would be
made to the capsule to complete the registration operation, either successfully
or with an error condition.

**Client validation** would require more effort. The core issue with validating
clients is that IPC Identifiers are forgeable in their current design. A client
could simply guess a 64-bit value that maps to an existing process. So even if
client validation was inserted into the discovery process, it would not actually
guarantee validated clients. Instead, we believe that client validation would
require a modification of the IPC Identifier design to include a
process-specific lookup table that maps from IPC Identifiers to discovered
processes. We discuss this possibility below as
"[Process Descriptor Tables](#22-potential-alternative-designs)". This was not
implemented in the initial design due to the effort involved in creation of
descriptor tables.

Preventing denial of service should be more straightforward and is attempted in
this IPC design wherever possible. In the IPC Relay Request system, requests
SHOULD be received in a round robin fashion, cycling through requesters. In the
IPC Relay Message system, we added the concept of an allowlist to enable a
process to choose which other processes can send it messages.

## 2 IPC Identifier

The IPC Identifier is a type loosely binding the entire IPC ecosystem of
capsules. It is an opaque handle which can be used by processes to identify
another potential process as a target for communication.

IPC Identifiers do not necessarily refer to an existing process. For example,
they can refer to a previously-existing processes which has now terminated
Communication with non-existent processes won't succeed, but the IPC Identifier
can still exist.

Each IPC Identifier MUST have a single, unique 64-bit encoding. This means
every IPC Identifier has a single 64-bit value it corresponds to. And every
64-bit value has a single IPC Identifier it corresponds to. Userspace can
always treat an IPC Identifier as an unsigned 64-bit integer. This can be split
into two 32-bit halves (upper and lower) for transmission through syscalls and
upcalls.

Typically userspace gets an IPC Identifier via an IPC Registry capsule as part
of the discovery process. This is not a hard requirement, however, and it's
possible for processes to receive IPC Identifiers through other means.

Within the kernel, an IPC Identifier can be generated from a 64-bit value
provided from userspace or directly from a given ProcessId. Capsules should also
treat IPC Identifiers as opaque handles, and their internal implementation is
subject to change.

IPC Identifiers are tied to the lifetime of the currently running process,
rather than a given application. If the process restarts, its IPC Identifier
MUST change. Communication destined for the original IPC Identifier will fail,
which has the benefit of signaling that the interaction needs to be restarted
entirely.

### 2.1 Current Implementation

The current implementation of IPC Identifier is found in
[capsules/extra/src/ipc/ipc_identifier.rs](../../capsules/extra/src/ipc/ipc_identifier.rs)

Internally, it uses a combination of ProcessID and ShortID to identify a
process. The combination of the two of these uniquely identify an application
and a process instance, which fit the implementation requirements of IPC
Identifier.

The ID number of a process ProcessID is a 32-bit, monotonically increasing
number assigned to a process when it starts. It already changes each time the
application restarts.

The ShortID is a 32-bit number assigned to an application, which is constant
across restarts of the application. However, for many boards, the ShortID is set
to LocallyUnique, which maps to a value of 0.

Particularly given the structure inherent to the current implementation, be
aware that IPC Identifiers are forgeable. It's possible for a client to guess at
IPC Identifiers for existing processes. So simply knowing an IPC Identifier
should not be used as proof that access has been previously validated.

### 2.2 Potential Alternative Designs

One remaining challenge is that IPC Identifiers as implemented are insufficient
as an access control mechanism. Nothing is stops an application from crafting
their own identifier value to refer attempt to refer to another process. This is
helpful for testing and debugging, but also means that access control of clients
through a system like the IPC Registry capsules is insecure.

A secure implementation for client authentication would require some type of
process descriptor, which is used to access a kernel-managed table of other
processes a client has been given permission to communicate with. The table
would map a process-specific descriptor number into an IPC Identifier. A
fixed-size table would limit the number of other applications communicated with,
but could be configured at initialization time. A dynamic-size table would
require dynamic allocation of grant space and could fail at runtime.

Another alternative access control implementation could be to push
authentication into userspace. This would extend the allowlist idea from
IPC Relay Message to other capsules. To add IPC Identifiers to the allowlist,
they would need to be determined via some other mechanism. For example, a
"knock" mechanism could request access from a server which would either
permanently allow or deny that IPC Identifier. This would avoid kernel effort at
the cost of additional userspace complexity.

The initial design will likely skip an implementation of any client access
control for simplicity. Systems desiring client authentication could use
standard process authentication mechanisms that already exist in Tock.

## 3 IPC Registry

IPC Registry capsules provide registration for services and discovery for
clients. Upon discovery, it provides an opaque IPC Identifier which can be used
to refer to a process for other IPC mechanisms.

If services or clients should be authenticated and/or authorized, these capsules
could perform that operation at registration-time. The initial design will
likely skip that feature but the design should enable it to be possible and
provide clear locations where it could later be connected if implemented.
Callbacks will be given on registration and discovery, allowing asynchronous
operations to take place before they are completed.

Two separate registration mechanisms are implemented, allowing boards to choose
which they want to use. This was chosen first because there were tradeoffs in
which implementation was most useful as a default, and second as an example of
supporting alternative capsule implementations as part of the IPC ecosystem.

Discovery in both cases is nearly identical. Applications provide an allowed
buffer containing data specifying the name they are searching for, likely in
UTF-8. If no such service exists, discovery fails, otherwise a process ID is
provided on success.

Discovery can fail because a service has not managed to register yet. There's
no guarantee that registration occurs before discovery starts. To overcome
this, applications could repeatedly attempt discovery after a short delay.

### 3.1 IPC Registry Package Name

The first option is the **package name** registry.
Registration uses the "package name" field from the application's TBF header.
This is an arbitrary-length string, which cannot be modified at runtime. An
application with an empty or missing package name field cannot register.
Discovery is performed by allowing an arbitrary-length string name, which MUST
contain UTF-8 values without a null terminator.

The package name registry has the advantage of being fixed for a given
application. If the application is signed, that includes the TBF header field,
giving some amount of validation (assuming the application developer is
trusted). It's likely most useful for security-concious deployments where all
applications are signed by the same developer. A downside for new users is that
the package name field is encoded as part of the build system, rather than being
visible in the application source code.

### 3.2 IPC Registry String Name

The second option is the **string name** registry.
Registration and discovery is performed by matching "names", which are
fixed-length byte arrays (20 bytes at time of writing). Allowed buffers MUST
match the fixed length or commands using them will fail. The byte array can
contain any values, but are typically UTF-8 values without a null terminator.
The default name value of all-zero-values cannot be used for discovery. Values
will be copied from the allowed buffer into a fixed-size allocation in the grant
region.

The string name registry has the advantage of being set by the application code
and clearly visible to developers reading the source code. It could also be
modified at runtime if desired. It's likely most useful for testing/development
and for tutorials, where ease of use and flexibility are desired more than
security. A downside is that any application could pretend to be some other
service, which is likely unacceptable for secure deployments.

### 3.3 Other Potential Registry Designs

Many other registry designs are possible. Any registry design SHOULD support the
same basic registration and discovery commands. They also SHOULD only
successfully complete registration or discovery by sending a callback, allowing
for asynchronous kernel validation to occur before the operation completes.

Registration and discovery by AppID
seems reasonable. That could allow for an "app store" type model, where it's
possible to discover one of several versions of a service by a single vendor.

Alternatively, the board configuration could provide a limited set of services
with registration acting as a petition to the board to designate the process as
one of those services. How a board would determine whether a process should be
designated as one of the listed services is uncertain, possibly some feature of
the TBF header.

## 4 IPC Relay

IPC Relay capsules provide single-copy, communication between processes. This is
typically implemented as a copy from an allowed buffer in one process to an
allowed buffer in another process. These capsules provide basic communication
tools for small amounts of data for which copying is acceptable.

### 4.1 IPC Relay Request

The Relay Request capsule provides a request-and-response system via
allow-to-allow copies. A client can start a transaction by sending a request to
a server. The server can collect an outstanding request on-demand to deal with
it. The server later responds to the client to complete the transaction. This
mechanism is particularly designed to support Remote Procedure Calls (RPCs).

No message data is stored in the capsule itself. Instead, both the client and
server allow two buffers: one read-only allow to send data and one read-write
allow to receive data. Clients must maintain the allow of these buffers for the
duration of an entire transaction to eventually receive a response. Servers only
need to temporarily allow these buffers to collect a request and send a
response.

Requests and responses are arbitrary-length byte buffers of data, with meaning
defined by the processes themselves. The processes SHOULD ensure that their
buffers are appropriately sized to hold request/response data. If a client has
not allowed sufficient memory to hold a server's response, the response  is only
partially copied and the client upcall will indicate this error. Similarly, if a
server has not allowed sufficient memory to hold a client's request, the request
is only partially copied and the command return will indicate this error.

Clients can only have one outstanding request at a time. They may end up waiting
for an arbitrary duration until the server responds to them, but may also cancel
their request at any time. If the server faults in some way, the client may be
stuck waiting for a response which will never occur, so clients SHOULD implement
a process-side timeout and cancel the request if no response has occurred.

Servers act on a single request at a time, and must send a response before
accepting a new request. Typical behavior for a server will be to yield
until a request is waiting. Then it can service that request and check for any
more before yielding again.

Servers never need to wait on clients. Accepting a request immediately either
succeeds or fails as the client must keep its request buffer allowed while
waiting. Clients must also keep buffer space allowed for the response while
waiting, so responses also succeed or fail immediately. Failed responses are
dropped, which could occur because the client canceled the request after the
server started acting on it or because the client faulted in some way.

Servers do not need to be aware of clients in advance, as they will receive a
IPC Identifier with each request. Clients do need to have previously discovered
a IPC Identifier for the server, possibly via an IPC Registry or possibly via a
separate mechanism.

### 4.2 IPC Relay Message

The Relay Message capsule provides a unidirectional message to be sent via
allow-to-allow copies. The intent is for these messages to be from servers (senders) to
clients (receivers), but the interface does not require that behavior and any
two processes could use this mechanism to communicate, including
bi-directionally. Messages are appended to a
[StreamingProcessSlice](https://github.com/tock/tock/blob/release-2.2/kernel/src/utilities/streaming_process_slice.rs)
if space is available. This mechanism is particularly designed to support
asynchronous callbacks, such as packet arrivals from a networking stack.

Servers allow a buffer to be appended to the StreamingProcessSlice for a
specific receiver. Each message in the buffer is prepended the IPC Identifier of
the process that sent the message, and the message length. If the receiver
StreamingProcessSlice lacks space to receive a sender's message, the next upcall
will indicate this error condition.

Senders must know the IPC Identifier of the client they are sending to, possibly
via a prior IPC Relay Request request. Receivers also need to be aware of the
IPC Identifier for any sender they wish to receive from. Receivers keep an
allowlist of which IPC Identifiers are enabled to send messages to them.

This design is still in progress and this documentation will be updated when it
is completed.

### 4.3 Other Potential Relay Designs

The designs of IPC Relay capsules are more open than IPC Registries. There are
no design requirements that all should follow.

We considered a notification-only system where no data values are shared and
only a notification upcall is sent. Similar behavior can be implemented by the
Relay Request capsule by sending requests and responses of length zero.

Another possible design is a publish-and-subscribe topic-based system where data
could be communicated from one-to-many or many-to-many. Or a broadcast system
that is explicitly one-to-all. The use cases requiring such a system were less
clear than for the above mechanisms.

An example of a IPC Relay design which would not require allowed buffers would
be a system for sharing small values (such as a 32-bit number), which could be
stored in the grant space of the sender. This should be straightforward to
implement as an additional capsule if desired.

## 5 IPC Share

To be determined. This design is still in progress and this documentation will
be updated when it is completed.

### 5.1 Kernel Support for Shared Memory

IPC capsule mechanisms will need to rely on the kernel for some functionality.
The goal is for this functionality to be as minimal as possible. Access to
these functions will be controlled with capabilities, which can be provided to
the IPC capsules by the board configuration at initialization time.

To be determined. This design is still in progress and this documentation will
be updated when it is completed.

### 5.2 Other Potential Share Designs

Kernel-owned memory chunk

## 6 Use Cases

The following are example scenarios that use IPC mechanisms or adapt them in
some way. The goal with these examples is to increase confidence that the
provided mechanisms are sufficient.

### 6.1 Thread Network Server

In this example, a Thread network is managed by one application (the server).
The server provides IPC access to the Thread network, allowing clients to send
messages and register to receive incoming messages on a certain port or IP
address. Other applications act as clients to this server.

The mechanisms used are:

* IPC Registry Package Name - registration and discovery
* IPC Relay Request - requests to the Thread server, mostly outgoing Thread packets from a client
* IPC Relay Message - forward incoming Thread packets destined for a client

The Thread server would first register with the IPC Registry Package Name
capsule, initialize the Thread network, and then yield until either Thread work
occurs or an incoming IPC Relay Request callback arrives. Requests would take
the form of either outgoing Thread packets, to be sent and then confirmed in a
response, or opening an incoming port/address for packets to arrive for the
client. In the later case, the server stores information about the registration,
including the client's IPC Identifier. Later, if an incoming packet destined for
that client arrives, the server uses the IPC Relay Message capsule to append it
to the client's StreamingProcessSlice.

Thread clients would first discover the server with the IPC Package Name
Registry. They could make requests of the server over the IPC Relay Request
capsule. A local queue in the client could be used to store multiple outgoing
packets while the first request is still outstanding. Typically clients would
yield on their own mechanisms such as timers or sensor data arrival.

A thread client wishing to receive incoming packets would create a
StreamingProcessSlice which it would allow to the IPC Relay Message capsule for
packet reception. It would also send a different request via the IPC Relay
Request capsule to the Thread server, registering for them. Finally, it would
yield on IPC Relay Message callbacks to handle packet arrivals.

If the server faults and restarts, most clients would determine this upon next
request, which would fail due to the non-existent IPC Identifier. Clients may
also periodically check that the server is still active by sending empty
requests to check for liveness.

### 6.2 Dynamic Application Loading

To be determined.

### 6.3 Automotive IPC Extension

To be determined.

## 7 Implementations

Implementations for IPC mechanisms can be found in
[capsules/extra/src/ipc/](../../capsules/extra/src/ipc/)

At time of writing this includes:

* [IPC Identifier](capsules/extra/src/ipc/ipc_identifier.rs)
* [IPC Registry Package Name](capsules/extra/src/ipc/ipc_registry_package_name.rs)
* [IPC Registry String Name](capsules/extra/src/ipc/ipc_registry_string_name.rs)
* [IPC Relay Request](capsules/extra/src/ipc/ipc_registry_relay_request.rs)

## 8 Authors' Addresses

Branden Ghena <branden@northwestern.edu>

Leon Schuermann <leon@is.currently.online>
