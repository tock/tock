# Core Working Group Meeting, April 15th 2022

## Attendees
 * Branden Ghena
 * Leon Schuermann
 * Phil Levis
 * Pat Pannuto
 * Hudson Ayers
 * Alexandru Radovici
 * Johnathan Van Why
 * Vadmim Sukhomlinov
 * Alyssa Haroldsen


## Updates
 * Alyssa: I'm going to be a semi-official liason between TI50 and Tock.
 * Phil: It might be good to start planning the next Tock world. An in-person meeting would be great to do. I don't know when a good time is, but it would be very good to organize another in-person Tock world. Maybe end-of-summer. I'll bug Amit about it.
 * Hudson: It would be great to do another one!
 * Phil: Hudson and I are trying to push forward the UART TRD and a redesign of the UART API. We've got some undergrads learning Tock at Stanford who are learning Tock and porting the SAM4L and nRF52. We'll see how that goes and then expand efforts to other chips.


## SHA Draft PR
 * https://github.com/tock/tock/pull/3010
 * Phil: The draft software SHA implementation PR is up. Part of the AppID effort so we have some way of checking integrity. I thought it would make sense to have a software implementation so it works across platforms with various hardware. SHA-256 is the implementation. There's a few edge cases, but I have it working. I'll transistion from draft to real PR as I clean up the code.
 * Phil: Generally, I think it's nice to have software-only implementations of some things so they are really cross-platform.
 * Alyssa: Are there any pre-certified libraries we could use instead of rolling our own?
 * Phil: Yeah, I don't want to do this for anything that involves secrecy, for that reason. AES for instance, is a bad idea to implement ourselves. For integrity stuff though, this seemed reasonable. Where this gets tricky is RSA. For SHA I felt comfortable because there are simple test cases and it's just an integrity question. Public key or really any secrecy questions, I would not feel comfortable implementing.
 * Phil: Notably, this is a super brain-dead implementation. It's not fast or optimized. It's just nice for platforms with no hardware support at all.
 * Leon: Interesting argument. This might be nice for CI use cases too, where there is no hardware support for stuff. I think it might be hard to find existing code for other things because Tock handles buffers differently. I do actually have an AES implementation lying around, but I'm not sure where the line is where Tock would accept something. Say that we know it has side channels, but it's nice for testing.
 * Phil: I'd say anything in the main repository shouldn't be rolled on our own. Even if it's only for testing, someone will use it for real just because it's there.
 * Alyssa: I'd also like to mention that there needs to be a stable interface for SHA. For platforms with their own crypto hardware, we want to switch out for that.
 * Phil: Absolutely. This is just a capsule that meets the Digest interface.
 * Phil: The current plan is to use this for AppID stuff to check integrity of app images.


## Tock Registers Alignment
 * Branden: I wanted to check if anyone saw: https://github.com/tock/tock/issues/3019 Looked scary.
 * Leon: I looked at this briefly and don't have enough time to look in-depth for now. We do make assumptions about the register structs and that they're matching the repr(c) rules. I think our logic doesn't take alignment into account right now.
 * Leon: I think this issue might not be one that we can really resolve. We only know at run-time what address is being loaded at.
 * Branden: I'll follow up on the issue. I'm also confused _how_ they ended up with an unaligned address anyways.


## Tock Registers Soundness
 * Alyssa: I'm also concerned about references to volatile memory in Tock registers.
 * Leon: Yeah, we did look into this. We believed that our current interface is valid as-is. There was a discussion here: https://github.com/tock/tock/issues/2593
 * Pat: Definitely take a look at that. And if you have real concerns still, please let us know. We're planning to release a 1.0 at some point relatively too.
 * Alyssa: Mostly, I think instead of individual field access, there should be accessors which return wrappers around a raw pointer. So there's never a reference to volatile memory and it remains a raw pointer the whole time?
 * Pat: Does that add an extra function call to each access at runtime?
 * Alyssa: No.
 * Leon: How does that work? Right now we use a struct that overlays memory and is the exact same size. This discussion sounds very much like what we talked about for userspace memory sharing. Fundamentally, this is the same issue, right?
 * Alyssa: You'd have your outer structure be a wrapper around a raw pointer. And you'd have functions that get individual fields within the struct.
 * Leon: I think I'd need a written-form version of this to get to think through it. It gets pretty complicated to think about it.
 * Alyssa: The concern is that LLVM can insert a read at any point if it sees a pointer that's dereferencable. Which is problematic with volatile memory.
 * Pat: So you think there could be spurious reads?
 * Alyssa: It's _possible_. Why would LLVM ever do this? I think it's unlikely. But it is a semantic correctness thing.
 * Pat: This is something that messed with C/C++ compilers for a while. I think there's almost no way to correctly represent memory-mapped I/O per the letter of the law. LLVM doesn't have a representation for it.
 * Alyssa: C represents volatile as a property of memory, not of an access. LLVM and Rust treat it as a propery of an access. But if you make a pointer to a volatile int. LLVM doesn't know for certain that it can dereference this pointer.
 * Johnathan: I think that's true in all cases for C/C++. But for Rust all references are dereferencable.
 * Alyssa: I thought C++ was. Maybe there's something tricky going on.
 * Leon: So you think the boiled down problem is that volatile cell as a concept is incorrect.
 * Alyssa: Yeah.
 * Leon: So I don't know the solution for laying out this struct on memory where there are arbitrary offsets into it.
 * Alyssa: Maybe I don't know enough about the Tock register interface. How do you access things dynamically?
 * Leon: I think it's all compile-time. It is arbitrary but constrained by byte-aligned at compile time.
 * Alyssa: Right now we're doing a repr(c) struct to do mmio offsets. Instead of exposing publicly accessible fields, you expose methods and the methods take a pointer to the wrapper struct. Each one of these methods take a pointer and offset it by a known static value generated by the macro and return the offset pointer.
 * Pat: So you don't need to change the interface at all, right?
 * Alyssa: Well, instead of fields you need methods.
 * Leon: I think the traits we have that provide methods allow us to change the representation underneath. So defined offsets with constant methods to determine offsets. Then we would return different types with the same interfaces as today. It would use the container pointer values of these types to return the appropriate data.
 * Leon: So we should change the way the registers are defined in the first place and the wrapper underneath.
 * Pat: I _think_ we can change this without changing anything for users of the library.
 * Alyssa: If you're accessing fields, you'll need to change. So calling the macro doesn't necessarily have to change. Just the using of the output of it.
 * Pat: I'm nervous because there's a lot of work that went into this and a huge volume of code that's using it. So I want a concrete statement that this is necessary because of a real functional issue and that the change is sanctioned and correct and won't have to change again.
 * Alyssa: Is there an issue open for this? The volatile cell issues?
 * Pat: #2593 mentions some things, but you'll need to open a new one.
 * Leon: This is separate from that issue. No issue exists for this yet.
 * Hudson: So this is the same problem that exists for the volatile registers crate and various other MMIO crates. It seems likely that if LLVM started inserting reads, a lot of stuff would start to break, not just Tock registers.
 * Johnathan: There's this one too: https://github.com/japaric/vcell/issues/10 I do recall the embedded working group making changes because of this concern.
 * Alyssa: There are also some minor soundness issues. You can create a register field that's just an exposed u8 buffer, which isn't great because it should be in an unsafe cell. It shouldn't be possible to just put a block of memory there. So some way to present a slice in mmio is necessary.
 * Leon: I think I agree. Though technically, the unsafe part there happens when you transmute the pointer to a reference of this struct. So it's more of a user mistake. So that's sort-of outside of the scope of what Tock registers defines. But we do it all the time so we should think about it.
 * Alyssa: We should probably have a constructor function that makes this instead of people converting themselves.
 * Leon: If we are going to have to change the interface, this sounds like a good idea. We first really need to document these issues so we can seriously consider them.
 * Alyssa: I can send a simple playground of what it might look like.
 * Leon: That would be great.

