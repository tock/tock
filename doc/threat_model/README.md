Tock Threat Model
=================

## Overview

Tock provides hardware-based isolation between applications as well as
language-based isolation between untrusted kernel capsules.

Tock supports a variety of hardware, including boards defined in the Tock
repository and boards defined "out of tree" in a separate repository.
Additionally, Tock's installation model may vary between different use cases
even when those use cases are based on the same hardware. As a result of Tock's
flexibility, the mechanisms it uses to provide isolation — and the strength of
that isolation — vary from deployment to deployment.

This threat model describes the isolation provided by Tock as well as the trust
model that Tock uses to implement that isolation. Users of Tock, which include
board integrators and application developers, should use this threat model to
understand what isolation Tock provides to them (and what isolation it may not
provide). Tock developers should use this threat model as a guide for how to
provide Tock's isolation guarantees.

## Definitions

These definitions are shared between the documents in this directory.

**Application data** includes an application's image in non-volatile storage,
its memory footprint in RAM, and any data that conceptually belongs to the
application that is held by the kernel or other applications. For example, if an
application requests samples from an ADC then those samples are considered the
application's data, even when they are stored in a location in RAM only readable
by the kernel.

**Kernel data** includes the kernel's image in non-volatile storage as well as
data in RAM that does not conceptually belong to applications. For example, the
scheduler's data structures are kernel data.

**Capsule data** is data that is associated with a particular kernel capsule.
This data can be either kernel data or application data, depending on its
conceptual owner. For example, an ADC driver's sample buffer is capsule data
associated with that ADC.

**Secrets** are pieces of application data, kernel data, and capsule data that
owning code wishes to keep confidential (that is, data the owning code does not
opt to share with another OS component). This term is used to give an overview
of confidentiality guarantees, and is always elaborated upon in this document.

**Tock's users** refers to entities that make use of Tock OS. In the context of
threat modelling, this typically refers to board integrators (entities that
combine Tock components into an OS to run on a specific piece of hardware) and
application developers (who consume Tock's APIs and rely on the OS' guarantees).

## Isolation Provided to Applications

**Confidentiality:** Application secrets may not be accessed by other
applications or by untrusted capsules. Note that Tock does not generally provide
defense against side channel attacks; see the [Side Channel
Defense](#side-channel-defense) heading below for more details. Additionally,
[Virtualization](Virtualization.md) describes some limitations on isolation for
application data transferred over shared buses.

**Integrity:** Application data may not be modified by other applications or by
untrusted capsules, except when allowed by the application.

**Availability:** Applications may not deny service to each other. As an
exception to this rule, some finite resources may be allocated on a
first-come-first-served basis. This exception is described in detail in
[Virtualization](Virtualization.md).

## Isolation Provided to Kernel Code

**Confidentiality:** Kernel secrets may not be accessed by applications. Kernel
code's secrets may not be accessed by untrusted capsules. The limitations about
[side channel defense](#side-channel-defense) and [shared buses](#shared-buses)
that apply to application data also apply to kernel data.

**Integrity:** Applications and untrusted capsules may not modify kernel data
except through APIs intentionally exposed by the owning code.

**Availability:** Applications cannot starve the kernel of resources or
otherwise perform denial-of-service attacks against the kernel. This does not
extend to untrusted capsule code; untrusted capsule code may deny service to
trusted kernel code. As described in [Virtualization](Virtualization.md), kernel
APIs should be designed to prevent starvation.

## Isolation that Tock does NOT Provide

There are practical limits to the isolation that Tock can provide; this section
describes some of those limits.

### Side Channel Defense

In general, Tock's users should assume that Tock does NOT provide side channel
mitigations except where Tock's documentation indicates side channel mitigations
exist.

Tock's answer to "should code X mitigate side channel Y" is generally "no". Many
side channels that Tock can mitigate in theory are too expensive for Tock to
mitigate in practice. As a result, Tock does not mitigate side channels by
default. However, specific Tock components may provide and document their own
side channel mitigation. For instance, Tock may provide a cryptography API that
implements constant-time operations, and may document the side channel defense
in the cryptography API's documentation.

In deciding whether to mitigate a side channel, Tock developers should consider
both the cost of mitigating the side channel as well as the value provided by
mitigating that side channel. For example:

1. Tock does not hide an application's CPU usage from other applications. Hiding
   CPU utilization generally requires making significant performance tradeoffs,
   and CPU utilization is not a particularly sensitive signal.

1. Although Tock protects an application's data from unauthorized access, Tock
   does not hide the size of an application's data regions. Without virtual
   memory hardware, it is very difficult to hide an application's size, and that
   size is not particularly sensitive.

1. It is often practical to build constant-time cryptographic API
   implementations, and protecting the secrecy of plaintext is valuable. As
   such, it may make sense for a Tock board to expose a cryptographic API with
   some side channel defenses.

### Shared Buses

External communication buses (for instance, an internet connection or a UART)
may be shared between applications. When buses are shared between applications,
Tock should provide isolation commensurate with the isolation provided by the
underlying communication technology. For example:

1. A UDP API can provide a mechanism for clients (applications and/or untrusted
   capsules) to gain exclusive access to a port. The UDP API should then prevent
   clients from reading messages sent to other clients or impersonating other
   clients.

1. A UART API with multiple clients cannot determine which client owns received
   data. Tock is unable to prevent one application from reading UART data meant
   for a different application.

## Components Trusted to Provide Isolation

The Tock kernel depends on several components (including hardware and software)
in order to implement the above isolation guarantees. Some of these components,
such as the application loader, may vary depending on Tock's use case. The
following documents describe the trust model that exists between the Tock kernel
and its security-relevant dependencies:

- [Untrusted Capsule Isolation](Untrusted_Capsule_Isolation.md) describes the
  coding practices used to isolate untrusted capsules from the remainder of the
  kernel.

- [Application Loader](Application_Loader.md) describes the trust placed in the
  application deployment mechanism.

- [TBF Headers](TBF_Headers.md) describes the trust model associated with the
  [Tock Binary Format](../TockBinaryFormat.md) headers.

- [Third Party Dependencies](Third_Party_Dependencies.md) describes Tock's
  policy on auditing third-party code.
