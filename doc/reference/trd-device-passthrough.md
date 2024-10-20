Device Passthrough Support
========================================

**TRD:** 107 <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft<br/>
**Author:** Alistair Francis <br/>
**Draft-Created:** September 9, 2024<br/>
**Draft-Modified:** September 9, 2024<br/>
**Draft-Version:** 1<br/>
**Draft-Discuss:** tock-dev@googlegroups.com</br>

Abstract
-------------------------------


1 Introduction
===============================
The Tock syscall interface has lots of useful advantages. It allows
virtulisation, platform agnostic user space applications as well being
secure. But this comes at a cost. The syscall interface requires lots of
buffer copying and context switching. This impacts performance and going
through the Tock syscall interface is a magnitude of order slower then direct
hardware access.

For most use cases this doesn't matter and the advantages outweigh the
performance impacts. The general exception to this is BLE, where the syscall
overhead is too large to meet the strict BLE timing.

2 Design
===============================
In the case of Tock we would assign a specific device to a single userspace
application.

The high level flow is:

 1. An application will perform a syscall via the `PassThrough` capsule
    to request access to a device.
 1. The capsule will then forward that request to the kernel.
 1. The kernel will check with the board if the request is allowed, using
    the `DevicePassthroughFilter` trait.
 1. If approved by the board, the kernel will allocate an MPU region.
 1. If everything succeds the kernel will report a success to the application
    and the application will be able to read/write to the device memory.
 1. Devices that support passthrough and interrupts will need to implement
    the `PassThroughDevice` device trait to report interrupts to the
    `PassThrough` and then to userspace.

3 Safety
===============================
In server class systems we use a IOMMU to avoid security issues from the
passthrough device accessing memory it shouldn't.

Tock targets obviously don't support IOMMUs, so we are left with a lack of
security.

This means that once an application has access to device memory, all security
bets are off. Even a well analysed device is going to be able to circumvent
a lot of Tock's security protections.

As such, the feature is up to a board to opt-in to. This way a board (and the
person writing the code) can make a decision based on their use case and their
thread model.

Note that alothough this blows a hole in Tock's security, it should provide more
protection then running blobs in the kernel, which is generally the only other
option for features like BLE.

There is hope that future hardware (like the RISC-V IOPMP) can mitigate some of
the security concerns.

4 Author's Address
===============================
```
Alistair Francis
Brisbane
Australia
```
