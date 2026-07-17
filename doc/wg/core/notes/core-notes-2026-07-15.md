# Tock Meeting Notes 2026-07-15

## Attendees
- Branden Ghena
- Brad Campbell
- Pat Pannuto
- Leon Schuermann
- Amit Levy
- Alexandru Radovici


## Updates
### Using Duplicate Capsules and Userspace
 * Branden: I wanted to make people aware of a discussion: https://github.com/tock/tock/discussions/4922
 * Branden: Paul Otto wants to use multiple consoles on a RPi Pico board and we were discussing multiple ways to do this
 * Leon: I had an idea once upon a time of a registry where you could look up a driver and connect to it, rather than just having hardcoded numbers. I might add a pointer to that idea to the discussion
 * Branden: That would be great. The user also proposed a capsule which lets you choose between driver capsules via some handle, which is similar.
 * Branden: In kernel, you can always just make two of a capsule and attach them to different driver numbers. The annoyance is in userspace connecting to those. For libtock-rs, you could have drivers either have a const-generic or just a member variable that's the driver number. For libtock-c this is a pain in the butt and the easiest way is to probably copy the whole driver with a different driver number.
 * Branden: No action needed here, but I thought I'd mention it if anyone is interested.
### Treadmill
 * Leon: Working on a Treadmill rewrite. Taking lessons learned from last few years to try to make it more resilient and simpler.
 * Leon: Because of failures and rewrite, Treadmill is currently disabled as a CI check.
 * Leon: We might test this by just having people work with it for a bit before adding it as CI again.
 * Leon: Also really need to write some good tests. Even though the platform was somewhat functional, no one has been writing new tests yet. So that'll be the next big thing to tackle.


## Documenting Unsafe Code
* https://github.com/tock/tock/pull/4900
* Brad: As we try to have safety documentation everywhere we use unsafe, we need to decide how this should look. How much code should go in a given unsafe block? Should we combine nearby uses of unsafe or keep them as small separate blocks?
* Brad: I'm wondering if there's a Rust best-practices guide on Safety documentation for unsafe blocks. The Rust 2024 switch has made this more evident, as you need unsafe blocks even in unsafe functions. Clippy also has documentation requirements of documentation immediately above the unsafe keyword.
* Brad: Something hard is that sometimes you might miss that there are multiple unsafe interactions going on in a single line. So, I proposed a "single unsafe operation" per unsafe block. Maybe multiple copies of that, but just one unsafe API per block. But that's kind of clumsy, as adding offsets to pointers and dereferencing pointers are both unsafe, so they'd be separate, but they're logically on action.
* Amit: I thought of it as having a single unsafe block having a safe boundary. So for the example of dereferencing a pointer with an offset from a known-good pointer. Those two operations are probably coupled because there's only a safety guarantee on the joint operation, not the things individually.
* Leon: I agree in spirit, but the way this applies on a per-function level in Rust, with safe functions providing a sound interface doesn't quite apply the same way to unsafe blocks within a function. You can have unsafe blocks doing an operation, then safe blocks that still need to treat that result properly.
* Amit: It's possible, in an unsafe block, to take an offset that's out-of-bound, adjust it to be in bounds, and then dereference it. You could have one unsafe block that mints an invalid-if-dereferenced pointer, then another unsafe block to offset it, then another unsafe block to dereference it. But after the first block, you have a pointer that points to invalid memory.
* Leon: I think that's fine because they're raw pointers. I do agree with the gray area here. It's ultimately always the case that your unsafety isn't just the unsafe block, but also the inputs and outputs. If you have one unsafe block that mints a pointer and another that dereferences it, I think that's fine because you're not leaking a dereferenceable pointer into Rust.
* Leon: Generally, I think combining unsafe things would be okay, but adding lots of irrelevant code makes it less clear about which parts are unsafe and which are safe. 
* Pat: Here's the specific discussion: https://github.com/tock/tock/pull/4900#discussion_r3582313281 We have two unsafe blocks.
* Brad: Does anyone think add should be documented separately from reading?
* Pat: I think this case makes sense to combine, but I do agree that it's unclear when it becomes bigger where things ought to be separate. I do appreciate Brad's point that it's easy to miss "what" is actually unsafe. So having a safety comment that talks about ALL unsafe operations within a block seems good. Maybe our policy should be that the safety comment must handle each separate unsafe operation within a block.
* Amit: That seems reasonable to me. Maybe literally a list of operations that are unsafe and then invariants required for them. The nice thing that does is that it doesn't get you off the hook for documentation by combining things. So bigger unsafe blocks don't really mean less documentation, it's based on the operations you do. So disincentivizes the lazy version of big unsafe blocks.
* Leon: I think this would be a great thing to raise in the Rust zulip. I'd be happy to draft a thread there, as I assume people there have formed opinions. And maybe there should be documentation for the language at large
* Brad: What Pat and Amit said makes sense to me.
* Pat: I could make a template safety comment document, which would explain how to do this. In documentation somewhere.
* Brad: Yeah. I do think I got confused when writing safety documentation. Clippy has different requirements for functions and for unsafe blocks. Which is confusing. How do multiple-paragraph SAFETY blocks work? We end up having to follow the Clippy way even if it's annoying.
* Leon: At work, they decided inline comments aren't markdown, so they don't have a header. Where as `///` document comments are literally Markdown.
* Brad: So, Pat's documentation would have to think about that.
* Brad: I'd love a follow-on tool that figures out what things are unsafe like the Rust compiler, and could then check what needs to be documented. Making it visible. That tool could parse standard comment structure.
* Brad: Okay, I think this is helpful and we can move forward with this idea of documenting each operation within a block.


## RISC-V 64-bit
* Amit: 64-bit RISC-V PRs and TRD. Need to go through PRs for review and merging. I'm a bit overwhelmed with how to tackle this, so I'm looking for some hand-holding about what's going on, what's ready, what to look at.
* Brad: First three things going on. 1) Basic add 64-bit RISC-V support to Tock period. Arch crate and boards. https://github.com/tock/tock/pull/4873 2) PMP code change. https://github.com/tock/tock/pull/4874 3) PR that changes assembly to be register sized. https://github.com/tock/tock/pull/4875 That's all enough to get an application running
* Brad: Next is a set of PRs that handle Syscall stuff for 64-bit. We did some of this for CHERI, but didn't handle everything. So PRs to separate those into their own functions. https://github.com/tock/tock/pull/4907 and https://github.com/tock/tock/pull/4904 There will probably be a few more PRs on this one. QEMU 64-bit works because are only memory is in the lower 32-bits right now. Having dedicated helper functions for the TRD104 and TRD-RISC-V-64 versions help there a lot.
* Brad: Next, there's a branch which has all of these PRs combined with CI testing and maybe some other small changes I forgot about. After PRs get merged, then that branch can go last.
* Brad: So, what would be a way forward here. With the current PRs, we could merge them and have things be in a quasi-broken state for RISC-V 64 bit support until everything goes in. Or we could merge things into a dev branch, then do a giant PR from their into Tock.
* Amit: So there's code that works. There's a TRD that describes something different from the actual implementation. Something non-obvious is whether the TRD can be final without an accompanying implementation. So there's a big of chicken-and-egg thing. So we could just merge an implementation, that's partially wrong. Then update that. And formalize the TRD to match.
* Brad: I would advocate for having some code be out-of-sync, but then have one PR that updates the base implementation to match the TRD. It's a change of maybe 100 lines and is very clear what you're looking at. With the TRD itself.
* Brad: It does feel unsatisfying to merge somewhat unfinished code chunks right now. But combining these PRs together makes stuff really really massive.
* Amit: You're saying it would be massive if we immediately did the TRD104 implementation. (yes). Is that because TRD104 requires more changes than would be required for minimal 64-bit support?
* Brad: Yeah... I can't say confidently that even the minimal version would _really_ work if upper-32-bit addresses were used.
* Amit: Yeah. Something that works in a very particular use case right now, but isn't general or finished. It works, but is non-final. So we could have stuff now, then make updates to make things comprehensive later. And those comprehensive upgrades could be thought of as bug-fixes on the basic thing.
* Brad: #4873 that adds the chip and board is good, I think. It's the lower-level stuff that'll have diffs.
* Leon: This is independent of chip-crate support and PMP, right? You're proposing adding all of the non-PMP and chip into one big PR? The PMP stuff could stand alone.
* Brad: Yes. The PMP could stand alone. We could include everything in one big PR if we need to.
* Leon: For the PMP PR, I think it's good for now. Adds some infrastructure that's useful for all boards but only implemented for one. Maybe needs some thought.
* Brad: So the question is if there should be one big PR or not
* Amit: I'd say we should merge the first three #4873, #4874, #4875 on their own merits. Then build on top of it from there.
* Brad: That would be very helpful, but #4907 as well would really help. That would make a base for system call stuff on top. Then we could work on TRD-compliant system call stuff
* Amit: And #4907 is really just refactoring to move system call handling into helpers so we can easily separate out 32-bit and 64-bit versions.
* Amit: I agree with all of that. So looking through all of this: Johnathan had wanted to couple #4873 with a TRD originally.
* Brad: I don't want to speak for him, but my understanding was that he would be happy with a TRD in parallel.
* Pat: Sounds good to me
* Branden: I agree. It's okay to have this new platform support start off kind-of janky, then get better over time. We're not going to do a release in the middle of this anyways.
* Leon: Agreed
* Amit: Okay, I'll be looking at all of those PRs.

