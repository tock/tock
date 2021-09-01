Application IDs (AppID)
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Philip Levis, Johnathan van Why<br/>
**Draft-Created:** 2021/09/01 <br/>
**Draft-Modified:** 2021/09/01 <br/>
**Draft-Version:** 1 <br/>
**Draft-Discuss:** tock-dev@googlegroups.com<br/>

Abstract
-------------------------------

This document describes the design and implementation of application
identifiers (AppIDs) in the Tock operating system. AppIDs provide a
mechanism to identify the application contained in a userspace binary.
AppIDs allow the kernel to apply security policies to applications as
their code evolves and their binaries change. A board defines how the
kernel verifies AppIDs and which AppIDs the kernel will load. This
document describes the Rust traits and software architecture for
AppIDs as well as the reasoning behind them. This document is in full
compliance with [TRD1](./trd1-trds.md).

1 Introduction
===============================

The Tock kernel needs to be able to manage and restrict what userspace
applications can do. Examples include:
  - making sure other applications cannot access an application's sensitive data stored in non-volatile memory,
  - restricting certain system calls to be used only by trusted applications,
  - restrict runnable applications to those signed by a trusted party.

In order to accomplish this, the kernel needs a way to identify an
application and know whether a particular userspace binary belongs to
an application. The mapping between binaries and applications is
many-to-many. Multiple binaries can be associated with an application
when software updates version the binary or when an application needs
to run in multiple processes. An example of a binary associated with
multiple applications is a program that migrates data from one
application to another (e.g., transitions keys from an old U2F
application to a new one).

The Tock kernel makes minimal assumptions on the structure and form of
application credentials that bind an application identifier to a
binary. Application credentials are arbitrary k-byte sequences that
are stored in an userspace binary's Tock binary format (TBF)
headers. When a Tock board instantiates the kernel, it passes a
reference to an AppID (application identifier) verifier, which the
kernel uses to determine the AppIDs of each userspace binary it reads
and decide whether to load the binary into a process.

Most of the complications in AppIDs stem from the fact that they are a
general mechanism used for many different use cases. Therefore, the
exact structure and semantics of application credentials can vary
widely. Tock's TBF header formats, kernel interfaces and mechanisms
must accomodate this wide range.

The interfaces and standard implemenentations for AppIDs and AppID
verifiers are in the kernel crate, in module `appid`. There are two
main traits:

  * `kernel::appid::Verifier` is responsible for defining which types
  of application identifier it can accepts and whether it accepts a
  particular application identifier for a specific application
  binary. The kernel only loads application binaries the `Verifier`
  accepts.
  
  * `kernel::appid::Compress` is repsonsible for compressing
  application identifiers into short, 32-bit identifiers called
  `ShortID`s. `ShortID`s provide a mechanism for fast comparison,
  e.g., for an application identifier against an access control list.
  

2 Terminology
===============================

This document uses several terms in precise ways. Because these terms
overlap somewhat with general terminology in the Tock kernel, this
section defines them for clarity. The Tock kernel often uses the term
"application" to refer to what this document calls a "Tock binary."

**Application**: userspace software developed and maintained by an
individual, group, corporation, or organization that meets the
requirements of a particular Tock device use case. An application can
consist of a single userspace binary, multiple userspace binaries that
run concurrently (an application split into several processes), or
multiple userspace binaries only one of which runs at a time (an
application with multiple versions).

**Tock binary**: a Tock binary format (TBF)[TBF] object stored on a
Tock device, containing TBF headers and an application binary.

**Application binary**: a code image compiled to run in a Tock
process.

**Application identifier**: a numerical identifier for an application.

**Global application identifier**: an application identifier which is
globally unique. All instances of the application have this identifier
and no instances of other applications have this identifier.

**Local application identifer**: an application identifier which is
locally unique. No instances of other applications on the same
Tock device have this identifier.

**Application credentials**: data that binds an application identifier
to an application binary. Application credentials are usually stored
in Tock binary format[TBF] headers.

In normal use of Tock, Tock binaries are copied into an application
flash region by a software tool. When the Tock kernel boots, it scans
this application flash region for Tock binaries. After inspecting the
application binary and TBF headers in a Tock binary, the kernel
decides whether to load it into a process and run it.

3 Application identifiers and application credentials
===============================

To explain the distinction between application identifiers and credentials,
consider these four use cases.
  
  1. The application has a single process. It has no application
  credentials: it only runs on kernels that are willing to load
  applications without credentials (e.g., research systems).

  1. The application has a single process. The application is
  identified by a public key (e.g., an RSA public key) that is also a
  global application identifier. This application's application
  credentials consist of a TBF header containing this key as well as a
  signed SHA512 hash of the application binary, signed with the
  private key corresponding to the public key in the header. The
  kernel decides whether to accept a particular public key for
  verification.

  1. The application has a single process. The application is
  identified by a unique identifier I. This application's application
  credentials consist of a TBF header containing a public key, the
  unique identifier I, as well as a signed SHA512 hash of the
  application binary and unique identifier I, signed with the private
  key corresponding to the public key in the header. The kernel
  decides whether to accept a particular public key for verification.

4 `Verifier` trait
===============================

5 Short IDs and the `Compress` trait
===============================


6 Capsules
===============================

This section describes the standard Tock capsules for SPI communication.

7 Implementation Considerations
===============================

8 Authors' Address
=================================
```
Philip Levis
409 Gates Hall
Stanford University
Stanford, CA 94305
USA
pal@cs.stanford.edu

Alexandru Radovici <msg4alex@gmail.com>
```
