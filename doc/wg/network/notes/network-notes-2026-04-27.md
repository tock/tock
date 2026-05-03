# Tock Network WG Meeting Notes

- **Date:** April 27, 2026
- **Participants:**
    - Branden Ghena
    - Leon Schuermann
    - Vishwajith Govinda Rajan
- **Agenda:**
    1. Updates
    2. Core WG ProcessID Discussion
- **References:**
    - [Core WG Call Notes](https://github.com/tock/tock/pull/4791)
    - [ProcessID PR](https://github.com/tock/tock/pull/4777)


## Updates
 - Vish: We've been working on reverse Syscall thing where userland apps can service certain system calls. That'll be presented at some point by Marshall from UVA. Likely on a Core call.


## Core WG ProcessID Discussion
 * Notes here: https://github.com/tock/tock/pull/4791
 * Branden: There was a misconception about whether the handle was public or private; once it became clear that it was opaque, nobody really cared any more.
 * Branden: Second, two possible implementations that came up. One with 64-bit ProcessID or one with 32-bit ProcessID and 32-bit ShortID combined.
 * Leon: I'm not yet convinced that we should close the 64-bit ProcessID PR. https://github.com/tock/tock/pull/4777 In the kernel, unrelated to IPC we may still want this.
 * Leon: But for IPC, we should 64-bit handles. Then we could even zero-pad ProcessID for now if we want.
 * Branden: Add to the queue of IPC work---make sure that 64 bit handles don't wreck anything.
 * Leon: Clarify this now while we're building the capsules.
 * Leon: I was also considering if it's problematic to split this integer. On a 32-bit platform, how should we split it when communicating it to userspace. We need to split consistently Kernel-to-Process and Process-to-Kernel
 * Branden: We can already return 64-bit values to userspace. Whatever return does, arguments for capsules should.
 * Leon: Agreed.
 * Leon: For u64 ProcessID, kernel things that use it could still be hit by confused deputy attacks. Upcall scheduling for instance. So there are concerns
 * Branden: But more niche
 * Leon: And no soundness issue. Less pressing.
 * Branden: And possibly less worth the amount of bytes this costs
 * Leon: Well if most things use IPC and IPC needs a u64 of some type, we might pay that cost anyways.


## Future Network WG Meeting Times
 * Branden: May 11 and May 25 are both unavailable for me (May 11 is a workshop, May 25 is US Memorial Day holiday). We could meet ad-hoc on May 4 or May 18 if anyone has agenda.


