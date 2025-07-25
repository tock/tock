# Tock Network WG Meeting Notes

- **Date:** March 24, 2025
- **Participants:**
    - Alex Radovici
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
- **Agenda:**
    1. Updates
    2. Ethernet
    3. Strategy Workshop
- **References:**
    - [RP2040 PIO SPI](https://github.com/tock/tock/pull/4378)
    - [Libtock-c LWIP](https://github.com/tock/libtock-c/pull/494)
    - [Encapsulated C Example](https://github.com/encapfn/encapfn-tock/blob/dev/lwip/examples/lwip/rust/lib.rs)


## Updates
- Tyler: Message from a grad student at UVA curious about OpenThread and border router. They were able to join our Tock open-thread port to an OpenThread border router, which gives it broader internet connectivity
- Branden: Did anything need to change for Tock?
- Tyler: Worked out of the box
- Branden: Then a question from there would be whether you can run LWIP on top of that. It would be super cool
- Leon: LWIP really expects a layer 2 interface. It could be easier to use SmolTCP on top of Thread
- Branden: Maybe we could fake a layer2 interface
- Tyler: There could be LWIP timeouts
- Leon: Honestly, the LWIP stuff just worked. So I'm not sure
- Branden: Really exciting to see another network that can connect to the Internet
### RP2040 PIO SPI
- https://github.com/tock/tock/pull/4378
- Branden: Two undergraduates (Anthony and Jason) worked on this all quarter. The goal was to make a SPI interface on top of PIO for the RP2040, which could later be used to talk to the WiFi chip on the Pico W
- Alex: So this is a general-purpose SPI.
- Branden: Yup. One problem is that it seems that the WiFi chip on the Pico W uses a weird SPI variant with a single half-duplex data wire. That might require a different SPI program loaded within the PIO. 
- Alex: Does Embassy use the same SPI program?
- Branden: Unclear. There are other PIO programs that at least some implementations that talk to the WiFi chip use. But we might be able to just hook up RX and TX to the same pin and make that work? Not sure.


## Ethernet
- Leon: No update. Everything in the Kernel is in tock-ethernet-staging
- Leon: In libtock-c Pat has fixed the LWIP application. Need to test briefly. Then that's good to merge. https://github.com/tock/libtock-c/pull/494
- Leon: In the kernel, the next PR is tock-ethernet-staging into master. That's all the changes we've already approved
- Leon: Finally, we'll work on Ionut's STM ethernet driver. It's already up-to-date, but it's on a diverged branch so it'll be a little touchy. I might have some questions on testing.
- Branden: Is that the last piece?
- Leon: Amit's USB driver is also missing. I might push him to get that merged


## Strategy Workshop
- Branden: For the strategy workshop, we should be proposing our own next steps
- Branden: Is that PacketBuffer?
- Leon: Yes? It seems pretty mechanical to get in at this point
- Branden: The advantage of PacketBuffer is allowing kernel network interfaces. Do we need those?
- Leon: The current status of networks is that the kernel is a hardware abstraction layer and userspace implements everything
- Leon: For any design in the kernel that works with packets, there is a design where we use something like SmolTCP within the kernel, which would provide a TCP stream interface. That wouldn't allow for multiplexing at arbitrary layers though.
- Leon: If we wanted to divide the stack more flexibly, that requires copying or a design like PacketBuffer
- Branden: Does PacketBuffer solve issues for CAN too? Will CAN just send everything to userspace?
- Alex: Yes. We probably want something like this. If we can construct a good IPC mechanism in userspace, then it doesn't matter. Because right now it's hard to send data between applications. ARMv8 has a better MPU which can help this. If we don't have this, the kernel is needed
- Branden: So a question is if we have been focusing on Userspace, do we really need PacketBuffer?
- Leon: Userspace is more quick-and-dirty. Not really meeting the full spirit of Tock which expects usually high-level interfaces for applications
- Leon: Right now the multiplexing for userspace is a mess. We do this for Ethernet by delivering all packets to all process, and they can only send to the outside world, not to each other. The kernel should do better than this
- Branden: We also have a circular problem where things are challenging in the kernel because we have no tools, so everything is in userspace
- Leon: So we should still strive to have better Kernel-level interfaces
- Leon: StreamingProcessSlice was an example of putting effort into making userspace better. But it's still much less efficient than handling things like retransmissions in the kernel. The context switch is expensive
- Alex: However, most of the libraries are in C. So unless we can actually use some drivers, it's impractical to rewrite everything in Rust
- Leon: C driver in the kernel is a possibility. I do hope to merge that some day
- Alex: What's the blocker?
- Leon: A lot of engineering effort. We just have a research prototype right now. Demonstrates that it's sound and possible, but not usable yet
- Leon: We actually have LWIP in the kernel isolated and working right now
- Alex: Can we make this happen faster? What would be needed?
- Tyler: It's really just us working on this more
- Leon: Needs to be prioritized
- Alex: I'm curious if an MS student could work on this?
- Leon: We have some known ergonomics issues to explore. But mostly it's a clean rewrite from the ground up to get it working
- Alex: This seems critical for network interfaces in Tock. Given the existing libraries in C
- Leon: If you can provide example libraries in C that you need that are open source, that would be helpful
- Alex: LWIP, and probably some CAN stuff
- Leon: We are proposing a BLE driver as a next effort
- Tyler: The biggest issue with encapsulated functions is the ergonomics. How you're supposed to use and do stuff is fixable, but a mess right now.
- Tyler: For LWIP we are just doing a ping test, but that exercises callbacks and allocations, and works.
- Branden: So maybe it's more important for encapsulated functions to work than for packet buffer
- Leon: All the code is open source and published (finally)
- Leon: Here's code now: https://github.com/encapfn/encapfn-tock/blob/dev/lwip/examples/lwip/rust/lib.rs
- Leon: Lots of generics. Some unsafe functions that we just didn't write safely. It is sort-of nested closures all the way down, which I suspect can't be removed. It does work, but it should be re-written to be well-reasoned and documented. Then we need to improve ergonomics a little. Finally, we need to have tons of documentation and guides on how to use it. That would be the steps to having it in Tock
- Branden: Okay, so the thing we should really bring up here is whether to focus on kernel interfaces in Rust for networking, and how important those are. Compared to encapsulated C code within the kernel, for example.
- Alex: Unless engineering effort suddenly appears, we could end up with nice interfaces in the kernel which no one uses
- Branden: Totally agreed
- Leon: Yeah, this makes sense to bring up to the whole group
- Branden: Is there anything else holding you back Alex?
- Alex: Mostly it's been retro-fitting with a Rust layer on top. Rewriting in Rust would be safer, but it's unrealistic. So it's highly important for us to reuse some existing stacks if possible.
- Alex: We did build a CAN stack, and we're still building on it. But it's slow
- Leon: One concern is performance and memory pressure? Those are problems
- Alex: Unclear. Everyone wants "performance", but it's unclear how good they actually need
- Leon: We do need a big chunk of contiguous memory. It's the same as a process. It's easier for the kernel to align this memory though. You can place the image in the middle of the kernel though, if you want, and the linker will handle it. So you could place this wherever and move kernel around it
- Alex: Another option here is to actually include the C code into Rust. And run it, and hope it's fine. Do FFI to the driver and let it process the packets. It has less security, but is better for performance. But if it overflows, we're done.
- Leon: Userspace processes don't work for you?
- Alex: No IPC is killing us there
- Branden: Would message-passing IPC work for you?
- Alex: Probably no. The copying would be a problem.
- Alex: But automotive is still deciding what chips to be on. So we should really decide what Tock ought to do independently
- Leon: Right now we don't even have message-passing IPC. So we really need some options to exist that are usable.
- Alex: From a developer point of view, process encapsulation is easy. But it need communication
- Branden: So we should also push on IPC mechanisms as a big networking need
- Leon: There are more chips lately, like CHERI and x86, which have new ideas and work differently. So maybe this is a way to rethink our IPC to something that could work with either message-passing or pages, as needed
- Alex: One of our colleagues is updating the x86 stuff to tock-registers. There's a PR coming for that very soon. So we can experiment on that as well
- Leon: Amazing. Really excited for that
- Branden: So, in summary we have two items to bring to the Strategy Workshop. The first is IPC. The second is kernel vs. userspace network implementations, and particularly how to move forward with networking in the kernel

