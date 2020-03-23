Virtualization
==============

Tock components that share resources between multiple clients (which may be
kernel components, applications, or a mix of both) are responsible for providing
confidentiality and availability guarantees to those clients.

## Data Sharing (Confidentiality)

In general, kernel components with multiple clients should not share data
between their clients. Furthermore, data from a client should not end up in a
capsule the client is unaware of.

When a capsule with multiple clients is given a buffer by one of those clients,
it must do one of the following:

1. Avoid sharing the buffer with any other kernel code. Return the buffer to the
   same client.

1. Only share the buffer downwards, to lower-level components. For example, a
   capsule providing virtualized access to a piece of hardware may pass the
   buffer to the driver for that hardware.

1. Wipe the buffer before sharing it with another client.

Kernel components with multiple clients that retrieve data on behalf of those
clients must implement isolation commensurate with their functionality. When
possible, components reading from shared buses should mux data transferred over
those buses. For example:

1. A UDP API can provide a mechanism for clients (applications and/or untrusted
   capsules) to gain exclusive access to a port. The UDP API should then prevent
   clients from reading messages sent to other clients or impersonating other
   clients.

1. A UART API with multiple clients should implement a protocol that allows the
   UART API to determine which client a received packet belongs to and route it
   accordingly (in other words, it should implement some form of muxing).

1. An Analog-to-Digital Converter (ADC) does not have a concept of which process
   "owns" data, nor is there a way to implement such a concept. As such, an ADC
   API that allows clients to take samples upon request does not need to take
   separate samples for different clients. An ADC API that receives simultaneous
   requests to sample the same source may take a single reading and distribute
   it to multiple clients.

## Fairness (Availability)

Tock components do not need to guarantee fairness between clients. For example,
a UART virtualization layer may allow capsules/apps using large buffers to see
higher throughputs than capsules/apps using small buffers. However, components
should prevent starvation when the semantics of the operation allow it. For the
UART example, this means using round-robin scheduling rather than preferring
lower-numbered clients.

When it is not possible to prevent starvation — such as shared resources that
may be locked for indefinite amounts of time — then components have two
options:

1. Allow resource reservations on a first-come, first-served basis. This is
   essentially equivalent to allowing clients to take out unreturnable locks on
   the resources.

1. Restrict access to the API using a kernel capability (only possible for
   internal kernel APIs).

An example of an API that would allow first-come-first-served reservations is
crypto hardware with a finite number of non-sharable registers. In this case,
different apps can use different registers, but if the registers are
overcommitted then later/slower apps will be unable to reserve resources.

An example of an API that would be protected via a kernel capability is
indefinite continuous ADC sampling that blocks other ADC requests. In this case,
first-come-first-served reservations do not make sense because only one client
can be supported anyway.
