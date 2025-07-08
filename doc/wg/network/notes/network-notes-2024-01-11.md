# Tock Network WG Meeting Notes

- **Date:** January 11, 2024
- **Participants:**
    - Branden Ghena
    - Leon Schuermann
    - Tyler Potyondy
    - Alex Radovici
    - Felix Mada
- **Agenda**
    1. Updates
    2. Pick New Meeting Time
    3. Buffer Management Next Steps
    4. Thread Tutorial
- **References:**
    - [Thread Tutorial Planning](https://github.com/tock/book/pull/26)


## Updates
- Alex: Trying to get a student to use Leon's buffer management proposal to build a better console for Tock, enabling several applications to read and write at the same time. This way multiple apps can be served on the console at the same time.
- Branden: is this related to the proposal where we add a header to each message to tell which process it's from?
- Alex: exactly.
- Branden: would be great if we could also read these messages in any generic serial terminal.
- Alex: Problem -- partial messages. For now, we'll try something and see where it goes
- Branden: For sure
- Alex: Also, this is a good example for using the Buffer Management stuff
- Alex: Eventually, I'd like to see support for this merged into tockloader-rs (on a branch)

## Pick New Meeting Time
- Branden: Sorry, for needing to move things!
- Branden: Let's move to Mondays at 10am Central. I'll move the calendar invite

## Buffer Management Next Steps
- Leon: Exciting to hear that Alex has a student who can possibly drive this forward
- Leon: Our position from last meeting is that we have a reasonably solid understanding of a set of types that seem to be sufficient to cover some of our use cases and compile and are sound. There are no unsafe usages or transmute calls. Things are still not perfect, but it's getting there.
- Branden: So types, but not functionality right now?
- Leon: We do pass these types between layers and can even pre-pend some bytes to a buffer. But there's not anything reasonably close to a real-world use case implemented with them yet
- Alex: One question is where we add this to Tock
- Leon: Before that, I'd like to see this working in some subsystem before adding it more generally. It is just a single Rust module. It could be added as a kernel utility
- Branden: I agree with that
- Alex: So we could do that. Then we'll use it in the capsule and the chip-level UART driver
- Leon: I am confident that we can get our base use cases working with what's here. Something I'm still working on are layers that use packets that might have layers that use contiguous buffers and layers that use non-contiguous buffers. The hope here was that there could be specializations of the general implementation for each case. Pre-pending data and passing between layers when entirely contiguous or entirely non-contiguous works in my examples. Or compiles at least.
- Branden: When would we need to change between the two? For example, if we had a Thread stack where in it might we change from one type to another?
- Leon: Requiring non-contiguous buffers would be easier. So we might have an IP implementation that uses it. But it wants to support hardware in either case. So the upper level only passes one type of buffer, but then both cases should be supported when we hit hardware.
- Leon: We could fail back to only having one type of buffer. Probably contiguous because it would support all use cases
- Branden: Do we only need to translate when hitting hardware? Or higher in the stack?
- Leon: In your network stack if you have a layer somewhere that pre-pends an entirely separate non-contiguous buffer, at this point you're unable to support hardware that requires contiguous buffers.
- Branden: Sort of.
- Leon: You could copy. But it seems bad to have devices that can do DMA but we still need to copy
- Branden: Is the goal to do DMA right from the buffer management mechanism?
- Leon: We do want to do DMA directly from these buffers. We could always copy before DMA as a fallback to support any type.
- Branden: So going back, what do we do next?
- Leon: More focused time to finish/polish the implementation. And try to use it in some use case that resembles a network stack.
- Branden: So Alex and students are working on part two of that, what goes into part one?
- Leon: Thinking about functionality that the Buffer Management supports. APIs for writing data, reading data, etc.
- Branden: Should we try brainstorming APIs on this call?
- Leon: Or by just using it? Then we can clean it up afterwards
- Alex: Yeah, that's not a problem. We can add APIs as we go. We do also need documentation though. Particularly how to use things and why certain things exist.
- Leon: I'm not sure how useful rust-doc style comments will be. There could be a more lengthy blog post style about the design.
- Alex: Possibly a TRD
- Leon: Possibly. Not sure we need the formalism just yet
- Alex: Even rust-doc comments to start should be enough. Acronyms and words that have particular meaning that wouldn't be clear to a user
- Alex: How should we use this packetbuffer as-is?
- Branden: Could just copy-paste
- Alex: Okay, so we'll work on a branch with a copy of this. Then we'll work from there. Can Leon make code changes as a PR to that branch if/when necessary?
- Leon: Yes, no problem. We could also jump on a call with your student to start using and implementing this.
- Branden: I think the plan here would be to demonstrate it and work on it in your console example. Then we'll end up making a PR to tock that only adds Buffer Management stuff. Then later we'll do a PR for your console stuff using the Buffer Management stuff. This will slow down overall PR for console changes, as long as you're okay with that
- Alex: That sounds good. My overall goal is to bring Buffer Management stuff into TockWorld in June
- Branden: Agreed
- Alex: I am very pleased with where this has gone so far
- Branden: I do think the APIs will be a lot of work, but at least it's not magic type-system work
- Leon: Agreed. I was always worried that the type system stuff wouldn't work at all


## Thread Tutorial
- https://github.com/tock/book/pull/26
- Tyler: We are about 100% committed to the tutorial at CPS week. We'll confirm in the next few weeks or so for sure.
- Tyler: For today, it would be good for everyone to look at the Tock Book PR about this and provide comments.
- Branden: From a high level, the goal is to 1) learn to use Tock and load applications, 2) write an application to act as a Thread child and connect to an existing network, and 3) do some application while connected to transfers data across the network and demonstrates reliability
- Tyler: One other comment is that Leon and I have been working on getting the OpenThread build system to work as a library for Tock. We've been meeting somewhat regularly and making good progress so far. Hopefully soon we'll have OpenThread working in Tock.
- Leon: I've really just been the rubber duck here having Tyler bouncing ideas off of me
- Alex: How do you deal with Tock asynchrony? OpenThread is synchronous, right?
- Leon: With processes asynchrony wouldn't be an issue. That's true for encapsulated functions too. They're sort of a hybrid of a kernel and userspace thing. The key that I've seen from OpenThread is that it doesn't require us to run one single blocking function. You can call individual OpenThread functions on reception of packets, so we're only ever making a call for as long as it takes to process a single packet. So really we'd be using OpenThread with Tyler's 15.4 implementation.
- Branden: OpenThread has to send too, right? Not just process incoming packets
- Leon: You send on users wanting to send packets, or on timer ticks. So those are events for triggering it.
- Alex: But when you send a packet, you need to know if it succeeds or fails, and that's asynchronous in Tock.
- Leon: OpenThread invokes a callback which is responsible for handling packets. So in the callback, we'd pause execution, switch to the kernel, do the work, and not switch back until we have a result
- Alex: Okay, that's exactly what I was interested in. You can stop it and resume it later. That's great. So this is going to exist publicly soon and open-source?
- Leon: Yes. Soon. And it will be entirely open-source
- Alex: Can you point me to the code? OxidOS has a huge interest in this
- Leon: For specifically OpenThread, I talked to some people about whether it would be reasonable to execute OpenThread in this hybrid mode. The answer is that we'll find out as we go. We're hoping to find some way to get some basic stuff working with the least effort. And if that happens to be a userspace application, we'll do that. The reason why I think my system could be promising is that it doesn't come with as much cruft as building userspace stuff does with the libtock-c build system. And it might just be easier to call functions rather than have a lot of callbacks and allow system calls. So we are trying my system, but will fall back on a userspace implementation
- Tyler: Right. There is a huge asterisk still on whether the system will entirely work. We believe it will work, but aren't entirely sure yet.
- Leon: Currently the exploration into the CMake build system will benefit both of these approaches at once. We just need some ELF file, whether it runs in the kernel or userspace.
- Tyler: A question: is it possible to invoke CMake within the Makefile? So we'd have some third-part directory and we could invoke it.
- Branden: I don't know if there's a hook to build an external library. Existing stuff expects to be pre-built and just grab the ELF
- Leon: There is actually a hook for an external Makefile for your library! You can use it, but only if it exports the right Make variables. So I'm worried about integrating CMake into a Make build system. This is kind of exotic.
- Tyler: CMake essentially creates a bunch of makefiles, though right?
- Leon: That is accurate
- Tyler: One thought I was having is what if we invoke CMake on OpenThread. Then we use the Make system to call those Makefiles?
- Leon: I really think that CMake just uses Make as effectively an executor as a few build steps. I really don't think that the Makefiles generated are suitable for being imported by anything else. Ultimately I think for now that what Branden suggested is reasonable. If we can get an ELF file somehow, we can plug it in somehow. The later would could think about how to integrate this upstream.
- Tyler: So that ELF file that you're talking about, we would need to specify for CMake to create an ELF file with the right compiler arguments?
- Leon: I think so? But I'm anxious about the compiler arguments. We could run in verbose mode to see compiler arguments from libtock-c
- Tyler: We'll have to think about how to "overwrite" the OpenThread compiler arguments with our own
- Leon: Plus the scary thing is that OpenThread itself has libraries like mbed-tls. So we have to compile those with the right arguments too
- Tyler: We could have a meeting with someone about the libtock-c build system
- Branden: The knowledge is split across myself, Brad, and Pat. I think Brad touched it most recently, but I was the one who originally added the double-dollar-sign stuff to it
- Leon: What is the double-dollar-sign stuff?
- Branden: It's multiple replacement steps for the variables. First we do one replacement, then later we do a second round of replacement based on that first one. Essentially, it's creating sets of rules for specific architectures
- Leon: Oh, it's meta-programming
- Tyler: Another approach was to try to make a new build system for OpenThread, but I hit a dead end there and moved to this new approach.
-
