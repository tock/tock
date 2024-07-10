Tock on Platforms Without Execute-in-Place (XIP) Flash
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Informational<br/>
**Status:** Draft <br/>
**Author:** Brad Campbell<br/>
**Draft-Created:** 2024/07/05<br/>
**Draft-Modified:** 2024/07/05<br/>
**Draft-Version:** 1<br/>
**Draft-Discuss:** devel@lists.tockos.org<br/>

Abstract
-------------------------------

This document provides information relating to running Tock on platforms without
execute-in-place flash storage.

Introduction
===============================

Traditional embedded systems use execute-in-place (XIP) flash to both store
program code and to enable executing that program code. However, newer hardware
designs, particularly those integrated into a system-on-chip alongside an application processor, do not include 
executable persistent storage. These platforms, subsequently referred to as "non-xip", can only
execute instructions stored in volatile RAM. All executable code must be stored
in external nonvolatile storage, loaded into RAM, and then executed.

This presents challenges for executing Tock on such platforms as Tock generally
expects to not only have XIP flash, but enough flash to support the overhead of
discrete applications and their headers. Concretely, Tock platforms generally
have at least 512 kB of flash. Supporting non-xip platforms may require new
interfaces or introduce design considerations that Tock on traditional platforms
would otherwise not consider.

1.0 Background
===============================

A non-xip platform has no persistent flash storage where compiled binaries can
be stored and executed. All static program code and data must be stored in some
nonvolatile storage and then be loaded into RAM to execute.

This system architecture shrinks the amount of the hardware that must be trusted
when securely executing programs. By not including execute-in-place flash, the
flash storage does not need to be trusted to correctly store program code and
data. When the program is copied to RAM the program binary can be verified
(i.e., by checking a cryptographic signature) before the loaded binary is
allowed to execute.



2.0 Tradeoffs
===============================

Non-xip platforms introduce a new design point for embedded devices with a
different set of tradeoffs compared to traditional platforms. To help motivate
potential Tock designs for non-xip platforms, we discuss both the limitations
compared to traditional platforms, and the properties that may be important on
traditional platforms but are likely not a priority on non-xip platforms.

Limitations:

- Non-xip platforms have less RAM that traditional platforms have XIP flash.
  Further, this RAM on non-xip platforms must (at runtime) store both the
  executable code and the conventional memory items (e.g., the stack and heap).
  This results in significantly less room for program code than on traditional
  platforms.
- As flash storage is not trusted, any data retrieved from flash storage must be
  verified before it is used.

Non-limitations:

- Energy is not a bottleneck, and additional power draw from having to load
  program code into RAM is acceptable.
- Execution time has relaxed deadlines, and additional execution time from
  having to load program code into RAM is acceptable.






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
