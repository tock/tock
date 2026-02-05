# Tock Network WG Meeting Notes

- **Date:** February 02, 2026
- **Participants:**
    - Branden Ghena
    - Tyler Potyondy
    - Leon Schuermann
- **Agenda:**
    1. Updates
    2. Check-in on what needs attention
        * DMA Slice
        * STM32WLE5xx
        * IPC Updates
        * LoRa in Libtock-C
- **References:**
    - [15.4 Documentation](https://github.com/tock/tock/pull/4726)
    - [DMASlice](https://github.com/tock/tock/pull/4702)
    - [STM32WLE5xx](https://github.com/tock/tock/pull/4695)
    - [IPC Updates](https://github.com/tock/tock/pull/4680)


## Updates
 * Tyler: 15.4 doc comments from Paul. https://github.com/tock/tock/pull/4726
 * Tyler: Looks good. Nice work. There are a few little things to be changed for consistency's sake. PSDU and which frames are included. We want to make sure this makes sense everywhere. I can figure that out.


## DMASlice
 * https://github.com/tock/tock/pull/4702
 * Leon: This is in a place where it can probably receive some more reviews. The feedback so far has been positive.
 * Leon: Maybe one small thing that might change: right now it's a slice over arbitrary type T. But that could have uninitialized data in it, like padding bytes, which shouldn't be exposed to DMA peripherals. And DMA can write arbitrary values, so the type T shouldn't allow for values that aren't valid Ts.
 * Branden: Requiring an array of bytes is the C way of doing this. And it's someone else's responsibility to make a type into an array of types.
 * Leon: Actually some DMA stuff has more requirements. For example, OpenTitan expects u32 arrays, to make things more performant for big number accelerations. So we want to work over both
 * Leon: So we'll probably do a trait, only implemented for unsigned integers.
 * Leon: We also need a couple of people doing a soundness review and reading over everything. We also need an implementation for Cortex-M. And an example that showcases how to use this for a driver that isn't VirtIO. Those could be combined.
 * Tyler: UART on the nRF would be a great target. It's currently unsound. And we've known it for a while and never touched it.
 * Tyler: I'll plan to look at it this week.
 * Tyler: 1) do we do anything to address the time the DMA operation is occurring. Or does this just replace TakeCell.
 * Leon: Implicit. It's a safer alternative to TakeCell where we store buffers as pointers, not slices. It turns out that's not enough though. You also need to include code that tells all of the Rust compiler, the CPU, and other hardware, to avoid certain reordering before you give the buffer to hardware and after getting it back from hardware. Doing this is inherently coupled to the operations that start and stop DMA operations. The code you use to modify a register doesn't change, but there's an implicit dependency.
 * Tyler: If you supply a dmaslice to a DMA, previously we'd grab the take cell, take the pointer, drop into a register. Now what do we do?
 * Leon: We still do that. There's a method to get a pointer. Before getting the pointer, there's a release fence that flushes writes. Running release only applies once you do a subsequent memory write. Writing the pointer to the register guarantees that prior writes to the buffer are visible to hardware. So there's surprisingly little code in this PR, just a ton of documentation.


## STM32WLE5xx Support
 * https://github.com/tock/tock/pull/4695
 * Tyler: Ping on this. There hasn't been movement recently. It's not urgent, but sizable and we want it to move forward eventually.
 * Tyler: Got some comments from Brad three weeks ago, which were addressed.
 * Tyler: Just a massive PR: 47 files changed. So it could linger for a while if not pushed.
 * Branden: Okay, I should follow up on this one. I made myself the assignee on it
 * Tyler: The only real concern is if there are big updates in other PRs that make me do big updates to this to keep up. But not a big deal.


## IPC Updates
 * https://github.com/tock/tock/pull/4680
 * Branden: Need some documentation updates (small). Need to implement synchronous mailbox and asynchronous mailbox (medium-big). Those would be enough to merge into Master.
 * Branden: Also need to design Shared Memory system (big).
 * Leon: I am interested in looking into one of the mailboxes when I have time.
 * Leon: An update is that I'm also working on a research project on general IPC design. Tangentially related to Tock. Collecting some research papers and discussing internally. Then I'm hoping to share out to the larger group.
 * Branden: Awesome. If you find things that Tock should be considering, then please let us know. We can still modify the design.
 * Leon: Looking at lessons-learned from other systems and using that to gain some confidence in our own designs. Looking at SEL4 and others. Including some systems that have high-performance requirements.


## LoRa in Libtock-C
 * Tyler: Userspace side of the LoRa/LoRaWAN stuff, which originated from Alistair. The definitions for that are rather radio-specific for one of the platforms (Apollo?) that Alistair was working with. That's something we try to avoid in libtock-c I think. So it's something to think about.
 * Branden: I think that was the only one. So it's a good point to think through generalization if we have a second.
 * Tyler: Yeah, will definitely need some updates. On the long-term horizon.

