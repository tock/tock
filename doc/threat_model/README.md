Tock Threat Model
=================


**Note: This threat model is not descriptive of Tock's current implementation.
It describes how we intend Tock to work as of some future release, perhaps
2.0.**

## Overview

Tock provides hardware-based isolation between processes as well as
language-based isolation between kernel capsules.

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

A **process** is a runtime instantiation of an application binary. When an
application binary "restarts", its process is terminated and a new process is
started using the same binary. Note that the kernel is not considered a process,
although it is a thread of execution.

**Process data** includes a process' binary in non-volatile storage, its memory
footprint in RAM, and any data that conceptually belongs to the process that is
held by the kernel or other processes. For example, if a process requests
samples from an ADC then those samples are considered the process' data, even
when they are stored in a location in RAM only readable by the kernel.

**Kernel data** includes the kernel's image in non-volatile storage as well as
data in RAM that does not conceptually belong to processes. For example, the
scheduler's data structures are kernel data.

**Capsule data** is data that is associated with a particular kernel capsule.
This data can be either kernel data or process data, depending on its
conceptual owner. For example, an ADC driver's configuration is kernel data,
while samples an ADC driver takes on behalf of a process are process data.

**Tock's users** refers to entities that make use of Tock OS. In the context of
threat modelling, this typically refers to board integrators (entities that
combine Tock components into an OS to run on a specific piece of hardware) and
application developers (who consume Tock's APIs and rely on the OS' guarantees).

## Isolation Provided to Processes

**Confidentiality:** A process' data may not be accessed by other processes or
by capsules, unless explicitly permitted by the process. Note that Tock does not
generally provide defense against side channel attacks; see the [Side Channel
Defense](#side-channel-defense) heading below for more details. Additionally,
[Virtualization](Virtualization.md) describes some limitations on isolation for
shared resources.

**Integrity:** Process data may not be modified by other processes or by
capsules, except when allowed by the process.

**Availability:** Processes may not deny service to each other at runtime. As an
exception to this rule, some finite resources may be allocated on a
first-come-first-served basis. This exception is described in detail in
[Virtualization](Virtualization.md).

## Isolation Provided to Kernel Code

**Confidentiality:** Kernel data may not be accessed by processes, except where
explicitly permitted by the owning component. Kernel data may not be accessed by
capsules, except where explicitly permitted by the owning component. The
limitations about [side channel defense](#side-channel-defense) and
[Virtualization](Virtualization.md) that apply to process data also apply to
kernel data.

**Integrity:** Processes and capsules may not modify kernel data except through
APIs intentionally exposed by the owning code.

**Availability:** Processes cannot starve the kernel of resources or otherwise
perform denial-of-service attacks against the kernel. This does not extend to
capsule code; capsule code may deny service to trusted kernel code. As described
in [Virtualization](Virtualization.md), kernel APIs should be designed to
prevent starvation.

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

1. Tock does not hide a process' CPU usage from other processes. Hiding CPU
   utilization generally requires making significant performance tradeoffs, and
   CPU utilization is not a particularly sensitive signal.

1. Although Tock protects a process' data from unauthorized access, Tock does
   not hide the size of a process' data regions. Without virtual memory
   hardware, it is very difficult to hide a process' size, and that size is not
   particularly sensitive.

1. It is often practical to build constant-time cryptographic API
   implementations, and protecting the secrecy of plaintext is valuable. As
   such, it may make sense for a Tock board to expose a cryptographic API with
   some side channel defenses.

### Guaranteed Launching of Binaries

Tock does not guarantee that binaries it finds are launched as processes. For
example, if there is not enough RAM available to launch every binary then the
kernel will skip some binaries.

This parallels the "first-come, first-served" resource reservation process
described in [Virtualization](Virtualization.md#availability).

## Components Trusted to Provide Isolation

The Tock kernel depends on several components (including hardware and software)
in order to implement the above isolation guarantees. Some of these components,
such as the application loader, may vary depending on Tock's use case. The
following documents describe the trust model that exists between the Tock kernel
and its security-relevant dependencies:

- [Capsule Isolation](Capsule_Isolation.md) describes the coding practices used
  to isolate capsules from the remainder of the kernel.

- [Application Loader](Application_Loader.md) describes the trust placed in the
  application deployment mechanism.

- [TBF Headers](TBF_Headers.md) describes the trust model associated with the
  [Tock Binary Format](../TockBinaryFormat.md) headers.

- [Code Review](Code_Review.md) describes code review practices used to ensure
  the trustworthiness of Tock's codebase.

## What is an "Application"?

Tock does not currently have a precise definition of "application", although
there is consensus on the following:

- Unlike a process, an application persists across reboots and updates. For
  example, an application binary can be updated without becoming a new
  application but the update will create a new process.

- An application consists of at least one application binary (in the Tock Binary
  Format), although it is unclear whether multiple application binaries can
  collectively be considered a single application (e.g. if they implement a
  single piece of functionality).

This section will be updated when we have a more precise definition of
"application".
