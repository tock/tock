# Tock Network WG Meeting Notes

- **Date:** June 30, 2025
- **Participants:**
    - Alex Radovici
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
- **Agenda:**
    1. Updates
    2. WiFi Development
    3. IPC
- **References:**
    - [MobiSys 2025 Tutorial](https://tockos.org/events/mobisys25)
    - [Tock Strategy Workshop 2025](https://tockos.org/tock-strategy-workshop-2025/agenda)


## Updates
- Branden: how was the MobiSys tutorial? https://tockos.org/events/mobisys25
- Tyler: It was good! 6-7 people attending. The materials for it are very good. Kat and Ryan from zeroRISC made some great root-of-trust materials. Brad and Vish had dynamic loading demos that worked great. Thread worked well.
- Alex: Darius is close on merging ARMv8, with a patch for the ARMv8 MPU
- Alex: Ionut ported the embassy executer to libtock-rs. It can do delay and GPIO with no modifications!!
- Alex: We also profiled timing for Embassy, libtock-rs, and RTIC. Embassy and libtock-rs were equally slow. RTIC was super fast.
- Alex: I'm interested in adding top-half handling to Tock based on RTIC ideas. I've got some ideas there that can disable interrupts and do really small things that should work.


## WiFi Development
- Alex: Started looking into the WiFi. Some interns working on it
- Alex: This will also need IPC to run network stack in userspace
- Tyler: Do you think a userspace stack will have too much latency?
- Alex: Not sure yet
- Alex: Students are just getting started right now.
- Alex: Also starting with PCI development, with a plan to eventually do Ethernet over PCI.


## IPC
- Branden: Can we get a summary of updates from the call last Wednesday?
    - Tyler: The gist was we should do something about IPC. Someone specific needs to be in charge. Network WG is interested in it for sure. Maybe it makes sense for Network WG to guide process.
    - Tyler: My proposal was thinking it could be good for the four of us to hammer out a plan for a higher-level design for IPC before TockWorld this year. We've never had a concrete proposal for a new IPC design, which would be pretty valuable.
    - Leon: We made it clear that Network WG would be shepherding, but not necessarily on the hook for engineering right now
    - Alex: I think this is super important. IPC is so important to many things. We could have people work on implementation if given a starting design
    - Leon: I do think that discussing this is super important. As we've seen with other efforts like the MMU which has been a huge diff, if we have a design we can agree upon in-advance, it would reduce friction a lot (general agreement)
    - Tyler: We'll save time by having conversations first
    - Leon: On the Core call, we raised the idea of revisiting Lawrence's IPC implementation which may or may not be public right now. That could be a good inspiration before we write up a concrete design.
    - Leon: Another reference point is from a labmate of mine (Gongqi) who is working on Tock for multi-core systems. He has some thoughts on IPC across cores. He's got basic implementation working of exchanging data across multiple cores.
- Branden: We also discussed this at the Tock Strategy Workshop: https://tockos.org/tock-strategy-workshop-2025/agenda
    - Client-server seems to be the dominant model. One service app (server) and multiple clients
    - Multiple possible mechanisms message-passing, memory-sharing, remote procedure calls. Would be great if they could be unified in some way.
    - Zero-copy is great, but not always required. Would be great to have the interface transparent to whether copies or memory-sharing is happening behind-the-scenes
    - Sometimes have most data which could be copies (small messages) but occasional large data that needs to be shared instead of copied
    - multiple-buffers per client/server connection is important
    - authentication is useful, but may not be a first-class priority?
    - Discovery could be uni-directional: clients discover server and register with it
- Tyler: Was there anything about policy for apps faulting?
- Branden: Bobby said IPC service crashing could cause issues for others. But one client crashing should affect other clients
- Tyler: Important to consider in our design
- Alex: Reliable clients should be possible even if server crashes. Not all clients have to be reliable, but it should be possible
- Alex: We should focus on paging, ARMv8 MPUs, and RISC-V PMP. We should avoid ARMv7 as it's going to hold us back a lot. For AMMv7 we could require copies, but other systems could have memory sharing
- Leon: That could be a reasonable tradeoff. But I want to see if it's necessary first.
- Alex: I do agree. But I'm afraid that ARMv7 would require over-engineering
- Leon: I agree with keeping it in mind. I just don't want to make those calls too early
- Tyler: We should be comfortable exploring is what works. Perfect is the enemy of good, so we don't need to be perfect.
- Branden: Having an interface that works at all on ARMv7 I think would be fine. Could totally have a performance hit
- Tyler: How should we structure these conversations? Maybe set some goals for what we want to discuss to make sure we make progress rather than just spitballing. We want some tangible next steps
- Branden: Some specific items to bring to a next meeting. Current IPC knowledge, CHERI IPC implementation, other IPC implementations (Hubris was Alex's recommendation). We could also mock up IPC interfaces.
- Alex: I can also put a document with several requirements for industrial environments for IPC.
- Alex: Hubris can send small payloads, but can also share buffers. At least a year ago they always copy. But it looks like sharing. They say they copy on demand. Whole OS is based on IPC.
- Branden: I can look into Hubris and other OS designs for IPC. I'll also send a message to see if we have access to the CHERI implementation or not.
- Tyler: I'll look through what exists and then write up a document about how IPC works
- Tyler: I know Kat from zeroRISC also has an implementation of some type? We could reach out to them to ask about it. I'll ping Kat to ask about it.
- Branden: An outcome I think would be helpful for TockWorld would be pseudo-code for interfaces and what the implementation(s) behind it would be
- Tyler: Having a pluggable implementation could be helpful here.
- Branden: I could see board files specifying copy vs shared memory, for instance. I could see multiple interfaces in userspace too. Maybe a memory-share interface and a message-passing interface.
- Tyler: Why not just always use shared memory if you have it?
- Branden: Good question. Not sure if that's important
- Branden: I'm thinking we could have multiple IPC interfaces. IPC is a big umbrella, and maybe some parts of the back-ends are shared, but I think it might make sense to present multiple interfaces to userspaces: message-passing (which could be shared memory or copied buffers), memory sharing, and remote procedure calls (which are message passing with return value messages, but maybe _feel_ different)
- Tyler: Something that's motivating me is exposing OpenThread to libtock-rs over a service. That's an overall goal. Would be really cool to have OpenThread support from Rust

