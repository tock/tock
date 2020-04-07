Application Loader
==================

## What is an Application Loader?

The term "application loader" refers to the mechanism used to add Tock
applications to a Tock system. It can take several forms; here are a few
examples:

1. [Tockloader](https://www.github.com/tock/tockloader) is an application loader
   that runs on a host system. It uses various host-to-board interfaces (e.g.
   JTAG, UART bootloader, etc) to manipulate application binaries on the Tock
   system's nonvolatile storage.

1. Some build systems combine the kernel and apps at build time into a single,
   monolithic image. This monolithic image is then deployed using a programming
   tool.

1. A kernel-assisted installer may be a Tock capsule that receives application
   binaries over USB and writes them into flash.

## Why Must We Trust It?

The application loader has the ability to read and modify application binaries.
As a result, the application loader must be trusted to provide confidentiality
and sometimes integrity guarantees to applications. For example, the application
loader must not modify or exfiltrate applications other than the application(s)
it was asked to operate on.

Tock kernels that require all application binaries to be signed do not need to
trust the application loader for application integrity, as that is done by
validating the signature instead. Tock kernels that do not require signed
application binaries must trust the application loader to not maliciously modify
applications.

To protect the kernel's confidentiality, integrity, and availability the
application loader must not modify, erase, or exfiltrate kernel data. On most
boards, the application loader must be trusted to not modify, erase, or
exfiltrate kernel data. However, Tock boards may use other mechanisms to protect
the kernel without trusting the application loader. For example, a board with
access-control hardware between its flash storage and the application loader may
use that hardware to protect the kernel's data without trusting the application
loader.

## Tock Binary Format (TBF) Total Size Verification Requirement

The application loader is required to confirm that the TBF header's
`total_size` field is correct for the specified format version (as specified in
the [Tock Binary Format](../TockBinaryFormat.md#tbf-header-base)) before
deploying an application binary. This is to prevent the newly-deployed
application from executing the following attacks:

1. Specifying a too-large `total_size` that includes the subsequent
   application(s) binary, allowing the malicious application to read the binary
   (impacting confidentiality).

1. Specifying a too-small `total_size` and making the kernel parse the end of
   its image as the subsequent application binary's TBF headers (impacting
   integrity).

## Trusted Compute Base in the Application Loader

The application loader may be broken into multiple pieces, only some of which
need to be trusted. The resulting threat model depends on the form the
application loader takes. For example:

1. Tockloader has the access it needs to directly delete, corrupt, and
   exfiltrate the kernel. As a result, Tockloader must be trusted for Tock's
   confidentiality, integrity, and availability guarantees.

1. A build system that combines apps into a single image must be trusted to
   correctly compile and merge the apps and kernel. The build system must be
   trusted to provide confidentiality, integrity, and availability guarantees.
   The firmware deployment mechanism must be trusted for confidentiality and
   availability guarantees. If the resulting image is signed (and the signature
   verified by a bootloader), then the firmware deployment mechanism need not be
   trusted for integrity. If there is no signature verification in the
   bootloader then the firmware deployment mechanism must be trusted for
   integrity as well.

1. An application loader that performs the nonvolatile storage write from within
   Tock's kernel may make its confidentiality, integrity, and availability
   guarantees in the Tock kernel. Such a loader would need to perform the
   `total_size` field verification within the kernel. In that case, the kernel
   code is the only code that needs to be trusted, even if there are other
   components to the application loader (such as a host binary that transmits
   the application over USB).
