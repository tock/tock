# Tock Core Notes 2022-09-30

Attendees:
- Alexandru Radovici
- Amit Levy
- Brad Campbell
- Chris Frantz
- Jett Rink
- Johnathan Van Why
- Leon Schuermann
- Pat Pannuto
- Phil Levis

## Updates
 * Phil: In the OpenTitan working group, we had a long discussion about app IDs
   and short IDs. Brad and I converged, and explained the mechanisms to
   Alistair. Need to make a few tweaks, e.g. at boot it does not check the
   uniqueness of short IDs, and we need to modify the process ID trait so you
   can get short IDs from process IDs.
 * Jett: We just found an issue in the timer driver when you have
   quickly-scheduled timer events. We will open up an issue or PR about it soon.
 * Phil: Is the issue when you have multiple userspace timers multiplexed on the
   same alarm?
 * Jett: No, it's on the kernel side. If the loop that looks through the linked
   list takes long enough that a timer has already expired, it will miss an
   event.
 * Phil: If I recall correctly, there should be something in the code to handle
   this case. Something like "if this will fire soon, then fire it now".
 * Jett: Yes, but there's still a race. We've seen it, and fixed it locally.
 * Pat: There's an issue that's come in about the HiFive1's stack usage. Gabe is
   going through some work to figure out what happened and how to address it.
   We'll want to see if any of the initialization component stuff is affecting
   our stack usage.
 * Amit: What's the stack usage now?
 * Pat: On HiFive1, it was limited to `0x900` and is now blowing up to
   `0x1200`. Hudson did some work to make sure that `static_init!` wasn't pulled
   onto the stack, and that didn't make it to HiFive1. Also a lot of the TBF
   parsing code might live on the stack for longer than it needs to.
 * Amit: How much memory does that chip have?
 * Pat: 16k, which is why we didn't do much with it.
 * Amit: For reference, `0x1200` is something like 5k, so small but not that
   small.

## Dependabot and tock-teensy
 * Amit: I see Phil has responded, it looks like we can archive tock-teensy.
 * Phil: I don't know anyone using and it's out of date. Yes.

## Updating Cortex-M context switch code (#3109)
 * Brad: I think this is something we should update for Tock 2.2. Subtle, so
   needs eyes on it, want a robust solution. Want more eyes to look at it. Can't
   reliably test it because symptoms could go away randomly due to unrelated
   changes. Pat and Amit, you've thought about this -- I think we should
   consolidate what we know so we're not repeating steps we've already done. Can
   we sketch out what should be happening then translate that into assembly, or
   how do we move forward?
 * Pat: I thought I commented on the PR but just checked and don't see it.
   There's a book that describes how to properly do syscalls on ARM and why, as
   well as how to correctly read vector control status registers. I will find
   that and post it on the PR this afternoon. 
 * Amit: We should use PendSV to switch to user code. Looking at early history,
   we were doing that, but now we're abusing `svc` to go both directions which
   is a bit of a mess. There is a more well-established way of doing this on
   ARM.
 * Pat: The other comment I'll bring up on reproducibility -- we could replicate
   our interrupt interleaving by throwing some busy loops into interrupt
   handlers.
 * Leon: Is there a way we can reproduce this in a well-defined manner? Maybe
   emulation with an instruction trace? Want to be able to reproduce the edge
   case after the fix and verify it works.
 * Pat: I think the answer is probably yes.
 * Amit: Research idea I've been playing around with that is applicable to this.
   I worry that one issue is that because this is happening on ARM, we don't
   have the same kind of tools as we do for RISC-V to do that stuff.
 * Leon: What I'm getting at is if there is a sufficiently-accurate emulator, I
   would be happy to look into this and try to reproduce it.
 * Amit: Maybe with some permutation of QEMU.
 * Leon: I guess so
 * Amit: There's also an undergraduate starting her senior thesis looking at
   bottom halves, kind of similar to this issue.
 * Amit: An issue with PendSV is that it is an asynchronous instruction. All it
   does is set the bit for the interrupt to occur, and that's not necessarily
   unworkable but it doesn't work well with our current schedulers. That doesn't
   mean it's the wrong thing to do, but it's a heavier lift than just changing
   the assembly.
 * Pat: I'll have to think more
 * Brad: Are these separate issues? Can we work on the register-stacking
   separate from the context switch mechanism?
 * Amit: Yeah, probably. It seems like there's enough motivation for several
   people to form an informal working group to try to figure this out on the
   side.
 * Brad: I think this is really important, yeah.
 * Pat: Yeah, that seems fine.
 * Amit: Slack channel being created as we speak.

## Notifying drivers about Subscribe/Allow syscalls (#3252)
 * Alexandru: The problem we are facing is the drivers have no idea if a buffer
   is swapped underneath them. This was not a problem before Tock 2.0, but now
   it is. The best example in upstream Tock is the touch driver. If you have
   multiple touches, the driver receives a stream of touches and writes them
   into the buffer. The application would read the data, then send a command to
   acknowledge it to the driver. Now the application needs to swap out the
   buffer, access the data, then swap the buffer back again. This requires an
   extra command. With my proposal, when the application swaps a buffer, the
   driver gets a notification. The other use case we have internally is the CAN
   driver, where we need to swap buffers fast. There's currently no way to tell
   if an application has finished with a buffer and swapped it. We need to know
   when the application has accessed a buffer and placed a new one there.
 * Phil: Can you walk through those examples again? In the CAN bus you're
   streaming data into the buffer -- can you walk through it again?
 * Alexandru: In the touch driver, I have a buffer, the driver receives
   notifications of multiple buffers, and fills the buffer. When the driver
   receives more events, it can either overwrite the buffer or drop events.
   Before Tock 2.0, it would fill the buffer then drop every event until the
   application sends a command acknowledging it has consumed the data in the
   buffer. Without this, the application could be in the middle of consuming
   data as it is erased. In Tock 2.0, the application cannot read the buffer
   while it is being shared. Now, the application needs to swap the buffer to
   read it. The driver has no way of knowing this, so it needs an extra "hey
   I've consumed the buffer command". The optimization here is for the driver to
   assume the process has consumed the buffer.
 * Phil: Basically you have to do a system call to swap in the new buffer, which
   could tell the kernel that it can begin writing to the buffer, but that
   notification is currently absent.
 * Alexandru: In the streaming driver we use multiple buffers. We can tell
   whether one is shared but not whether it was consumed. Currently have an
   extra Command call.
 * Phil: So the basic use case is streaming data.
 * Alexandru: I am pretty sure this will appear for any streaming use case.
 * Phil: The driver needs to know where the tail of the stream is.
 * Alexandru: Exactly
 * Phil: It's not that you are concerned it's been swapped, you just need to be
   notified that there's a buffer so you can reset your index to zero.
 * Alexandru: Exactly
 * Leon: I've looked at this before, and I am sympathetic to this change. Tock
   2.0 requires the application to swap buffers. On the other hand, I think we
   should still document that the notification is not sufficient to ensure
   buffers don't change. I fear that developing this notification will lead
   developers to rely on the assumption that this notification is required for
   the buffer to not change.
 * Alexandru: It doesn't change anything in the guarantees.
 * Leon: I'm trying to think in terms of developer expectation management.
 * Alexandru: So add a comment stating that this does not guarantee that the
   buffer changes.
 * Leon: Not a criticism, I just want to reiterate that we don't change those
   guarantees.
 * Leon: I would also be interested in the runtime overhead and code size
   impact.
 * Alexandru: I can check runtime overhead, but on ARM I always get exactly the
   same size for the kernel text segment. I think I'm encountering some
   rounding.
 * Amit: Yeah, it may be allocating or counting in chunks of 4kB or something.
 * Brad: What's the issue with using Command?
 * Alexandru: It generates several additional system calls.
 * Brad: Is that too much?
 * Alexandru: Probably if it's high speed.
 * Phil: I'm less concerned about the speed cost, mostly concerned about the
   semantics. Can we imagine a case where userspace wants to change the buffer
   but wants state the driver is keeping not to change? For example, I swap the
   buffer, but the kernel still writes to the location it was before. It seems
   like a nice clean thing for the driver to automatically know to update its
   state.
 * Leon: That depends on whether the driver is incorporating such functionality.
   If the driver wants to support functionality which says "a packet must not be
   located at the start of the buffer but can be located elsewhere", then if the
   driver wants to give these additional options it still can opt for the
   solution with the explicit command. In terms of network drivers, there's very
   seldom the use case where you want to mutate a buffer but not send the packet
   at the start. Could be an efficient streaming mechanism for network
   interfaces.
 * Alexandru: That is exactly our use case.
 * Brad: One of the advantages to internalizing the Allow implementation is that
   capsules don't get a callback when buffers are shared. To me, that is
   intentional, because we had early capsules where that made sense. "I got that
   buffer, lets do something". By removing this, we made it impossible for
   capsule authors to do that, and all actions must be a result of Command. By
   adding this callback, we are re-introducing this capability. Do we have a
   definition of what this callback is allowed to do?
 * Phil: That's a good point. We had some early capsules that did that, and we
   realized that it is not a way to do things. However, that is not why we moved
   Allow to the kernel, and that was a side effect.
 * Brad: I think this is a very nice side effect -- it makes code review easier.
 * Phil: Think about the state drivers are keeping -- indexes, sizes.
 * Brad: I'm not saying we shouldn't consider it. I'm saying that if we don't
   talk about this, then we're re-opening an issue that we had previously
   closed. We should discuss what we will approve, because it shouldn't be
   something like issuing an I2C write.
 * Phil: Any definition we can come up with, I think we can come up with a
   counterexample. May be more clear from userspace, in the context of streaming
   operations. One limit is you can only get upcalls in response to Commands.
   Swapping a buffer should not start an operation, but in a case like streaming
   it could involve continuing the operation.
 * Leon: I'm taking issue with the statement that upcalls should only happen in
   response to Commands. That's not currently true; a lot of mechanisms can
   schedule an upcall.
 * Phil: This is not a statement of "anything we state is true has to be
   statically verified".
 * Leon: Are we enforcing that? I haven't heard of that paradigm before.
 * Alexandru: The touch driver currently sends upcalls without needing a
   Command.
 * Phil: So it sends when you register -- there's no command to say "I want to
   receive events"?
 * Alexandru: You just register.
 * Phil: So it's automatically activated?
 * Alexandru: If you have activated the receipt of events but don't subscribe,
   but then subscribe, events will start coming in.
 * Phil: That's the point. Of course if you activate it without having an
   upcall, then you won't receive upcalls until you subscribe. If you do a
   subscribe without doing a command, will you receive an upcall?
 * Leon: Oh, I thought you were saying that one upcall has to correlate with one
   preceding Command call.
 * Phil: No, e.g. GPIO. It shouldn't be that I Allow a buffer and I receive an
   upcall that the buffer was transmitted.
 * Leon: This is kinda pedantic, but we can emulate that for now. A driver could
   iterate over all buffers for every app.
 * Alexandru: We could change this PR by adding a bitstream within the grant.
   When an application swaps a buffer, set it to clean, and when a capsule
   accesses the buffer, set it to dirty. This will add some space requirement to
   the grant. This will be enough -- you can enter the buffer and see if it is
   dirty or not. If clean, you can just reset the index. In this way, the driver
   would never receive a callback, but would still know if it is a different
   buffer than it saw last time. I think it would work.
 * Leon: I would be hesitant because it makes things so much more implicit.
   Behavior would become much less predictable. A notification mechanism is easy
   to understand. 
 * Alexandru: This is why I chose the callback instead of this.
 * Brad: I do kind of like that idea but probably not enough -- the clearing
   would be very implicit and potentially confusing. The general idea that this
   isn't a closure-style callback where you can do anything arbitrary but is
   instead a notification channel is nice. It scopes it down to what it is
   intended to do, and doesn't increase capsule-writing complexity.
 * Alexandru: Okay, but it will take more space in the grant.
 * Brad: Sure, yeah. I'm not saying this is the implementation we need to go
   with. I am advocating for avoiding designing a generic mechanism; rather say
   "this is what it is" and the implementation is the current implementation.
   Could change the implementation in the future if we find a better one. Want
   to reserve the ability to change the implementation. Are we setting the
   parameters where we want them to be? We should be explicit: is this new
   functionality that we want people to experiment with, or is this meant for a
   specific use case and may change in the future?
 * Leon: This mechanism prevents the one use case of avoiding another Commnd
   call. I'm fine with that, but it would make this impossible.
 * Brad: In #3252, it does not look like any event is triggered. Is that because
   it is not the same case in the pull request? It just sets a flag, it does not
   trigger an event.
 * Alexandru: It just sets a flag, the ACK flag.
 * Brad: So it's not triggering an event?
 * Alexandru: No. The previous driver had an extra Command which was
   acknowledging because before 2.0 this was not a problem.
 * Brad: I'm saying to Leon, is this PR not capturing what you're talking about.
 * Leon: Yes, so I was specifically referring to the motivation that Alex was
   talking about at the beginning of the discussion. I got the impression he was
   talking about streaming data, where the transmission of data was initiated by
   this transaction.
 * Alexandru: That was not my intent. You start reading from the network, you
   have a big buffer, application will swap it out, don't know when to reset.
   Position 100, application just swapped it, have no idea to know should reset
   to 0. Problem is I'm getting out the buffer -- it's a race.
 * Leon: I get that. It was just a misunderstanding on my side -- I thought you
   were thinking of the case of starting a transmission on Allow.
 * Alexandru: Command is not sufficient, it is not atomic. I can swap out the
   buffer, and before I issue the Command the driver can continue to write to
   the buffer.
 * Brad: Can we scope this to only the grant? The use case is we want to modify
   our grant state.
 * Alexandru: The only way I see to do it is to add extra bits to the buffer.
   The counter Leon proposed would add a lot of space. I think one bit would be
   enough -- indicate "swapped since last use by the capsule".
 * Leon: Regarding scoping, the immediate problem is we don't know the type of
   the grant in the handling logic. Would need to do dynamic dispatch in the
   driver to invoke a method on that type.
 * Brad: That is definitely a challenge. I think we've converged and can
   continue this discussion on the pull request.
 * Alexandru: Sure.
 * Leon: Sounds good to me.

## `static_init!` update
 * Brad: Now that the `static_init!` PR has been merged, I have been updating
   capsules. I've done X. Don't know how to do it, as we have about 60
   components. Do you want one big PR, maybe split some out if they're not rote?
 * Amit: Maybe all the rote ones in one big PR, and do harder ones in separate
   PRs.
 * Phil: Does it need to be atomic?
 * Brad: No, independent changes.
 * Phil: I'm of the opinion of preferring larger PRs, with notes saying what is
   rote and what to pay attention to. I'm happy to not require that because I
   know I differ. Unless you don't want to have to do all of them.
 * Brad: I will open a pull request with the straightforward ones. Can take a
   look and see if you agree with my changes. Some of them are "we need to do
   this" and some are for consistency.
 * Pat: I thought we were letting those sit because you and Hudson were
   discussing a change that was getting made to components which would bubble up
   through all your updates. Are you done with that?
 * Brad: Yes, it was all bundled into last week's PR.
