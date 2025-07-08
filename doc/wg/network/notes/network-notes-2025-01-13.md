# Tock Network WG Meeting Notes

- **Date:** January 13, 2025
- **Participants:**
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
    - Felix Mada
    - Alex Radovici
- **Agenda**
    1. Updates
    2. Check-in on Status
    3. Priorities for 2025
- **References:**
    - None


## Updates
 * Tyler: Tutorial at EWSN went pretty well! 15ish people attending. Using RPis to program Tock boards worked well. The Thread tutorial is in a pretty good, stable place.
 * Leon: Really interesting talk at 38c3 on building an open-source WiFi stack for the ESP32s. Lots of reverse-engineering. Written in async Rust for Embassy. It would be valuable to consider for Tock.
 * Branden: For the non-RSIC-V chips, right?
 * Leon: Yes. We could port Tock to Xtensa, which would be less work than a WiFi stack! The toolchain process is a lot smoother today than it used to be.
 * Branden: Are they still building Xtensa chips?
 * Leon: Yes, definitely. So it's not a dead target
 * Branden: In general, there's so much work in Embassy that it would be great if Tock could leverage
 * Leon: Pretty difficult. Bare-metal but with a heap, and async workflow
 * Tyler: Any particular ideas about using Embassy?
 * Branden: No. Just a desire to re-use high-quality engineering outputs
 * Leon: Stuff like probe-rs is primarily built for the embedded-rust world, so we are already getting benefits from some of that work


## Check-in on Status
### Ethernet
 * Leon: Still have a tock-ethernet branch with stm32 Ethernet driver. Still need to do a PR into tock master. There's also a lwip stack in a branch in libtock-c.
 * Leon: What we have right now works in an initial fashion, and there aren't any blockers.
 * Branden: We have had at least one or two people in Slack who were interested in Ethernet status
 * Leon: The other thing that's changed is that Tyler's port to OpenThread to userspace was a pretty huge success story. It's worked better than expected. So I think we should do something very similar for an initial Ethernet step
 * Tyler: Does Ethernet have tighter timing requirements? Thread works so well because it has lots of leeway in timing
 * Leon: Ethernet doesn't have any particular timing requirements. It is a very fast interface though, as there can be a huge flood of packets that overwhelms buffers. That could use improvements possibly, as we just have an insane amount of kernel stack buffering right now.
 * Leon: The streaming process slice we merged recently will greatly improve this. So I plan to migrate to that. You could still DOS a device by sending too many requests, but hopefully TCP retransmits will handle that
### Thread/15.4
 * Tyler: Biggest focus right now is on testing. We worked on some remote tests for Thread, which are fully functional. OpenThread, 15.4 transmit, 15.4 receive, 15.4 transmit raw. All four of these work in Treadmill in a branch. We're going to merge them into the main CI soon
 * Tyler: For the stack itself, on the nRF, we can respond to acknowledgements but we aren't listening for Acknowledgements. That's keeping us from having a Thread router. I do know how to add this, just haven't had time.
 * Tyler: OpenThread stuff is mostly stable right now
 * Tyler: 15.4 interface could use an overhaul, with two weird parallel stacks, but it works and I don't think anyone has time/energy to resolve that for now
 * Branden: There was some issue with 15.4 we were still dealing with?
 * Tyler: It was the off-by-default behavior. We had to add a new permission in the API to request to turn on the radio. That made it into the Tock 2.2 release, and our tests focused on that (and detected other things that had gone wrong)
 * Tyler: Every 3-5 months it feels like the radio stacks don't work, and so having CI tests for it will be a big deal
 * Tyler: There is still something broken (for several years now) with UDP over 15.4. There are issues with binding and Hudson has some thoughts. It's on my todo list to make an Issue for it.
 * Leon: Maybe I missed this, but I think 15.4 still uses its own non-standard streaming process slice idea. Are we going to port it?
 * Tyler: Yes, that's on the todo list as well. Hard to motivate fixing an interface that's currently working. We do want to do it
 * Branden: Two tasks for Tyler: create these issues
### WiFi
 * Alex: For the RP2040, we have the PIO SPI stuff in progress here. That should be pretty easy to do, hopefully. Following that, we want to support the Infineon driver. Reverse engineering the Embassy driver and porting to Tock would be the approach. RP2040 and RP2350 share this chip. The RPi also seems to have a repacked version of this same chip.
 * Leon: Do you remember the layer for the chip?
 * Alex: Based on Embassy, I think it's layer2. They use smallTCP on top of it
 * Leon: I think we should ideally aim for WiFi using a compatible interface to Ethernet. One of the goals we originally set was thinking an Ethernet interface, and hopefully WiFi should be pluggable into that.
 * Alex: It's mostly just additional functions for connection to AP. So hopefully the data interface is very similar.
 * Branden: I've got a pair of students working towards this as well. They're currently working on PIO SPI with the goal of moving onto WiFi after that
 * Alex: We should connect our students together to make sure they are building on top of each other rather than duplicating work
 * Branden: That would be great! I'll contact you and send an email to get everyone in touch
 * Leon: Is Alex working on a port for the RP2350 to Tock?
 * Alex: Vaguely. Interesting, but it has this boot header that we need to support. There are two CPUs: ARM and RISC-V, and I'd target the RISC-V chip
 * Leon: I'd be happy to be helpful on that. Pretty excited about the RP2350 chip
 * Alex: The ARM port should be pretty easy. It's a Cortex-M33 on ARMv8
 * Leon: I'd say the RISC-V interface is likely simpler
### USB
 * Alex: Call for help here!
 * Alex: This stack really really needs some love. Doesn't work with many Host OSes as-is in Tock. I don't know why, right now. We could really use some help here, if anyone has experience
 * Leon: I could ask around. I can also poke Amit about it
 * Branden: USB has been an issue for a while. The interfaces that exist are not very composable or extensible.
 * Alex: I'm not sure what's missing. We're missing some "IAD" descriptor that Windows needs. We seem to have everything else
 * Leon: Amit did get Ethernet working over USB, which might be our most stable Ethernet implementation right now
 * Alex: We need to understand the Descriptors. The ordering and which ones we include is important. I copied from Embassy and honestly don't see any differences between that and Tock. I can't find any good documentation on these descriptors.
 * Alex: My testing was on the RP2040 nRF chips might be better, but I don't know if they work on Windows either. There is some weird bug here
### CAN
 * Leon: We always meant for CAN to be a use case for the streaming process slice stuff in userspace. The kernel abstraction exists and is merged. I don't believe there's a userspace example of interacting with this from libtock-c (or libtock-rs) to let people use this interface correctly. If CAN doesn't do this, we will end up doing it in Ethernet
 * Felix: Working on decoding CAN packets. Reverse engineering a lot of other's work.
 * Felix: There's a CAN driver in Tock right now. I have some updates for it re-writing it. CAN messages usually just contain data, but the hardware stores the ID as well. If you filter which IDs trigger it, if the sender sent an error for example, I need that metadata later to decode the packet. So the current driver de-serializes each frame in the driver and passes it on. What I plan on doing is to have the driver just return a reference to some struct that is chip-dependent. And that struct will have a function that extracts the data / ID. The exact layout will be dependent on the chip. The higher-level capsule could then use those functions to decode it. The hardware also has some special values, which need a trait function to access. Using both of these, we could DMA the packet when it arrives and decode it at a later time.
 * Felix: I'll also have to consider the userspace/kernel interface. I think it's byte read/write right now, which isn't going to be good enough.
 * Branden: Some examples of userspace interactions in 15.4 or Ethernet would be useful for you to copy from
### PacketBuffer
 * Leon: Interface seems to be pretty good right now. The implementation needs some love, as it was a bit cobbled together. 
 * Leon: Amalia worked on Port to UART, which is good. I'll need to look into the status on that.
 * Alex: The proof-of-concept was great for UART, but we haven't really touched that since summer.
 * Leon: I think that the UART work wants to be able to determine which console it's interacting with to determine if multiplexing should happen or not. That's beyond the core infrastructure
 * Leon: We will end up wanting a PR that moves at least one thing over to PacketBuffer
 * Branden: I think something big that's missing in PacketBuffer was a clean example of how to use PacketBuffer. Something that shows the purpose of the interfaces in action. Doesn't have to be fully functional. That'll be helpful in showing for the PR whether this is the "correct" thing
 * Leon: I also wanted to do a write-up about our design that could function as a blog post
 * Branden: That could be very valuable for a PR as well


## Priorities for 2025
 * Leon: Ethernet merged into Tock master first
 * Branden: Coordination on RP2040 WiFi efforts
 * Leon: Return to PacketBuffer with documentation progress
 * Leon: Implementation of streaming process slice for userspace interaction for some system
    * Alex: C or Rust? Neither exists yet
    * Leon: C to start with
 
