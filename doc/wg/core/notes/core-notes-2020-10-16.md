# Tock Core Notes 2020-10-16

## Attending
 * Hudson Ayers
 * Pat Pannuto
 * Leon Schuermann
 * Amit Levy
 * Johnathan Van Why
 * Philip Levis
 * Guillaume Endignoux
 * Alistair Francis
 * Brad Campbell
 * Branden Ghena

## Updates
 * Brad: I have a student working on restartable apps, eventually over
   bluetooth. Working on interfaces and then will get feedback
 * Hudson: Could this eventually be over 6lowpan as well
 * Brad: Yep

## 1.6 Status
 * Amit: We have gotten coverage over almost every board, which is pretty cool
 * Amit: A few breaking tests, but seem to be PRs that fix most of these
   breakages
 * Brad: Sounds right, there are a couple of mine that are not merged. For stuff
   that are bug fixes trying to get it in quickly.
 * Brad: 2 questions I have - Are there any lingering larger issues?
 * Hudson: A pretty weird bug with UDP virtualization. Basically, there is a
   test that involves flashing 2 apps that both try to bind to the same port.
   One app binds immediately, the other after a 10ms delay. The one that delays
   asserts that it should fail. However I now see that this test fails unless I
   extend the delay to >100ms. This seems like a potential scheduling bug to me,
   but have not yet dug into it -- it could be something else, because there are
   portions of the test that happen before we get to this port binding portion that
   could be affecting the timing.
 * Brad: Any other issues like this that anyone is aware of?
 * Leon: Are all schedulers tested?
 * Hudson: Round robin on most boards, cooperative on a few, priority on one. I
   tested MLFQ separately with a few apps on Imix and baseline behavior seemed
   fine.
 * Amit: There is the Intel 8080 PR, but I think that can go after release
 * Amit: Other than that just small bug fixes, this BLE driver alarm fix, I
   think we can expect a release by next Friday?
 * Hudson: Well I think there is still some thinking to do on Brad's MPU PR.
 * Amit: Brad, can you describe?
 * Brad: Yeah, basically if a process reduces its memory claim and restarts, it
   immediately crashes because the kernel has moved the pointers back up and
   they are no longer in the MPU region. This happens both ways, but I only caught
   it on the reduction case.
 * Hudson: The additional problem is that we reuse the MPU config of the
   original process, and if additional regions were modified that still applies
   to the restarted process, which I think could lead to bugs. I looked at how to
   fix this, but it gets a little hairy. Probably better if someone more familiar
   with this code looks at it.
 * Brad: I suspect that a fix for this will not be too hard. I think we should
   block 1.6 on this PR. I'll make a motion that we release 1.6 when all the
   core checkboxes are checked, or Friday, whichever comes sooner.
 * Amit: Sure
 * Hudson: Sure

## USB
 * Amit: Now that we have Guillaume on the line, I want to talk about a few open
   questions for the USB stack
 * Amit: First, SAM4L support: is anyone using this? Some of the HIL changes we
   think we need will requires implementation updates, but noone seems to be
   using this and unclear if it is worth blocking.
 * Alistair: What are the proposed changes?
 * Guillaume: Max proposed packet size, I think it could be changed to 64
   everywhere, but I don't have hardware to check that. Second point: if we want
   to have lifecycle management in the HIL, we would also need to change some SAM4L
   stuff.
 * Phil: I think it is okay if we have leftover code that is not up with the
   latest HIL. I think its okay to just not implement the HIL for the SAM4L if
   it is not being used.
 * Guillaume: If we update the HIL, we have to somehow modify the code. Should
   we just remove that file from sam4l? Should we put new HIL functions as
   unimplemented and remove old ones?
 * Amit: My sense is that if we have a working stack for the nrf52, it would be
   fine from my perspective in the same PR to remove or delete from the
   compilation tree the sam4l USB stack unless someone wants to pick that up.
 * Phil: I think that is fair. If the HIL changes this makes sense, USB is non
   used and there are clear technical changes. If noone will pick this up, that
   is a sign it should not still be supported.
 * Brad: This seems like a strange argument, because the HIL should be hardware
   independent, lets write code when we have real use cases, with 2 stacks we
   can write a HIL for 2 platforms! We have great tests for the sam4l USB code, it
   would be a shame to lose it.
 * Guillaume: I just think that noone seems will to volunteer the time and
   resources to do this
 * Amit: In normal times I would volunteer to do this because I know the USB
   stack well, and send an Imix to Guillaume, but I can't do that well now
 * Phil: How did the HIL change?
 * Guillaume: It did not change yet, but I am focused on these lifecycle
   changes.
 * Phil: Can we not just make that a new trait, and then not implement the new
   trait for the sam4l?
 * Guillaume: Maybe, might complicate capsules. But this packet size issue is
   harder
 * Hudson: I think this is sort of hard to answer now without seeing the
   changes. I am guessing that if the required changes for SAM4L are small, I
   would be happy to make the required changes. I think a reasonable approach would
   be to propose the changes and implement them for NRF, and then start a timer on
   whether anyone is willing to implement the updated HIL for SAM4L. If noone will
   do it, we should probably delete the sam4l implementation and post a tracking
   issue. If someone will do it, great! But I don't think we can ask anyone to
   commit to that now when it is impossible to know the scope of work that would be
   required because the HIL changes do not exist yet.
 * Phil: I do not think it is reasonable to force all devices to use full speed
   over this issue. There are valid reasons to use low speed (cable length, low
speed device connected to hub, etc.)
 * Hudson: FWIW, tinyusb is widely used in embedded systems and does not really
   support low speed USB either because of many assumptions throughout the stack
of 64 byte + buffers. Low speed does seem like mostly a relic of the past

## USB lifecycle discussion
 * Amit: The next part of the USB discussion is whether we need lifecycle events
   -- or whether they would be useful to have in the HIL, and whether designing
   those lifecycle events is something that should block progress on the USB stack
   for now (and CTAP in particular)
 * Guillaume: Yeah so lets say you want to send a packet to the host, and then
   the USB disconnects and reconnects, maybe you dont want to send that packet
   at all because once you reconnect you have to set up all sorts of control stuff
   etc. So last week it was argued that userspace should not be aware of the
   connection status and the application should just try to send and recieve
   packets, but it seems to me that for the application to know about reconnect
   events is important to allow apps to reset themselves, rather than trying to
   continue a transaction that is no longer valid from the hosts perspective.
 * Phil: I think the challenge here is that we can separate out the low level
   USB from say CTAP or HID as its being presented to userspace. 
 * Phil: I think the conclusion from last week was that we do not need to block
   CTAP on lifecycle stuff, because it is not needed for this particular use
   case (getting disconnect events is not necessary to write a correct CTAP
   userspace application), though it is important for other USB profiles.
 * Alistair: Agreed.
 * Phil: Basically we need this anytime there is something with state sharing so
   apps can reset their state to match.
 * Guillaume: If you send a message that is split into multiple packets, but
   disconnect in the middle, what do you do on reconnect? Send remaining
   packets? Restart? Abort? Maybe this disconnect/reconnect use case is not super
   high priority. Another case for the HIL which I think is important is when you
   initialize the USB first on the chip it takes some time to boot up and be ready,
   if the capsule tries to send a packet before it is initialized, this causes
   problems in the current initialization.
 * Phil: key here is capsule cant send a packet! Host has to request a packet.
 * Alistair: Exactly
 * Guillaume: You have things like enable and attach, but if you call attach too
   early when the chip is not yet ready, you shouldn't attach now but should
   later. Currently we do not support this, which is why #2094 does not work on the
   nrf52 currently.
 * G: In Alistairs version it is possible for the capsule to enable and attach
   even when the chip is not ready
 * Amit: I want to pop up, not sure there is really controversy here. It seems
   CTAP does not need to block on these changes to the HIL, because the relevant
   stuff from CTAP is the higher level stuff, and it seems that including lifecycle
   events in the HIL will be relatively simple (the implementations of the HIL will
   have to handle those things) and propogating them up to the capsule allows them
   to be used if needed, but they can also be ignored. This allows useful things
   like attaching the USB only when it is ready, etc. I think the only question is
   do we block on CTAP for this or do we do this as 2 independent tracks.
 * Guillaume: I think we can mostly do this seperately, but I think we need to
   ensure we can call enable and attach at any time, but the current HIL has
   enable and attach callbacks that are never triggered, so I was wondering whats
   up with that.
 * Hudson: Why not just return an error code if enable and attach fail?
 * Guillaume: Maybe, but I just want to point out that the current CTAP PR does
   not work on nrf52 because attach happens too early, and this seems indicative
   its a problem.
 * Phil: Hudson, the reason is that its unclear when to retry if all you have is
   a failure but no indication of when the call will succeed.
 * Amit: I think it is fine in some interim if USB works only under certain
   conditions (USB attached to board at boot etc.), and it is fine if OpenSK
   waits to adopt the new CTAP implementation until we fix the HIL to support more
   advanced scenarios
 * Guillaume: Sure, we also assume USB plugged in at boot. I just want to point
   out that until now we have assumed that at boot the USB is already attached
   and this currently does not work.
 * Phil: I think we have broad agreement that we want this in order to make USB
   more robust, just that we do not think we should block CTAP on it.
 * Hudson: Could we just add a short delay in a driver to get this to work until
   we have the lifecycle HIL?
 * Guillaume: Sure, but setting an arbitrary timeout is pretty brittle and
   wasteful compared to listening for the appropriate interrupts.
 * Phil: And there could be something that just doesn't use the HIL temporarily,
   and uses an additional trait, and another capsule that requires both traits.
   If you have something that uses the USB HIL and an implementation specific trait
   with events thats totally okay. If this other one does not get
   connect/disconnect events that is fine and can be updated once we have an
   official HIL for connect/disconnect events.
 * Guillaume: Sounds reasonable to me. So there could be a capsule that uses an
   additional trait and only works on specific chips until this stuff makes it
into the HIL.
 * Phil: Yeah. Similarly noone has the same hardware accelerator as say H1B but
   there is a capsule for using it.
 * Amit: OK - so I think we have summarized where we are on that, any last
   minute thoughts?
 * Leon: Short Q: I have been pushing for Zenodo integration into Tock, which
   saves all changes and pushes them to CERN with a DOI, which is useful for
   citing. I want to cite the threat model and currently do not have a great way to
   do this. Is there any way we could do this before 1.6 so the release would be
   archived.
 * Amit/Brad: Yeah I think we can do this and add something to the README also
   asking people to cite the SOSP paper.
 * Amit: Seems reasonable to me.

