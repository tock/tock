# Tock Cryptography WG Meeting Notes
- **Date:** October 31, 2025
- **Participants:**
    - Tyler Potyondy
    - Amit Levy
    - Kat Fox
- **Agenda:**
    1. Updates 
    2. State of Cryptography Stack/Interface in Tock
    3. Logistics 
- **References:**
    - [AES128 Crypto Stack Updates RFC](https://github.com/tock/tock/pull/4609)
    - [Hussain's Tockworld slides](https://world.tockos.org/tockworld8/slides/data-movement.pdf)

## Updates
- Tyler: For the tutorial we hosted in June, we looked into using AES GCM on the nrf52840dk. This however was broken and did not work. Kat confirmed that the AES GCM driver / functionality did not work on OpenTitan either.
- Tyler: In attempting to debug this, it became clear that the existing stack does not present a clean interface and involves many circular dependencies and nested clients that make it cumbersome to write drivers.
- Tyler: I have been working to overhaul the stack and introduce a generic crypto virtualizer. The current stack uses the AES128 HIL in a confusing way and has multiple higher level crypto modes (e.g. software GCM implementation
built upon CTR / GHash) implementing the HIL. 
- Tyler: Part of the redesign also involves simplifying the HIL to only represent hardware and introducing something, for lack of a better term, that is a "logic interface layer". This is a series of traits for each AES128 mode that captures the nuance of each mode (e.g. the nonce length required). 
- Tyler: Here is the link to the RFC: https://github.com/tock/tock/pull/4609

## State of Cryptography Stack/Interface in Tock
- Amit: At a high level, the current upstream crypto interfaces seem to make sense in a vacuum, but overall seem to strike the wrong balance of generality and efficiency and portability.
- Amit: It seems on all fronts, no one is happy with the existing stack.
- Amit: Maybe it is worthwhile for us to step back and think about what the right goal for these interfaces are.
- Amit: Hussain's suggestions seem mostly to deal with efficiency.
- Tyler: If I remember correctly, Hussain's suggestion was less about efficiency (in terms of performance) and more ergonomics and the challenge to maintain virtualizers over time.
- Kat: I am aware of Tyler's RFC PR, but is there one for Hussain's?
- Amit: No, just Tockworld slides: https://world.tockos.org/tockworld8/slides/data-movement.pdf
- Amit: It seems, a big problem is trying to design interfaces and a whole stack that is trying to match up clients and providers. E.g. a low level hash implementation with clients while applications may also just wish to encrypt and don't bother with the underlying details and wish to work across different boards. 
- Amit: On the other hand, a capsule may have a very specific hash implementation or requirements for the type of encryption. 
- Amit: All these things together seem at face value incompatible and now we have a suboptimal middle ground.
- Kat: For our use case, we need very specific implementations and they need to be standards compliant. 
- Kat: We also need fine tuned control.
- Amit: This feels very reminiscent to me of the Java cryptography api. You initialize you encrypt function call with a string and get an unrecoverable error at runtime if the crypto op is unsupported.
- Amit: It is unreasonable to anticipate that most platforms will provide accelerators.
- Tyler: Can't we just do this in the driver that handles the syscall and return an error to the application if the requested hardware accelerator is not there?
- Amit: This could get complicated on a platform like nrf52840 that has 2 AES accelerators (ECB and CCM). The CCM accelerator does far more than the ECB accelerator.
- Amit: Even drawing a boundary is a bit tricky because what I want is AES CTR or AES GCM but maybe I don't have this mode. 
- Kat: Opentitan does AES GCM and there is a bottleneck with doing some parts of this in SW. 
- Tyler: Amit, I'm a bit confused on your last point.
- Amit: Let's say we have an openthread app that we want to run on a variety of chips that have radios and some hw crypto accelerators.
- Amit: nrf52840 has ccm accelerator, so all of this should be deferred to the hw.
- Amit: Say we have another board which maybe only has a ctr accelerator, but not the ccm accelerator. 
- Amit: This would need to have more to ask for.
- Amit: What do you do? Do you make your app have extra code on the nrf52840 to support sw crypto on other boards even though it isn't needed?
- Tyler: It seems in this case, you should have different board files for the kernel and the same app.
- Amit: Yes, you could do this. Maybe there's a requirement in the TBF header specifying you need specific requirements to run this app.
- Amit: There's a similar question at the HIL or kernel level. This seems easier because we can statically enforce this.
- Amit: If there is an app that doesn't care and has higher level abstractions so that users don't pick a bad mode, the app doesn't care, it just wants encryption. 
- Amit: If you look at the symmetric encryption capsule it requires all modes to be implemented. This requires the board to ship with a lot of sw that may never be used.
- Tyler: The more we talk about this, it seems that SW crypto that builds upon a HW accelerator should just be handled in userspace.
- Amit: Yes, I agree.
- Kat: This seems like it should be in a shared library.
- Amit: It is maybe the case that if this is in a shared library, things might be pulled in on demand, and this library could be board specific. This may be a reasonable way to have something that both allows the applications to be portable across boards and for the board to not be overly bloated.
- Amit: The userspace shared library gets to be board specific and be specialized. 
- Kat: I like this idea. My other concern is having the nice security guarantees of doing this in a capsule vs an application. I do understand that having sw crypto live below syscalls in the kernel results in bloat, but this comes with nice security benefits.
- Kat: Perhaps this can be used with something like the OxidOS configuration manager.
- Amit: From a protection point of view, for OpenTitan and Pluton, and in the static case what gets shipped is a set of applications and a kernel. This shared library could be something that the code is part of the kernel and is specialized to the board. When this shared library runs, it runs in userspace.
- Amit: So this shared library is just code and any memory lives within the application. This then can ship with the kernel image.
- Tyler: I like this idea. We should continue to discuss this.

## Logistics
- Tyler: Let's plan to hold this meeting every other Friday from 10-10:45am PST. 
- Amit/Kat: Sounds good.
