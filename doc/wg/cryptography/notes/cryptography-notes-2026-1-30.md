# Tock Cryptography WG Meeting Notes

**Date:** 2025-12-12

**Participants:**
  - Tyler Potyondy
  - Bobby Reynolds
  - Kat Fox
  - Amit Levy

## Summary

The working group reached consensus on exploring a major architectural
pivot: moving software cryptography out of the kernel and into
**userspace shared libraries**. This addresses the current complexity
and circularity of kernel HILs while maintaining application
portability. The group will now conduct a comparative code review of
AES implementations across Pluton, OpenThread, and OpenTitan to inform
the new HIL design.

## Action Items
| Task | Owner | Status |
| :--- | :--- | :--- |
| Prepare AES (GCM/CCM) code/constraints for comparative review | **@reynoldsbd** (Pluton) | New |
| Prepare AES (GCM/CCM) code/constraints for comparative review | **@tyler-potyondy** (OpenThread) | New |
| Prepare AES (GCM/CCM) code/constraints for comparative review | **@pqcfox** (OpenTitan) | New |

## 1. Cryptographic HIL Redesign Strategy

### Context

The current AES GCM implementation is broken, revealing a broader
issue: Tock’s kernel crypto stacks have become overly complex and
circular. Currently, HILs try to balance hardware acceleration with
software fallbacks in the same layer, leading to kernel bloat and
maintenance difficulty.

### Decision

**Shift software cryptography fallbacks to userspace shared libraries.**

* The kernel will focus strictly on exposing hardware-backed services.
* Kernel will not be as concerned with portable hardware interfaces
* Application portability will be handled by a userspace layer that
  connects to hardware accelerators when available and provides
  software implementations when not.

### Rationale

* **Efficiency:** Avoids carrying unused software crypto "baggage" in
  the kernel.
* **Portability:** Applications can remain agnostic of the underlying
  hardware by linking against a consistent userspace library.
* **Safety/IPC:** Shared libraries provide a middle ground between
  bloat and the high overhead of IPC-based crypto services (a pain
  point noted by Pluton).

### Constraints & Challenges

* **Opaque Hardware:** Hardware that manages keys or padding
  internally (opaque to firmware) requires HILs to adapt to it.
* **Certification:** Kernel interface changes must be careful not to
  invalidate side-channel protections or existing certifications.
* **Code Size:** Moving crypto to userspace doesn't eliminate the
  software footprint, but a shared library could allow multiple apps
  to share that cost.

## 2. Comparative Analysis: AES GCM/CCM

### Context

The group needs a representative problem set to test the new HIL
design across different hardware paradigms (Pluton, OpenThread, and
OpenTitan).

### Decision

The group will focus its first deep dive on AES (GCM/CCM) modes.

### Rationale

AES shows the most significant variation in hardware support between
chips. By comparing how Pluton, OpenThread, and OpenTitan handle AES,
the group can design an HIL that captures varied constraints (like
opaque keys or specific padding schemes) without over-engineering the
interface.

***

# Transcript

## Recap/Overview
- Amit: AES GCM doesn't currently work. One of the takeaways from Tyler's suggestion and trying to debug that is that the kernel crypto stacks are overly complex and circular.
- Amit: We should fix the broken pieces in GCM, but also address this broader issue of kernel HILs trying to do too much in the same layer.

## Cryptographic HIL Redesign
- Amit: There are two major problems with the existing interfaces. In practice, almost no hardware provides accelerators for all crypto modes. A lot provide some. We want to be able to use this when it is available, but also do not want to carry the baggage for the cases when sw crypto is not needed (bloat, security, etc).
- Amit: We had been discussing what might be the correct approach moving forward.
- Amit: On the one hand, we want to avoid unused sw crypto in the kernel. On the other, we want apps to remain portable.
- Amit: The "next step" we came to was to try supporting shipping shared libraries. This would be the layer that connects specializing the kernel to what hardware provides. The userspace layer (which could be shared across applications) would provide whatever software applications require (software crypto).
- Amit: I suggest, as Bobby had mentioned, we take a look at the crypto code used in Pluton. Comparing this usecase to maybe OpenThread and seeing how this might run into issues with HIL designs.
- Bobby: I have some code set aside that we can begin looking through together.
- Bobby: A few constraints from our end: Having access to a shared library is very interesting for our usecases. Not having this has resulted in us using IPC more than we would like to.
- Bobby: It is a good goal to have software crypto with increased portability in apps. However, if the hardware provides a specific crypto service, there may be some issues. For instance, RSA/PKCS1.5 padding scheme---you could imagine hardware that provides low-level operations (upon which you build higher abstractions). Other hardware perhaps does all of this. For performance/sidechannels etc, it might be preferable to use the hardware rather than the library.
- Bobby: An even harder case is when hardware manages keys or sensitive information in a way that is opaque to firmware.
- Bobby: The only way to take advantage of the hardware is to play by its rules so to speak.
- Bobby: I agree that we do not want libraries like openssl in the kernel. In the case when this cannot be done in hardware, it should be in userspace.
- Amit: Many applications want "encrypt this stream of bytes".
- Amit: If the hardware provides the needed encryption, then the kernel should expose that. If it doesn't, ideally, the same application should be able to be used with a software implementation of what is missing.
- Amit: The easy thing to support for this are applications that are not intended to be portable.
- Bobby: For us, we want consistency. For instance, we have libpluton-c which has our syscall bindings. When we build apps against this, we have C crypto headers. This is pluggable at build time.
- Amit: The one thing enabling a shared library approach allows is not having to build applications themselves separately.
- Amit: In the openthread case, there are many boards with a 15.4 radio that are similar except each chip has particularities for which crypto accelerators on the chip.
- Tyler: We currently do not use hardware cryptography with openthread. We pull in a sw crypto library.
- Tyler: Code size is the biggest downside to this. It adds ~30kB to our app's binary size.
- Kat: We have similar issues with crpytolib in OpenTitan. With any HIL rewrites or doing this might result in running into certification issues and may be challenging to encapsulate.
- Kat: Timing sidechannels and managing cache can become tricky if we have this functionality managing the HIL vs using cryptolib.
- Tyler: I propose we move forward by sharing/looking at some concrete examples and code that currently exists.
- Tyler: There likely is not a "perfect" solution to our HIL redesign so it will have to make some tradeoffs.
- Tyler: Between OpenThread/OpenTitan/Pluton we have a varied enough problem/constraint set should hopefully be representative.
- Bobby: I have code I can share. Is there a specific family of encryption we want?
- Bobby: I have some AES code ready, but can find others.
- Amit: I vote AES.
- Tyler: Me as well.
- Amit: AES seems to have a lot of variation between chips.
- Kat: This is a core one for us.
- Amit: Let's plan to each prepare some code to show, focusing on AES GCM/CCM style usecases.
