# Tock Core Notes 2021-01-22

## Attending
 * Alistair
 * Amit Levy
 * Arjun Deopujari
 * Brad Campbell
 * Gabe Marcano
 * Hudson Ayers
 * Johnathan Van Why
 * Leon Schuermann
 * Pat Pannuto
 * Philip Levis

## Updates
 - Gabe: We have started benchmarking Tock on RISC-V platforms. We have selected
   earlgrey OpenTitan on the NexysVideo FPGA board. I have built the bitstream
   on my own and can load Opentitan on the board and my next step is running
   Tock on it. I noticed OpenTitan enabled i-cache -- should I leave it enabled
   or turn it off for benchmarking?
 - Amit: How large is the i-cache?
 - Gabe: From what I remember, 4kB in size.
 - Phil: Is it a speed optimization or a power optimization?
 - Gabe: I don't really know the rationale. From what I've read on the
   repository I believe it is because they expect flash to be significantly
   slower than SRAM. I don't think that is the case at the moment, but I think
   it is what they expect it will be.
 - Phil: I suggest benchmark it both ways. If you see a performance difference,
   turn it on, because that is the expected operating mode.
 - Amit: Pat, I wonder if that accords with your intuition on things. I thought
   on our ARM platforms both flash and SRAM are 1 or 2 cycle fetches.
 - Pat: Most of the cores we work with are single or double digit MHz, they are
   looking at 100's of MHz. They are pushing low-resource microcontrollers into
   a higher-performance regime.
 - Phil: Many of the microcontrollers we have effectively have RAM prefetch
   buffers for instructions.
 - Pat: A Cortex-M3 has a 3 word instruction buffer, which is a lot smaller.
 - Phil: That doesn't matter if you're consistently hitting it. Cache hit rate
   is what matters.
 - Pat: It's interesting to me as an architect that an i-cache and SRAM are not
   terribly different in structure, so on a machine that has only 64k of RAM in
   the first place, which would you sacrifice that to a fixed function
   structure?
 - Phil: Because otherwise you have to carefully map in instructions yourself
 - Pat: Yeah but 4k worth?
 - Amit: Hardware people want that sweet operating systems money.
 - Phil: If you have 64k of RAM then 4k more is not a big deal. If your
   processor runs 4 times as fast, that's an easy calculus.
 - Gabe: I can try to benchmark both. I expect to ask in Slack for where to
   enable and disable i-cache. I think the OS can toggle cache on and off.
 - Amit: Gabe, do you have access to the earlgrey people?
 - Gabe: I know people here at UCSD working with it, but I don't think they're
   developers working on it.
 - Amit: I think you should get in touch with them, they're fairly open. I'm not
   sure anyone here knows how to enable and disable i-cache.
 - Gabe: I know how to enable and disable i-cache. Can do it through System
   Verilog or through a control register on the Ibex CPU.
 - Amit: A request I have -- IDK how annoying this would be. I now have one of
   these Nexys Video boards. Building the bitstream is annoying. It would be
   useful for the Tock community if you would post a binary occasionally.
 - Gabe: Sure, I can do that.
 - Alistair: OpenTitan hosts a copy of their bitstream. I have never built the
   bistream.
 - Amit: I take back what I said. I should ask for help asynchronously about
   that.
 - Alistair: The README will tell you how to download it.
 - Gabe: Installing Xylinx tools is really annoying. Took finangling on
   non-Ubuntu OS. Successfully got it to build.
 - Amit: Any other updates?
 - Brad: On the Tockloader front, I've been experimenting with autodetecting
   both Jlink EXE and OpenOCD boards, so you don't have to explicitly pass in
   the loader. Tockloader will try to detected that. If you have a board you
   would like it to work on I'd be interested to keep working on that. In latest
   master, not a released version. Works for NRF52840-DK, Microbit v2, and a
   couple other boards.
 - Amit: Cool, how do you do that? Guess and check?
 - Brad: Yeah. Jlink EXE gives us a way to view the connected devices, OpenOCD
   doesn't. Have to configure it per-board. If you see this signature, it means
   a particular board is attached. It's fast enough to not change perceptible
   performance.
 - Amit: Actually flashing something is slow enough you should be able to mask
   that.
 - Brad: Right and you only have to do it once at the beginning.

## 2.0 Roadmap
 - Amit: What do we still have to do? We do we start testing? How do we do the
   merge? What do we do after that to release?
 - Phil: We have 5 remaining capsules, but Amit may have done some of them just
   recently -- I see pull requests. I may pick up 1 or 2 if necessary. exit is
   merged, yield updates done. Question is there anything else that is a
   blocking item before we say the `tock-2.0-dev` branch is ready to merge into
   master?
 - Brad: I have a question about that. Hudson, for the ReturnCode/ErrorCode
   switch, when do we need to do that? Have we settled on a plan?
 - Hudson: I don't think we need to do the switch before we release Tock 2.0 in
   the kernel. Currently, callbacks use the old ReturnCode, so in theory we can
   wait until we get rid of ReturnCode to do the switch. I am intimidated by the
   amount of effort required to check `libtock-c` when we do the switch.
 - Brad: Do we run the risk we will never make the switch?
 - Hudson: Perhaps
 - Phil: Yeah that's always a risk, but if we're committed to doing it there's
   not much risk.
 - Hudson: I think it is important and will make effort to not forget about it.
 - Amit: Can we summarize what the plan is? Currently we're passing ReturnCodes
   in callbacks to userspace. Where do we want to get to?
 - Hudson: Passing ErrorCodes instead of ReturnCodes in callbacks. If you want
   to pass success, callbacks could -- by convention -- pass 0. The easy way to
   do this would be to negate all values received in callbacks in `libtock-c`
   and change the returncode values to the negatives of the ErrorCode values.
   Then you have different ErrorCode values in `libtock-c` from the kernel which
   isn't what you want. The "correct" [sic] way to do this would be to change
   libtock-c to not rely on error codes being negative. Brad and Leon seem to be
   of the opinion we should do that. Should check if result equals 0 to check
   success rather than result less than 0.
 - Leon: We don't need to get rid of the "errors are less than 0" paradigm
   throughout `libtock-c`. Just need to change in the callback handlers
   themselves. We can convert it to the native `libtock-c` representation using
   a helper function and pass it downwards.
 - Brad: I agree with that. I've changed my mind. I think we should follow the C
   convention of errors are negative. I do think we should separate return
   values from error codes. We want to keep that part of the Tock change so you
   can't pass through both a value and an error in the same thing. I don't think
   we should go against standard C convention.
 - Hudson: That means most functions, in order to return two items, will need to
   pass a pointer in that is set by the callback or something like that.
 - Brad: Yes. There shouldn't be too many of those.
 - Amit: This is blocking for a 2.0 release, right? Not for merging, but for the
   actual release.
 - Hudson: I think that's a good question for everyone else.
 - Brad: If we're going to do it, yes. Otherwise, we should just say we don't
   care.
 - Amit: It's a pretty meaningful change to the userland API. From a semantic
   versioning perspective, it would be very bad for apps that run on 2.0 to
   break on 2.1.
 - Leon: I would have argued that way too. I don't think the ABI is simpliy
   defined as to how values are encoded but also what values are encoded, and
   this changes every callback throughout the kernel.
 - Amit: Generally our policy is that minor version kernel updates should not
   break most existing apps that use well-established capsules. This seems like
   a change that would cause significant breakage.
 - Brad: That's what I'm concerned about -- it will be a lot of churn and
   confusion.
 - Amit: That seems like a blocking issue. Do we agree we don't need to block
   merging the `tock-2.0-dev` branch on it?
 - Hudson: Definitely yes. We should merge `tock-2.0-dev` as soon as possible.
   We can block a Tock 2.0 release and `libtock-c` on that.
 - Leon: If we are finished with this discussion, there's one other thing we
   would need to block merging Tock 2.0 on. The change that Brad has done a lot
   of work -- getting rid of stack pointer tracking -- actually making the
   switch to giving apps the least space possible. This will affect
   kernel:userspace behavior a lot. I suppose we want to do this as part of the
   atomic switch.
 - Brad: I agree. I think every open 2.0 PR right now should be closed before we
   merge.
 - Hudson, in text chat: We should also block on removing SuccessWithValue,
   which as of now is only blocked on the sd card capsule being ported
 - Phil: I think that makes sense, as long as there aren't things that are
   wrongly labeled.
 - Amit: Right.
 - Johnathan: Leon, you have a PR to further specify the semantics of
   `subscribe` around swapping callbacks. Does that PR represent the semantics
   you plan it to have? I know there was still the question of enforcement the
   last time we spoke.
 - Leon: The story behind that is it gives me headaches to think about it. I've
   been working on it for a few days now. There are so many options, none of
   them are non-intrusive. There's a lot of changes to the kernel. I guess --
   reading the TRD again -- my changes would be pretty mechanical and just give
   additional safety guarantees with regards to capsules being not able to swap
   callbacks of different processes. In a further stage of development, capsules
   not being able to swap callbacks of the same process. There are a lot of
   variants we can go in, but I don't think we should block Tock 2.0 on this
   because it would not affect the Tock 2.0 ABI at all, just the potential
   damage capsules can cause.
 - Johnathan: It does affect the semantics, though, and parts of the design of
   `libtock-rs`.
 - Leon: As far as I understood the discussions, userspace should assume a
   capsule does not swap AppSlices or callbacks.
 - Johnathan: If we make that assumption then the core kernel has to enforce it.
 - Phil: I would push back on that a little back. We want userspace to be able
   to assume those things don't happen. It is then a policy decision within the
   kernel to decide whether the kernel enforces it. Absolutely, we want to have
   a version of the kernel that does, but
 - Johnathan: If userspace relies on that for memory safety then not enforcing
   it would be a threat model violation.
 - Leon: So I'm really in favor.
 - Phil: This sounds like something that's really good to discuss. I think the
   short answer is we want to have a kernel that enforces it. I think we agree
   on that.
 - Leon: I guess my story is that for the next 2-3 weeks I'm pretty busy so I
   don't want to block Tock 2.0 on this. On two calls now we've reached the
   agreement that if the cost is not enormously high -- and I have proof of
   concepts that are essentially zero-cost implementations -- then we want to
   enforce it. It's just a question of getting the code right and doing the
   changes in a non-intrusive manner. Is it sufficient for you to say that we're
   going to fix it in a few weeks, maybe prior to releasing to Tock 2.0, but not
   blocking the merge on it.
 - Johnathan: I'm okay with pushing it off until after the branch merge, but if
   we might push it off until after Tock 2.0 then I want the TRD to reflect the
   kernel's actual behavior. I need to know that so I can make `libtock-rs`
   compatible with it.
 - Leon: That makes sense.
 - Johnathan: This next week, I'm going to be writing some of the components
   that depend on that corner of the behavior. If I don't know what that is, I
   will have to go back and fix them later, and I'd rather not do that.
 - Leon: Sure. I can push something out in the next few days, I have something
   working. Hudson did a review of a previous iteration of this and he wasn't
   too happy about the implementation. I'm still looking for ways to convince
   others that this is the way to go. At least then there's something out there
   that works.
 - Johnathan: It sounds like I should assume we don't have enforcement for now
   and then wait and see.
 - Leon: Maybe we can have a discussion via email?
 - Phil: This sounds like a good discussion to have on tock-dev. Johnathan,
   there's the question of what does "it's enforced" mean? One approach is to be
   sure the code is correct, versus reading the code and checking no capsule
   does that. What constitutes guarantee, because we're not doing formal proofs.
 - Johnathan: Yeah, the untrusted capsule isolation is "capsules are allowed to
   do anything the Rust type system allows them to do". Maybe they should behave
   differently, but as far as the userspace it is not acceptable from a threat
   model perspective for a userspace app to lose its confidentiality or
   integrity guarantees if a particular capsule decides to misbehave in a way
   that works under the Rust type system. If I write the userspace in a way
   where doing an incorrect callback swap results in undefined behavior within
   an app, that is a security issue in that a capsule could trigger that
   behavior and the kernel doesn't catch that.
 - Phil: That makes sense. I'm much more worried about app data than the
   function pointer.
 - Johnathan: In particular it matters in the case where the app data is a
   pointer to a structure on the stack.
 - Amit: Yeah
 - Leon: I guess in summary I take away from this thread that I open an issue
   that outlines the thoughts I've been having over the past few weeks and all
   the guarantees we should be making and why that is difficult. Put out one or
   two implementations so we have rough PRs to look at how it should work.
 - Johnathan: If anything, I'm getting the impression that the kernel should not
   make the guarantee, that it would be expensive in terms of code size.
   Userspace runtime can just do it, which doesn't seem that expensive to me.
 - Leon: It's not expensive, just inelegant. Can discuss this for a long time
   but seeing the code will be more insightful.
 - Amit: A shortcut here might be that if userspace enforces it, and it's not
   too expensive for userspace, then if the kernel adds enforcement to it it
   won't break userspace. Userspace can remove checks later.
 - Leon: This is the point where I'm confused. I assume this is going to have
   large implications for userspace. When I think about async code and
   callbacks, if we don't implement this enforcement then callbacks will need to
   be registered for the lifetime of the application because we cannot guarantee
   that a capsule will never call it again in the currently state.
 - Johnathan: Asynchronous callbacks will be mostly be static in `libtock-rs`,
   it's the synchronous ones that are problematic because they get to reference
   the stack. Then `libtock-rs` has the control to not return until the callback
   is done and so it can make sure to unregister the callback before the stack
   frame disappears. This impacts the design of the unit testing support, where
   I will have a `FakeDriver` trait -- a parallel version of the kernel's
   `Driver` trait. That trait will need to represent exactly what capsules are
   capable of doing.
 - Leon: It's a lot to think about, so I suggest I open the issue and we can
   talk about it offline.
 - Amit: I think that's the right place to discuss it.

# Tock 2.0 Merge and Post-Merge plans
 - Phil: Once we think the branch is ready -- have ticked off all the boxes --
   what will be the merge process? Merge master in, test, and go, or something
   else? Talking and agreeing now seems better than everyone having different
   ideas.
 - Amit: The diff will be big. How do we want to deal with that? Do we want to
   review it, or do we have high enough confidence in the review process that
   went into the tock-dev branch?
 - Brad: I absolutely want to review it and am hopeful that most of the changes
   are in capsules that have been reviewed independently. I'm mostly concerned
   about kernel changes that are hopefully small enough to review thoroughly.
 - Leon: I had two thoughts on previous calls. For one, I think for other open
   source projects it is advisable to have an integration branch -- the merge
   result -- which we already have on `tock-2.0-dev` when merging `master`.
   Snapshot of one merge that can be thoroughly tested and iterated
   upon, so we're sure the merge doesn't break anything. Second thing is we
   should review all the changes, including capsules. Many changes skimmed over
   just looking at Tock 2.0 correctness, so issues may have slipped in.
 - Phil: I think we may run into trouble with old capsules that are fast and
   loose with semantics. You look at the code and what it does is clearly not
   correct -- many unhandled edge cases. We could enter this path of fixing all
   the bugs, which could be a lot.
 - Amit: That is too high of a bar. We have talked about having an experimental
   submodule in capsules where we could move things -- for example the signpost
   drivers that are not actively maintained and a few fall into that category.
   They're not multi-process, they do old things we now consider wrong. We
   should not block on fixing those. Brad's perspective seems right that the
   stuff that is relegated to capsules is not as big a deal for merging as stuff
   in kernel or arch. Those seem like the really important stuff to review that
   probably didn't get a good review except from Phil and Leon in the first
   place.
 - Phil: I think that's totally right and I agree with Brad. We should go over
   the kernel and arch stuff with a fine toothed comb, may save big headache
   later if there is a bug.
 - Amit: I looked at it and it's not a lot. The main changes are the
   `ReturnCode`/`ErrorCode` transition.
 - Phil: Leon structured it so it's a very surgical change.
 - Brad: Phil, to get back to your question, my preference would be to do the
   review on the Tock 2.0 dev merge, but not too much testing. Merge that, tag
   as 2.0-rc1, then stay on that for an indeterminate amount of time as we
   resolve other issues. We'd be clear that userspace is broken. Then make
   2.0-rc2, start normal release cycle including testing then. That way we don't
   double our testing effort for the same release.
 - Amit: I agree.
 - Phil: Seems reasonable to me. We have been testing on the individual level.
   Haven't tested things like multiple processes.
 - Amit: Okay, post merge plans? Sounds like post merge we want to:
   - Wait for the `libtock-c` changes to callback arguments
   - Test
 - Amit: Are there other things I am missing? Potentially this change that Leon
   has been suggesting.
 - Brad: Update documentation across the book. Docs in the kernel + wherever
   else we have written things down.
 - Leon: Yeah, that's going to be a lot of effort.
 - Hudson: Are we going to block on both userspace libraries being 2.0
   compatible?
 - Amit: I think we need one for testing.
 - Hudson: Agreed
 - Amit: I think it should be `libtock-c`, because that one has the most
   extensive coverage still. `libtock-rs` might end up being more expedient and
   better but we would be lacking drivers we want to test. Is that fair,
   Johnathan?
 - Johnathan: Yeah, that's fair. I'm probably only going to implement drivers
   that have documentation in the `doc/` folder, which is a small fraction of
   the drivers.
 - Alistar, in text chat: Why not block on both?
 - Amit: My suspicion is `libtock-rs` will probably be done faster anyway, but
   to my mind it doesn't seem necessary to block on it. It's fine for userspace
   to take longer because we still have 1.0 versions.
 - Johnathan: I think it'll be done by the end of March. I don't know if that is
   sooner.
 - Amit: Oh
 - Johnathan: There is a lot of work to do in `libtock-rs`.
 - Alistair: `libtock-c` won't test everything. HMAC and CTAP are
   `libtock-rs`-only.
 - Amit: CTAP is hard to port to `libtock-c`, especially in a way that is not
   testable. I suspect HMAC is pretty simple.
 - Alistair: Yes
 - Amit: Johnathan, if you had help, how much faster would it be?
 - Johnathan: The biggest part is putting all the building blocks in place that
   I have already designed. Actually writing the bulk of the drivers isn't
   terribly hard. I can sit and diagram all the individual pieces that need to
   be rewritten and come up with a timeline, but so far all my estimates have
   been very poor and optimistic.
 - Amit: How much of what you expect you need to do still is relatively easily
   delegated and is parallelizable?
 - Johnathan: Once I have `libtock_runtime`, `libtock_platform`, and
   `libtock_unittest` mostly complete, the rest is parallelizable. Those three
   parts are very complex. What I'm missing is:
   - ARM support in `libtock_runtime`
   - System call implementations in `libtock_platform`
   - Entire `libtock_unittest` framework
   - A way to write integration tests. Not fully designed -- may be able to lift
     that from tock-on-titan (could be outdated). May need a tock 2.0 library to
     put the tests in and test infrastructure.
 - Johnathan: There's a lot there that I've already thought through and I don't
   think others should have to think through, but that serializes things on me.
 - Hudson: This feels like a problem we can push back, as this is not the only
   remaining blocker. Johnathan can watch for problems to share with others.
   Keep pushing on remaining kernel stuff and see if it's an issue.
 - Johnathan: Once I have a working console driver the remaining drivers can be
   parallelized, but that's like the end goal. I can talk to Alistair about
   porting CTAP, as I don't want to do that.
 - Amit: I like Hudson's suggestion of punting on the decision. It's critical we
   have `libtock-c`, it's also pretty close. We've been porting stuff over to
   `libtock-c` as the kernel's being written anyway, and sorta been testing.
   Alistair has a good point that there are a few drivers that we want to test
   that would be hard to do in `libtock-c` and it would be nice to be able to
   move them all in lockstep.

## TBF Header Permissions and Persistent Access Permissions
 - Alistair: The KV store stuff is getting to the point it is stuck on the
   permissions interface. I was wondering if people could review the header PR
   ([2172](https://github.com/tock/tock/pull/2172) and
   [2177](https://github.com/tock/tock/pull/2177)). Seems to line up with what
   Johnathan was proposing. I'm not sure if people have read them.
 - Amit: A practical issue is that many of us have been focused on Tock 2.0
   ports, which have been blocked a while. Which isn't to say this is not
   important.
 - Alistair: If no-one has any comments we can
 - Phil: We need time to look

## Flash HIL discussion
 - Brad: The one thing we've talked about is maybe we need two HILs. One for
   internal flash, one for generic flash. We can rely on capsules to reconcile
   them as needed. This issue keeps coming up and it doesn't seem there is a way
   to reconcile these two use cases cleanly.
 - Alistair: What do you mean internal flash?
 - Amit: The flash where the code is stored.
 - Phil: Synchronous flash.
 - Alistair: I have two HILs for sync and async.
 - Leon: The use cases are mostly the same. Every consumer should use the
   asynchronous HIL. We can make a synchronous HIL asynchronous but we can't
   make asynchronous synchronous without blocking. What's the point of the
   synchhronous HIL?
 - Alistair: I have two separate PRs. One improves the async HIL, the other adds
   a sync HIL. I think they're separate issues.
 - Leon: My worry is if I was implementing a storage driver, would I use the
   synchronous or asynchronous one?
 - Alistair: That depends on the hardware, which is not a good answer.
 - Amit: The cases where you should use sync are fairly narrow. For example, if
   there is code that dynamically adds or changes installed apps, that will
   always be on local flash. Maybe there it is reasonable to use the synchronous
   version.
 - Phil: I totally agree. You can imagine there is sensitive code that will
   always be on local flash, it is easier to check synchronous code. It is an
   edge case, but an important one.
 - Leon: This tells me that every other use case would use the async HIL. More
   important to stabilize first.
 - Alistair: Yes
 - Amit: That's the pull request Alistair sent.
 - Phil: I think it's hard to have a detailed and thoughtful discussion in five
   minutes when we haven't read them before. I suggest moving to next week and
   having everyone read first.
 - Amit: That's a good idea. Alistair, I think these are important things and
   they have been on my list to look into more deeply, but at least for me the
   Tock 2.0 stuff has taken precedence.
 - Alistair: I wanted to try to keep the ball going. Pat was asking about two
   interfaces -- an advanced and a basic interface. I couldn't picture how that
   would work.
 - Phil: You use one or the other, generally. Think read/write or ioctl.
 - Alistair: Wouldn't that mean capsules have to choose between features and
   hardware support?
 - Phil: Think of it as a generic, hardware-independent one and chip specific
   ones.
 - Amit: Pat, is that what you meant?
 - Pat: Not so much. The precedent for the word advanced is the UART. The goal
   was not to be chip-specific, more capability specific. Every flash you can
   access at page granularity, some flash you can access at finer granularity. I
   wanted to represent that in a hierarchical way. Whether your platform
   supports more efficient flash read and write can be represented.
 - Amit: The matrix here is if a particular hardware supports finer-grained
   access than page access it should implement both interfaces. Capsules should
   opt for the basic interface if they can use it.
 - Alistair: Can we have a capsule try the advanced interface first and fallback
   to the basic one if that fails?
 - Pat: Can imagine a translation capsule. You could implement the advanced
   interface in terms of the basic interface using an internal buffer.
 - Amit: I understood the question differently. Alistair, did you mean at
   runtime?
 - Alistair: Yes. For example, if a capsule wants to read a value from flash, it
   can try the advanced read first then fall back to reading the entire page to
   extract the information it wants.
 - Amit: I don't think that would be the intention. If a capsule can be written
   to only use the basic functionality then it should be written that way,
   otherwise use the advanced interface. Would need a translation capsule which
   Brad claims exists.
 - Pat: I would go further. Chip implementers should write the interface they
   want to use. When you create a board, you need to add the wrapper capsule
   over the chip driver.
 - Alistair: There would be a default implementation of the advanced
   functionality, that we would fall back to.
 - Pat: I think so
 - Amit: Would a reasonable name be page level and byte level interfaces?
 - Pat: Sure
 - Amit: What you're suggesting -- I think -- is that chips maybe implement one
   or the other.
 - Pat: Chips should be welcome to implement both if they can. A board author
   has to link a chip implementation to the capsule that uses it. If a capsule
   needs a byte interface not provided by the chip implementation the board will
   need to add the translation between them.
 - Amit: But these are hierarchical -- we don't expect a chip with the
   byte-level interface but not the page-level interface.
 - Leon: I would expect one to be a super-trait of the other.
 - Amit: Yeah
 - Pat: Sure. I don't know what a super-trait is, but it sounds like a
   reasonable notion.
 - Amit: A supertype, like UART and UartAdvanced, whatever it's called.
 - Phil: This sounds like a good path. I want to say flash is fussy, all sorts
   of edge cases for different instances of hardware. It is tricky to generalize
   in a complete wai. I know we have UartAdvanced -- I would have conceived of
   it as a SAM4L-specific API at the same time. Byte level access of flash for
   reads, you buffer, but for writes it gets weird.
 - Alistair: That's the problem. For a write you have to read the whole page,
   modify it, then write it back.
 - Phil: Wear levelling becomes a problem.
 - Alistair: Some chips only allow a certain number of writes.
 - Phil: Most chips allow more than one write to non-continguous regions. Erase
   cycles are a concern. Makes in-place writes tricky and bad. Only have
   hundreds of thousands of erase cycles.
 - Amit: We're out of time. I'm not really sure where we ended on this. We
   should prioritize looking at these before next week.
 - Phil: Sounds good.
 - Alistair: Sounds good
