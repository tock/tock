# Tock Meeting Notes 2026-04-01

## Attendees
 - Branden Ghena
 - Leon Schuermann
 - Hudson Ayers
 - Johnathan Van Why
 - Brad Campbell


## Updates
 * Brad: Some fixes to Tockloader to make using nrfutil a little speedier. Also changed some things to support multiple ports on nRF52 boards, which seemed to change. We'll do a release relatively soon

## Tock Registers Status
 * Johnathan: Design I've been prototyping with is workable, and I've got some documentation up on it. I've got two PRs ready to open, one to Tock switching drivers to it and one to Tock-Registers with the update.
 * Johnathan: I'm going to write up which things really need eyes and are open questions, then I'll make the PRs and close the older PRs.
 * Johnathan: I'll also mention some alternative options. The next major milestone is deciding whether this is the design we want to move forward with
 * Leon: Very exciting. I'd be happy to sanity check design documents and understand parts of it
 * Johnathan: The first PR is going to be huge. I'm hoping for thoughts on it first, then I'll make new PRs to heavily review and merge parts of it. I'm hoping to see it in the next week.
 * Branden: It will be helpful to point us to which things to focus our attention on and start out debating.
 * Johnathan: Want to avoid bike-shedding for now, but I can talk about features that exist or are missing for now
 * Leon: There are people using Tock-registers now who are strongly considering moving to other designs, particularly due to soundness issues and documentation issues
 * Brad: What's the documentation issue?
 * Johnathan: The documentation generated from register bitfields right now is terrible. It's something we can address, but not my first priority.
 * Brad: Is it possible that this documentation issue is easy to fix?
 * Johnathan: I think it should be easy to improve. At some level we'd need a proc macro to do better docs. But even macro_rules should resolve some of that
 * Branden: I'm not worried specifically about keeping current users, but I do tock-registers as a valuable thing we can provide to the community. If we can have something sound and highly useful that solves problems, that's great. People will use it if they want to
 * Leon: Yeah, people do say they find it valuable now
 * Johnathan: We would be the only registers crate with native unit testing support

## DMA Slice Status
 * Leon: Pickup up DMASlice stuff again. Thanks for naming updates from Brad! They're mostly great, there's a JustBytes one that I don't like but we can take that offline.
 * Leon: What's the plan going forward with this? I think this is a step in the right direction but we're not sure we want to port all chips to it immediately. So if I get this into a state where it looks good, do we merge it on its own? Or do we wait for some nicer abstraction to use it?
 * Brad: Glad you like the API changes. I think that'll make it easier to teach. I would really prefer in my perfect world that we move "making this trait thing" into its own crate. Then we can bikeshed over there and come up with whatever traits we need separately. Coming up with names for that trait isn't the most important thing. The trait now is immut-from-into-bytes, which is the same thing that current tock registers does where we say "it's basically a 16-bit number". I see all of those as failures of Rust that I'd like to make a single shared crate for
 * Leon: My reasoning against the shared crate is that splitting things into crates needs a reason, like sharing or applying widely in Tock. But none of those seem to occur here, as it's only an internal bound to DMASlice that users should never see. Also, there exists a much better version of this trait in the zerocopy crate, which is well documented and maintained and highly respected. We're not using it just because of our external dependencies policy.
 * Brad: I am happy to merge the DMASlice stuff though. I have a port of the nRF52 UART driver, which side-bar is crazy that it works today as I think it has a huge bug. I'd be happy to merge that update, after testing. I also have a half a blog post on this. For now, I think we can just have some functions for entering and exiting DMA mode which clearly separates the code and could be refactored easily in the future if we come up with something that's included in tock-registers.
 * Leon: I do think a future adaptation would be relatively straightforward.
 * Brad: So I'll dig up that commit and actually test it.
 * Leon: I think we addressed the other comments on the DMASlice PR. Most were just things to clarify.
 * Branden: Most of my comments were on the order of "here's how I understand it so we can document if I'm way wrong"
 * Leon: Yeah, I think we're doing better about documenting the most important assumptions. Some of it is niche enough that it'll always be there, but clarifications are good.

## Yield-WaitFor
 * https://github.com/tock/tock/pull/4769
 * Brad: When converting libtock-c to yield-waitfor, it turns out GPIO didn't work. The GPIO driver right now notifies all applications if any pin has interrupts enabled. So there's no space in the grant region for specific GPIO storage, just an upcall. So if you used GPIO with upcalls, you store the upcall which creates the grant. If you don't store an upcall, you don't create a grant region, and then when iterating apps we never trigger on this app in the driver.
 * Brad: It's not really clear to me where the bug lies. This PR allocates the grant on yield-waitfor occurring. But it also seems like GPIO has a bug maybe? There are other fixes. It is kind of strange to have a capsule without anything in the grant region. So I wanted to see if anyone had a different understanding.
 * Branden: On the GPIO design, when I started on IPC discovery, my initial design also had upcalls without any grant state. Ispent a while trying to decide if that was okay, and found that the GPIO driver did this and figured it was fine. So I do think there are designs that can end up this way, although I believe a later IPC change meant that there is some grant space in the end.
 * Leon: We also had an issue where one process could denial-of-service another process by filling its queue. We had an allowlist mechanism to prevent that. I guess that's orthogonal here
 * Johnathan: The app presumably knows GPIO could trigger and calls yield-waitfor, but what happens in the GPIO trigger occurs before that? Would that hang forever?
 * Brad: Yup
 * Johnathan: So userspace apps really can't rely on yield-waitfor here
 * Brad: The semantics are "wait for the next occurrence after calling yield-waitfor"
 * Branden: I think yield-waitfor always has this issue, that something could occur right before you wait for it. I traced this the other day, you end up enqueuing something but then later drop it if the yield-waitfor isn't active
 * Johnathan: So we'd need the userspace subscribed before calling yield-waitfor or having some kernel mechanism that enqueues until the yield-waitfor occurs.
 * Branden: We could say yield-waitfor only works with late or repeated behaviors
 * Brad: I'm pretty concerned about this. The real goal of yield-waitfor is to have behavior that works like you expect. But with multiple apps you could be timesliced after starting something but before the yield-waitfor. Then you'd hang forever and that would be impossible to debug...
 * Branden: We could fix this with persistent upcalls, which are saved until a yield-waitfor or subscribe actually occurs. To do that we'd need to use a bitmap for tracking upcalls, rather than an actual queue.
 * Brad: Rather than unconditionally queuing things when an event happens, as I think that would have unintended consequences of queueing stuff we don't care about, what if we change yield-waitfor so that if userspace registers an upcall and we don't use it when we call yield-waitfor
 * Branden: that's what happens now, I think. Even
 * Brad: So you subscribe an upcall, then do some capsule thing which leads to an event in the queue, then we don't do anything with that until you call yield. Then you instead call yield-waitfor, find you have a matching upcall, and then deliver that to the yield-waitfor.
 * Branden: So userspace would have to subscribe, even if they never intend to yield instead of yield-waitfor
 * Leon: Could they subscribe with a null upcall?
 * Brad: I think we can't, because that unsubscribes.
 * Johnathan: There could be some shared upcall function, which is irrelevant because it should never be called.
 * Branden: That comes down to enabling/disabling long-term queueing of an upcall.
 * Brad: So I see how this would work. It does have an unfortunate side-effect where yield-waitfor will still work without the prior subscribe, sometimes. Data races.
 * Leon: We could have a special value to subscribe that means "yield-waitfor" is coming. It would be backwards compatible, but we could pick something like 1 that we'd never use
 * Johnathan: Changing the queue to always drop newer upcalls would solve the default race, but could break functionality for designs that rely on not doing that
 * Leon: It's funny, bundling a command, subscribe, yield sequence wouldn't have this issue. You'd always know what you need to store, as you'd know that you're yielding on something when the command comes in.
 * Leon: One fix to yield-waitfor could be completely changing the semantics: yield-waitfor could be synchronous and cause the next command to block. But commands always return right now, and have possible failures too. Could break a lot of drivers
 * Brad: Checking, we merged yield-waitfor in 2024 https://github.com/tock/tock/pull/3577 and last released in 2025 https://github.com/tock/tock/releases/tag/release-2.2 So this is in a release now
 * Brad: So, if yield-waitfor doesn't work without the upcall, I'm trying to remember how we were imaging this would look. We could go back and say "you do have to register the upcall", could be real or some sentinel.
 * Leon: So we haven't seen this before because a command doesn't end a timeslice, so assuming those system calls are right next to each other in the app, this would only occur if there's a timeslice between the two?
 * Brad: Any interrupt occurring between the command and the yield-waitfor.
 * Johnathan: Theoretically you should be able to get this to occur by connecting two GPIOs together, and triggering one with a command which will happen before the yield-waitfor occurs.
 * Branden: We also haven't written a lot of code using yield-waitfor
 * Brad: I'll raise an issue about this all. I'll block that PR.

