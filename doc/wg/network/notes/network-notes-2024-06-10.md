# Tock Network WG Meeting Notes

- **Date:** June 10, 2024
- **Participants:**
    - Alex Radovici
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
    - Amalia Simion
- **Agenda**
    1. Updates
    2. Tutorial
    3. TockWorld Planning
    4. PacketBuffer
    5. 15.4 Status
    6. Buffer API
- **References:**
    - [4023](https://github.com/tock/tock/pull/4023)


## Updates
- None


## Tutorial
- Leon: Tutorial overview, also given in Core team.
- Tyler: In broad strokes, a success. Leon did a bunch of the writing. So there's a strong networking tutorial around now that we can use.
- Tyler: We had some bugs at the tutorial with the Thread network. The issue was due to on of the 15.4 PRs that fixed corrupted packets by dropping them. One function returned when it needed to advance the state machine first.
- Leon: We also identified some other bugs. The current re-entrant printf issues mean it's virtually impossible to build reliable userland applications right now. This has reinforced to me that if we want reliable apps, we need to fix yield-for. The only realistic way to write async things now is with callbacks that set bits, and then a big select loop in main.
- Branden: All async or sync works though right?
- Leon: Yes, but even printf is sync and gets mixed in to a lot of things. And the network stuff is async right now, so that means you never get the choice to use all sync stuff.
- Tyler: Right. I don't know how you'd receive packets with a sync operation. You don't really know how long it might be before an event occurs. You need to use async operations.
- Tyler: In practice, it's very hard to have both sync and async operations, and also very hard to not have both.
- Tyler: I'll be working on the rest of the bugs this week and test things.
- Tyler: We had an issue with the VM image freezing. Pat considered that maybe we should buy some cheap chromebooks instead of putting effort into a VM image.
- Branden: Maybe we should be working on WSL2 Tock support?
- Alex: We tried that. It just DOES NOT show the USB ports.
- Alex: We could move to probe-rs to help the toolchain work on Windows. Then people could build on WSL and load from command prompt
- Leon: Some action items from tutorials:
    * Summarize the problems we had more formally
    * Push on the yield-for PR with evidence that we need it
    * Ensure that the tutorial still works and makes sense when fixing bugs
    * Figure out what to do around VM image


## TockWorld
- Branden: Wanted to talk about strategy for TockWorld for this Working Group
- Leon: Okay, so let's keep thoughts on PacketBuffer reasonably brief and talk about organizational aspects. We can focus on 15.4 and tutorial developments. That way we aren't redundant on PacketBuffer
- Branden: Sounds good. And Amalia will cover PacketBuffer in her keynote, so the update from the working group will not go into PacketBuffer details


## PacketBuffer
- (Rewrote history a little here. The TockWorld and PacketBuffer discussions were intermingled.)
- Alex: Would like to have a plan sketched out on switching to PacketBuffer (screen).
- Leon: I like the other use cases, and I'm excited that it might be useful for several use cases. I am worried about whether that will hurt our original focus of working on IP stack
- Leon: I do think it's relatively straightforward to implement an IP stack on top of Ethernet with PacketBuffer right now. I can spend one or two days to do this before Tockworld
- Alex: Can we use the STM32 layer2 we wrote?
- Leon: We have quite a few implementations all in a branch that are all using a unified interface. So I'd like to push on that as an intermediate result for Tockworld
- Alex: So we'd port them to PacketBuffer before merging into master?
- Leon: Yes, I think it'd be more convincing if we have it working
- Branden: Idea sounds great; don't necessarily have to wait to merge PacketBuffer if it works for console. It should be its own PR still, but one use-case is enough to motivate. Don't want to block console on any pending IP-stack changes.
- Leon: Just want to make sure that we can make changes later for IP if needed
- Alex: I think we should merge stuff as-is for now, and make changes later. Rather than keep stuff all on branches.
- Leon: Amalia and I can create a PR together
- Branden: How close is the console subsystem? Would still make sense to have both at the same time.
- Amalia: It needs some cleanup and adjustments for initialization still. We also need to implement the UART for every chip supported, if I'm not mistaken. It's only done for nRF52 right now.
- Leon: Oof. I also revived a PR trying to migrate the UART HIL. And that's tricky because of how wildly different UARTs are between chips. So if we're doing that migration anyways, it's a similar effort to port towards PacketBuffer. Separately, we should think about whether it should be done together or in separate PRs.
- Alex: This is a lot of work. 1) change the HIL and 2) use the PacketBuffer. How close are we to using the new HIL?
- Leon: The new HIL exists, and I implemented a few chips in a branch. My benchmark for success is just compiling though, because I don't have hardware to test them. So we should only merge close to a release, so there's lots of testing. I think porting to PacketBuffer will be equivalent amounts of churn, but it's easier to get right. Just bytes at the bottom anyways. Changing the HIL hits the state machine.
- Alex: Porting things to PacketBuffer is something we have some students who have some time for.
- Leon: My concern is that it would be more work to do porting to PacketBuffer first, and then porting to the new HIL second. It would be less work the other way around.
- Branden: Not sure about this. We don't want to sit on PacketBuffer until vague changes like HIL changes are in.
- Leon: Yes, that's a good point. Not opposed to this.
- Branden: Should PacketBuffer just be its own PR right now?
- Alex: We'd have to do a lot of explaining on the PR, but can get feedback early.
- Leon: Okay, I'll meet with Amalia to go over the current version of the code. We'll see how much churn and cleanup, and then we'll identify the best path forward.
- Leon: Plan to get a draft PR out before Tockworld


## 15.4
- Tyler: Not too much here. Brad's done a ton of work, but it's mostly shuffling around Tock kernel internals, and shouldn't affect the tutorial.


## Buffer API
- https://github.com/tock/tock/pull/4023
- Alex: The problem we're facing with CAN driver is that packets come in bursts. The car often sends a bunch of packets, then sleeps. So you have this situation with a large number of packets that the application will take some time to process. So we needed to swap buffers. The application would swap the buffer and notify the driver that it's swapped the buffer out. But there's a race between swapping and saying you swapped. Packets can come in between the two. That means that the kernel can't track where it is in the buffer. So what we did was used the first bytes of the buffer to hold the offset. The kernel will update this. And the userland will clear it when setting a new buffer.
- Alex: So, I formalized this by adding functions to the process slice API to implement this with helper functions. That's the idea in a nutshell.
- Tyler: Interested to look into this and to see if it can be used for 15.4. This is nearly identical to what I implemented, but without API support. The difference with the ring buffer I'm using is that this loses packets once the buffer fills. That could create issues for OpenThread. In reality, in the off-chance that the buffer fills you'll drop a packet and the packet will get retransmitted. So I think this system would work for me.
- Branden: Can't you have both ideas together?
- Tyler: You allow a buffer, the capsule writes to the buffer, it notifies the app, but the app hasn't yielded (so didn't have a chance to swap the buffer).
- Alex: This would swap the buffer without even yielding. Depends on the app. For CAN, we checked if this was the first packet in the buffer, and then notify the app. This is a separate issue, avoiding filling up the notification queue.
- Alex: We may need to implement "reserved upcall" / idempotent upcall -- where a second upcall doesn't add anything new if one already exists
- Alex: PR will follow for this.
- Leon: Idempotent upcalls looks like the right mechanism in conjunction with this buffer structure.
- Branden: This feels like we're creating a bunch of special cases.
- Alex: Touch screen driver has the exact same issue. I only care about the "last" update. So I'm overwriting the buffer and just want to ensure that there is one upcall existing
- Tyler: So the implementation in the PR for Buffer API says we drop new information.
- Alex: The touch driver would use it a differently. We can replace whenever we receive new touch events. What's actually important here, I drop the last packet while you drop the first packets. For touch, I do want to drop the first. But both should be an option
- Leon: Okay, so it's not really necessary for Touch to use this Buffer API. Both network and touch just want idempotent upcalls. These are two separate concerns.
- Leon: Secondly, on these policy decisions for replacing early or late packets. I don't see why this wouldn't be possible for us to have in the implementation as a parameter.
- Alex: Definitely possible. We just have to pre-pend packets with the length. I'm looking into having both options.
- Branden: That makes sense to me. And the idempotent upcalls make more sense to me in that multiple different drivers need them for different but similar purposes
- Tyler: Alex, you're not changing the allow interface with these buffers, you're just creating a standardized way to represent a current length / offset in these buffers. What's the reason for the checksum still?
- Alex: The idea was to try to recover if the app misbehaves. If the app just writes a zero to the first position, or the app appends some data, the driver verifies this and sees things are uninitialized and can reset to zero. The driver would work either way, but I see this as a perk for the application. I'm not sure if this is needed or not
- Leon: I think on the core call, we determined that we don't need protect the app against its own misbehavior. We're still doing length checks all the time, so this would really just check for whether the app is written correctly. And generally, we don't do that in the kernel. So I don't buy the argument that we want the checksum. It also makes the implementation more complex.
- Alex: I could remove it
- Leon: For Tyler, the implementation just represents the length/offset into buffer in a standardized way. This infrastructure also implicitly gives you the ability to swap buffers by resetting an offset.
- Tyler: The offset would be reset when the application shares a new buffer? That makes sense
- Leon: One thing that just came to mind, maybe we can introduce a bit to show the application that there was an overflow. This is similar to the congestion notification in networks. This could be reasonable to share, as it's a real outcome
- Alex: We share this in the upcall right now. For example, we need to report an issue if we drop CAN packets.
- Leon: Okay, that makes sense
- Alex: Imagine having many packets. We have some small space left in the buffer, and can't fit a big packet so we drop it. Then we still fit in a small packet. Was this an overflow?
- Leon: Maybe we do want to signal. That's worth thinking about, I am also uneasy about special-purpose implementations here
- Alex: We could have a bit reserved for it. Most drivers will still notify in the upcall
- Alex: Right now we can't put this into an upcall, because we might overflow the notification queue. Once we have idempotent upcalls, this wouldn't be an issue any longer.
- Branden: You could get rid of the xor bits, and add 8 flag bits. And one of those will a overrun bit.
- Leon: This buffer isn't aligned anyways, as userspace can share anything. So we could just have a five-byte header
- Alex: I also wanted a version for the buffer API. Maybe four bits for that
- Leon: One trivial way to add versioning without immediate overhead: just say that one flag bit is always zero in this version. So that bit will become one if there's ever another version
- Tyler: Our current 15.4 buffer structure has a lot of wasted space, as you have to pass down the full max buffer size from userspace. Is there any issue with the waste when receiving smaller packets? Is this worth thinking about?
- Leon: Big issue. This gets MUCH worse in Ethernet. We don't want 64-byte ping packets in a 9 kB buffer... This API is still a great first step, as you can stuff packetized frames into the buffer if it's too big. So really the benefit here is for Rust to compose near-zero-cost abstractions that give you several APIs for the different structures we're making in the buffer

