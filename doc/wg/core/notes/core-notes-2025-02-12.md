# Tock Meeting Notes 2025-02-12

## Attendees
- Branden Ghena
- Leon Schuermann
- Alexandru Radovici
- Amalia Simion
- Brad Campbell
- Amit Levy
- Johnathan Van Why
- Viswajith Govinda Rajan
- Kat Fox
- Tyler Potyondy

## Updates
 * Brad: Submitted a tutorial request for MobiSys in June. The idea is to continue with the Thread tutorial we've been doing, but to expand it to add dynamic process loading and other security-based ideas like process filtering and signing. This way it's hopefully a good fit with the audience and also pushes us to move along development and documentation. More security focused rather than sensor-network focused.
 * Brad: Plan for a Tock workshop in March. Next Tockworld isn't going to be until September, so it seemed like really should do some project-planning. This will be an online meeting, probably towards the end of March. Any core members and other technically interested people are invited.
 * Brad: We'll put together a full agenda later, but the vague idea is a check-in on what we discussed and last Tockworld, some breakout groups for topics of discussion, and some capture of current efforts and priorities for Tock: what's in flight and where things are going.


## Tockworld Planning
 * Amit: Alex brought up that Rustconf dates have been publicly announced and is in Seattle. So we would like to align Tockworld with that.
 * Amit: We're still waiting on a final venue confirmation, although this looks good so far. Probably at Microsoft but with a backup at University of Washington. The hotel that Rustconf is happening in would also be an option if necessary. So, no exact address yet, but we can commit to dates in Seattle
 * Amit: The main Tockworld conference day would be the Friday after Rustconf. September 5th, 2025. That's often the Rustconf "unconf" date, an optional part of Rustconf. So we'd be overlapping with that. The Monday before is not an option as it's a federal holiday in the US (Labor Day).
 * Alex: Would Tockworld be just one day?
 * Amit: The main day would be one day. But we could do another day for technical stuff or tutorials. We could even try to do a tutorial on Tuesday _at_ Rustconf.
 * Alex: That would be my idea. We would have more people there for a couple of hours at Rustconf.
 * Alex: I'm meeting in-person with someone from the Rust foundation soon. I could bring things up with them.
 * Amit: What would make sense to bring up?
 * Alex: That we're doing Tockworld around then. Maybe that we can have some presence there and do a tutorial. We should also apply as an associate foundation
 * Amit: I'm working on the paperwork for that already actually
 * Amit: Presumably there will be a call for tutorials or workshops?
 * Alex: Possibly, but I think it was just a call for talks so far.
 * Amit: It would be great to touch base with them on this then
 * Alex: So, do those dates seem okay to folks, generally? It steps on the unconf, but does help double-up on people attending
 * Brad: So the "main day" would be community day with talks?
 * Amit: Yes
 * Brad: So we could do a developer day around then too?
 * Amit: Yes, maybe Monday, Wednesday, Thursday, or Saturday. That would not focus on external people, but rather on frequent contributors and can get into the technical weeds


## StreamingProcessSlice and Libtock-rs
 * Branden: In the Network working group call, we've been talking about the plan for Ethernet merging into Tock mainline. Ethernet and CAN share a similar issue of collecting many packets and needing to send them upwards, possibly directly to userspace. To support this, Alex added StreamingProcessSlice, which is a mechanism for sharing a buffer that can contain multiple packets and notation of how many packets there are between the kernel and userspace. However, StreamingProcessSlice doesn't have any userland support yet, so in moving Ethernet to it, we need to develop that as well. The libtock-c side here makes sense, but there's a concern that we're not sure how or even if this interface will work in libtock-rs. So if we move Ethernet to use StreamingProcessSlice, it could cause issues for StreamingProcessSlice being used in a Rust userspace.
 * Branden: So the question, is it okay to move the kernel to interfaces that libtock-rs does not support and we don't have any plans (or path) to support in libtock-rs?
 * Amit: So, it's possible we could have an interface that _only_ works in libtock-c because it's so C specific. Although lots of interfaces from C have been coerced into Rust with enough unsafe.
 * Amit: Another take is that libtock-c has some specific ideas about interfaces, and maybe other userlands in Rust could support a similar interface but libtock-rs does not
 * Amit: A third take is maybe that we have problems with the StreamingProcessSlice interface?
 * Leon: I think we're looking at the second option here. There are perhaps aspects of StreamingProcessSlice that don't work well with libtock-rs's current design. But I think it's primarily libtock-rs design restrictions compared to other possible designs.
 * Leon: Rephrasing the problem, we've had a lot of desire for supporting libtock-rs and making it a first-class citizen. But this is a fork in the road where we have to decide whether we support it right away, or if we punt on it for yet another driver/subsystem.
 * Alex: The problem isn't only StreamingProcessSlice. I tried porting the Embassy execuctor on top of it, but swapping buffers is basically impossible in safe Rust right now.
 * Amit: So there's a somewhat related issue with Embassy, that the current libtock-rs API might not be a good fit for it (and vice-versa)
 * Alex: It's impossible to swap buffers in safe Rust. Because the buffer is allowed and you don't unallow it, it just goes out of context and issues an unallow system call. But if you have several stacks because you have futures, this doesn't work. The future isn't the problem, it's the polling function when you write the bottom layer
 * Johnathan: For futures in general, I think we need to move to an API that supports pinned buffers. Alex is describing: https://github.com/tock/libtock-rs/issues/494
 * Johnathan: Years ago I looked at using futures and saw a huge code size impact. And today Embassy has a pretty large code size. So I don't think it should be a core part of libtock-rs, but it really should be a possible integration for people who aren't concerned about that. Not have libtock-rs rely on it, but support it for sure
 * Leon: The interface the Tock kernel gives to userspace over syscalls sort of looks like a DMA interface. Alex's comment in the networking Working group meeting, was that the layering of Embassy on top of the existing libtock-rs abstractions, but maybe over an interface with less assumptions/constraints on top of raw system call semantics could be a lot easier. I don't want to go off-topic though
 * Amit: Sort of related. My opinion is that there need not be one libtock-rs or one libtock-c. We even sort of have two libtock-c versions for sync and async APIs. It's just possible to layer them in C, but they ought to be used separately. That could be fine for libtock-rs as well. Maybe Embassy could be its own thing. And libtock-rs focuses on applications that are entirely synchronous and they get more safety and more efficiency
 * Amit: So if that's the world we might live in, the question for StreamingProcessSlice is, is the best version of this supportable by _any_ rust userland, probably yes. And part of the goal here is for higher concurrency stuff, keeping receiving while other operations are going on. Maybe libtock-rs could use those interfaces but without the benefits
 * Johnathan: I don't necessarily think the Embassy integration should be a different thing from libtock-rs. I think libtock-rs should provide the proper abstractions Embassy would need
 * Amit: Let's focus on just the StreamingProcessSlice question for now. I think it would be great to have a common core. But the main question is whether we are okay integrating StreamingProcessSlice into Ethernet and CAN if there's no sense of how to support it in libtock-rs
 * Johnathan: Yes. If you can use it soundly from C, we should be able to use it soundly from Rust
 * Amit: Is there a way to write a libtock-rs driver today with StreamingProcessSlice which just doesn't take full advantage of it?
 * Leon: We're only going to expose the one interface for those subsystems. And I do think we can make a sound implementation for C. And I believe Johnathan that means we can make a sound implementation for Rust, I just don't know what it would look like.
 * Leon: There are two parts here. What is our commitment to supporting a Rust userland and how do we apply that to new interfaces in the kernel. Then the specific StreamingProcessSlice version of that question at hand
 * Amit: I'd say that we want to be reasonably sure if we stabilize an interface, but that's not happening here. But I see that there's a desire to not rewrite things again in six months if there are problems
 * Amit: I am a little worried that soundness requirements in C and Rust are different. Liberally using unsafe Rust could do it, for sure. So there's a question of whether that's acceptable in libtock-rs for some drivers
 * Leon: We can be more concrete even. StreamingProcessSlice allows one buffer to the kernel which can receive multiple packets. The kernel fills up the buffer and tracks the current offset within the buffer itself. When notified that one or more packets are in the buffer, the userspace can swap this buffer out with another one
 * Leon: So in libtock-rs we could design something sound that holds two buffers and swaps them out without ever giving up ownership of them. But that's not what the interface will look like in libtock-c, which wanted to avoid copies.
 * Amit: So maybe the libtock-rs version could not get the benefits, but could exist. For example, maybe just one buffer even.
 * Leon: There's a question of what's a basic correctness property. But you could have non-atomic swapping of buffers, but then you start losing packets and data.
 * Amit: That's no worse than the system before StreamingProcessSlice
 * Leon: It is. What we have now is a very expensive system that buffers packets in the kernel. That will go away
 * Amit: Okay, in my mind either of these would be fine. Part of what you lose from an entirely synchronous interface is packet loss. Even the expensive kernel stuff can overflow. So, if either of these options is doable, even with loss of efficiency in libtock-rs, that seems okay to me
 * Leon: I don't think this is capturing it quite right. The kernel will assume that there is always a buffer allowed. And if the userspace doesn't have a buffer allowed, the kernel will drop packets. So if we implemented this in libtock-rs without an atomic buffer swap, the kernel wouldn't buffer and we'd lose stuff.
 * Amit: I think the difference in contract there is not super meaningful. And the userland won't know what the kernel will have provisioned for packet buffering.
 * Amit: But back to the question, it's fine to use this if the difficulties are not difficulty to _any_ rust userland, and if there's a version for libtock-rs that can be implemented even with big downsides like additional copying or lots of packet loss
 * Branden: This makes sense to me. This is what we were looking for, for now
 * Amit: We'll move a broader discussion of libtock-rs priorities to a latter meeting

## TRD104
 * https://github.com/tock/tock/pull/4228
 * Brad: Any opposition to merging this?
 * Johnathan: I've got a concern. I'll comment on it

