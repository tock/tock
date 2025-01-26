Tock on Constrained Platforms Without Execute-in-Place (XIP) Flash
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Informational<br/>
**Status:** Draft <br/>
**Author:** Brad Campbell<br/>
**Draft-Created:** 2024/07/05<br/>
**Draft-Modified:** 2025/01/15<br/>
**Draft-Version:** 1<br/>
**Draft-Discuss:** devel@lists.tockos.org<br/>

Abstract
-------------------------------

This document provides information relating to running Tock on a subset of
platforms without execute-in-place flash storage.

**This is an initial, draft document and is not intended as authoritative.
Feedback from downstream users on XIP use cases, needs, experiences, etc
is greatly appreciated.**

Introduction
===============================

Traditional embedded systems use execute-in-place (XIP) flash to both store
program code and to enable executing that program code. However, some hardware
designs, particularly those integrated into a system-on-chip alongside an
application processor, do not include executable persistent storage. These
platforms, subsequently referred to as "non-XIP", can only execute instructions
stored in volatile RAM. All executable code must be loaded from some external
source, stored into RAM, and then executed.

The external source that provides the executable code may be transient. For
example, one common use-case during development is the ability to stream code
directly into RAM via JTAG and then execute from RAM, without having to store
that code in flash at any intermediate point. While some platforms may have
runtime access to the external source for executable code, in general non-XIP
design assumes it does not have access to external source after the initial
load.

There exists a wide design space of non-XIP platforms. This document is focused
on "constrained" non-XIP platforms, subsequently referred to as "con-non-XIP".
In this context, "constrained" refers to the size of the available RAM, and
indicates that the available RAM is generally less than the expected size of XIP
flash on typical Tock platforms. That is, these platforms are, in general,
unable to run a typical Tock instance from RAM as a conventional MCU would run
Tock from flash.

This presents challenges for executing Tock on such platforms as Tock generally
expects to not only have XIP flash, but enough flash to support the overhead of
discrete applications and their headers. Concretely, Tock platforms generally
have at least 512 kB of flash. Supporting con-non-XIP platforms may require new
interfaces or introduce design considerations that Tock on traditional platforms
would otherwise not consider.

1.0 Background
===============================

A con-non-XIP platform has no persistent flash storage where compiled binaries
can be stored and executed. All static program code and data must be stored in
some nonvolatile storage and then be loaded into RAM to execute.

This system architecture shrinks the amount of the hardware that must be trusted
when securely executing programs. By not including execute-in-place flash, the
flash storage does not need to be trusted to correctly store program code and
data. When the program is copied to RAM the program binary can be verified
(i.e., by checking a cryptographic signature) before the loaded binary is
allowed to execute.



2.0 Tradeoffs
===============================

Non-XIP platforms introduce a new design point for embedded devices with a
different set of tradeoffs compared to traditional platforms. To help motivate
potential Tock designs for con-non-XIP platforms, we discuss both the
limitations compared to traditional platforms, and the properties that may be
important on traditional platforms but are likely not a priority on con-non-XIP
platforms.

Limitations:

- Non-XIP platforms have less RAM that traditional platforms have XIP flash.
  Further, this RAM on con-non-XIP platforms must (at runtime) store both the
  executable code and the conventional memory items (e.g., the stack and heap).
  This results in significantly less room for program code than on traditional
  platforms.
- As flash storage may not be trustworthy, code retrieved from flash storage may
  need to be verified before it is used.

Non-limitations:

- Energy is not as significant of bottleneck as in other settings, and
  additional power draw from having to load program code into RAM is acceptable.
- For platforms with persistent access to an external code source and more
  applications than available SRAM, the execution time for most applications
  have relaxed deadlines, and the additional execution time from having to
  dynamically load program code into RAM is acceptable.
   - This observation does not prohibit "pinning" timing-critical apps or other
     techniques to ameliorate this concern when necessary.
- Loading code into RAM allows more predictable layout of code and data and/or
  rewriting some instructions during load time, potentially enabling simpler
  relocation and shared libraries.





Author Addresses
=================================
```
Brad Campbell 
Computer Science	
241 Olsson Hall
P.O. Box 400336
Charlottesville, Virginia 22904 

email: Brad Campbell <bradjc@virginia.edu>
```
