Design of Kernel Hardware Interface Layers (HILs)
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Philip Levis <br/>
**Draft-Created:** April 1, 2021<br/>
**Draft-Modified:** April 1, 2021<br/>
**Draft-Version:** 1<br/>
**Draft-Discuss:** tock-dev@googlegroups.com</br>

Abstract
-------------------------------

This document describes guidelines and considerations for designing
hardware interface layers (HILs) in the Tock operating system. HILs are
Rust traits that provide a standard interface to a hardware resource, such
as a sensor, a flash chip, a cryptogrpahic accelerator, a bus, or a radio. Developers
adding new HILs to Tock should read this document and verify they have
followed these guidelines.

1 Introduction
===============================

In Tock, a hardware interface layer (HIL) is a collection of Rust traits and types that
provide a standardized API to a hardware resource such as a sensor, flash chip,
cryptographic accelerator, bus, or a radio. Capsules typically use HILs to
provide their functionality. For example, a system call driver capsule that
gives processes access to a temperature sensor relies on having a reference to
an implementation of the `kernel::hil::sensors::TemperatureDriver` trait. This
allows the system call driver capsule to work on top of any implemeentation of
the `TemperatureDriver` trait, whether it is a local, on-chip sensor, an
analog sensor connected to an ADC, or a digital sensor over a bus.

HILs are used for many purposes within the kernel. They can be directly accessed
by kernel services, such as the in-kernel process console using the UART HIL. They
can be exposed to processes with system driver capsules, such as with GPIO. They
can be virtualized to allow multiple clients to share a single resource, such as with
the virtual timer capsule.

This variety of use cases place a complex set of requirements on how a HIL must
behave. For example, Tock expects that every HIL is virtualizable: it is possible
to take one instance of the trait and allow multiple clients to use it simultaneously
through queueing, such that each one thinks it has its own, independent instance of
the trait. Because virtualization means requests can be queued, all HILs must be
nonblocking and so have a callback for completion. This has implications to buffer
management and ownership.

This document describes these requirements and provides a set of design guidelines
for HILs. They are:

1. All split-phase operations MUST return a synchronous error.
2. All split-phase operations with a buffer parameter MUST return a tuple in their error case, which includes the passed buffer as an element.
3. Split-phase operrations with a buffer parameter SHOULD take a mutable reference even if their access is read-only.
4. All split-phase completion callbacks MUST include an error as a parameter; these errors are a superset of the synchronous errors.
5. All split-phase completion callbacks for an operation with a buffer parameter MUST return the buffer.
6. The HIL SHOULD separate control and datapath operations into separate traits.
7. Use fine-grained traits that separate out different use cases.
8. Blocking APIs are not general: use them sparingly, if at all.

The rest of this document describes each of these guidelines and their reasoning.

2 Return Synchronous Error
===============================

2 Return Synchronous Error
===============================

2 Return Synchronous Error
===============================

2 Return Synchronous Error
===============================

2 Return Synchronous Error
===============================

2 Return Synchronous Error
===============================

2 Return Synchronous Error
===============================

2 Return Synchronous Error
===============================







10 Author Address
=================================
```
email - Philip Levis <pal@cs.stanford.edu>
```
