# Tock Core Notes 2022-08-12

Attendees:
* Branden Ghena
* Johnathan Van Why
* Leon Schuermann
* Arun Thomas
* Alexandru Radovici
* Chris Frantz
* Pat Pannuto
* Phil Levis
* Alyssa Haroldson
* Jett Rink

## Updates
 * Branden: I have been working with capsule unit tests based on Alyssa's draft PR. https://github.com/tock/tock/tree/capsule-unit-test I'm making GPIO unit tests right now and so far things are working. Thinking about how to add helpers for unsafe kernel resources we need to use within capsules while unit testing.
 * Alyssa: Google has documentation on how to make better unit tests and helpers. I've got lots of thoughts on the issue.
 * Branden: Awesome. I think we can discuss and think about how to do unit tests best. I'll likely make another PR/Issue to discuss it.
 * Branden: The issue right now isn't so much the unit tests as the infrastructure. Having kernel and capsules in different crates makes testing harder than it might otherwise be, since the tests config option doesn't cross crate boundaries. There's an open Rust issue on it.
 * Alyssa: Yeah. There are some options you could use a feature or have a dedicated crate. Using a feature gets all the trait elements, which is what my draft PR does.


## PR Overview
 - Merged (non-trivial):
   - https://github.com/tock/tock/pull/2516 QEMU riscv32 "virt" board
   - https://github.com/tock/tock/pull/3117 Apollo3 IOM fixes
   - https://github.com/tock/tock/pull/3118 AirQuality capsule
   - https://github.com/tock/tock/pull/3119 NotEnoughFlash TBF parsing errors
   - https://github.com/tock/tock/pull/3120 Tickv panics to errors
   - https://github.com/tock/tock/pull/3131 Catch errors in size CI tool
 - Opened (non-trivial):
   - https://github.com/tock/tock/pull/3129 (DRAFT) Capsule unit test
   - https://github.com/tock/tock/pull/3134 (Release Blocker) SAM4L UART deferred calls
   - https://github.com/tock/tock/pull/3136 Alarm implementation bug
   - https://github.com/tock/tock/pull/3137 Scheduler infinite loop
   - https://github.com/tock/tock/pull/3139 (Release Blocker) Redboard Artemis hardfault
   - https://github.com/tock/tock/pull/3140 (Release Blocker) Reboard Artemis app loading

 * Branden: Note that some of these are release blockers for Tock 2.1


## Infinite loop in scheduler
 * Branden: Jett found a scheduler infinite loop in https://github.com/tock/tock/pull/3137 that's had a lot of comments and back and forth.
 * Jett: We had some discussion and are pulling more general design questions into a separate issue: https://github.com/tock/tock/issues/3138
 * Jett: I think taking a step back and looking at the incoming work and processed blocked machinery is a good idea here. It's probably the better solution long-term because I don't think that the current optimization to not iterate processes is really needed.
 * Jett: I suspect that just iterating all N processes is fine because N is very small.
 * Branden: This code is fairly legacy so I think we'd at least be open to discussing all kinds of changes.
 * Pat: As memory serves, the optimization comes from a concern of energy savings. Don't want to waste energy checking all processes.
 * Pat: I think we should measure how large the costs actually are, and it's not worth saving unless the measurements support it.
 * Jett: Do we have any low-power apps for guidance?
 * Pat: Mostly on the research side. Energy-harvesting applications like Brad has.
 * Phil: It's not just about energy, but also CPU cycles. How long this takes constrains the maximum interrupt rate.
 * Phil: I'm not saying this optimization is the right choice. However, there are some weird things without it. If interrupts come in really quickly, they just chain and work fine. If they come in just a bit slower, we could get caught in the "scanning processes" loop and not make any progress. So there would be a weird step delay as the rate decreases.
 * Phil: So the question is, how many cycles does it take to do this scan. If it's microseconds, we're likely just fine with it. If it's milliseconds, that could be an issue.
 * Alexandru: We ran some tests at one point and found that Tock stopped responding when given an external interrupt at around 10 kHz. We didn't look into the exact reasoning at the time.
 * Leon: It would be great to reproduce that. I tested once and found much larger numbers, with this still reasonably workable. It would be good to look into this.
 * Alexandru: We will try it, probably next week or so.


## Tock 2.1 Status
 * https://github.com/tock/tock/issues/3116
 * Branden: 2.1 release candidate is out and there is an issue tracking testing of boards. Everyone should look and see if they're responsible for testing and check their box if so.
 * Alexandru: What's the deadline? Is next week okay? (general consensus that it's fine)
 * Leon: I think the policy was two weeks without hearing back from anyone is a problem.
 * Leon: Also, I was collecting LiteX feedback since the last release and am going to make updates to board/readme/bitstream to make it easier to use, possibly with a Docker image. Is this okay as part of 2.1?
 * Branden: I think it's probably fine. A lot more palatable than kernel or capsule changes.
 * Pat: I agree. Sounds like patching a bug to me.
 * Alyssa: Do we have a list of API breaking changes since 2.0? That would be quite useful.
 * Jett: I have one to add to it.
 * Leon: Generally, the release notes include things like that. Let's make a separate tracking issue and put them there.


## Ti50 Frustration - Printing Flush and Synchronization
 * Alyssa: One theme we've run across is that printing is frustrating from code size. Performs worse than printf.
 * Alyssa: Flush doesn't actually flush either. Getting a better idea of why there are multiple queues would be useful. And we want a flush operation that always validly flushes.
 * Leon: Are we talking about code size or speed (code size).
 * Phil: And by flush, you mean a synchronous wait until stuff is printed out.
 * Alyssa: yeah. I'm trying to understand why it's not today.
 * Phil: There are no synchronous calls in the kernel. So it's not the same semantics as a POSIX flush.
 * Alyssa: So there's that in the kernel. Or some way to wait until the print is finished. This is in apps too. Definitely want the app to wait until things are really printed and finished.
 * Branden: I think these are two issues. The app one is somewhat segmented from the kernel one. In the kernel you can't have a synchronous interface and need a callback. You could pause the state machine and not move forward until the callback.
 * Phil: We do not ever ever want to use a synchronous flush in the kernel for normal code since the whole kernel will halt. But for tests it may make sense.
 * Alyssa: Or log-based debugging which was the case here.
 * Phil: But if you flush and the kernel spins for a while, all kinds of things could go drastically wrong.
 * Alyssa: I'm curious about what could go wrong?
 * Branden: Generally introducing arbitrary timing delays like that screws up all kinds of external interactions with hardware. It's the same reason that you sometimes can't attach GDB to a running microcontroller. It'll get you to a point, pause, and let you introspect variables. But continuing running afterwards sometimes doesn't work at all.
 * Leon: For testing, do you mean tests that run on actual hardware?
 * Alyssa: Not tests. It's debugging. So we want to make sure that prints are consistent and ordered between the app and the kernel. I want to have ordering constrained.
 * Phil: Can you describe the case of a userspace process and calling print? What's the timing concern there?
 * Alyssa: For debugging, say I'm debugging and app and a capsule it communicates with. So the app prints. Then after that the app syscalls into the capsule. Then the kernel capsule does another print. There's no guarantee that the kernel print occurs before or after the app print.
 * Phil: This is helpful. So this is a very specific, and important, case. The issue here is that the virtualizer for the console doesn't operate in a FIFO order. So the app can print and memory gets copied into a buffer. The virtualizer doesn't guarantee which buffer gets copied into the low-level buffer first. It's somewhat arbitrary based on the order of virtual requests in the linked list. We have discussed having FIFO ordering.
 * Alyssa: That would help. Flush is also an issue for some things. Particularly it would be great to be able to print a massive amount of stuff and have the debug buffer not overflow, which is a big issue for us now.
 * Alyssa: The other frustration is moving between apps and kernel.. The happens-before relationship doesn't happen, which breaks expectations.
 * Leon: I was thinking that printing in the kernel is used seldomly in production. So maybe we only need to support the debugging use case rather than the general one.
 * Phil: Well, ordering is still useful. The reason we don't have FIFO ordering is mostly accidental. The happens-before relationship would be important to maintain.
 * Phil: The challenge here is that if you just do a simple queueing approach, you can get weird behavior. An app does a print which gets enqueued first. The capsule does a print which gets queued second. Then the application does another print. The second application print is just appended at the end of the application buffer. So you might get both application prints before the kernel because we don't keep track of individual requests, just a buffer.
 * Alyssa: I think we should consider changing that.
 * Leon: Let's say we did have total ordering and ticketing for individual reservations. You'd have a different problem where one app could denial-of-service another app by printing a very long string.
 * Phil: Well, there are limited sized buffers. So there's already a switching mechanism there.
 * Leon: Is the chunking on the driver level? Is it exposed to applications?
 * Phil: No, it's not exposed.
 * Leon: So from an app's perspective, you couldn't be certain that a long message you are printing would print in its entirety with total ordering.
 * Phil: I see, so if the app does a huge dump of information that would starve the other applications. To be fair, POSIX doesn't ensure atomicity.
 * Leon: I was going to ask how POSIX does this. Is there ordering or is it unspecified?
 * Phil: It's tricky, because things in POSIX are blocking. So there's fundamentally some ordering. When I call print and print blocks until the print is complete that fundamentally creates an ordering. Because if I then send a message to another process, I know that message is after the print.
 * Leon: But if you have two applications printing to the same virtual terminal at the same time, those can be intertwined.
 * Phil: Right, but that's not a happens-before relationship. Independent processes could be running at whatever speed on different processors.
 * Leon: Okay, so applications can synchronize themselves due to blocking. That maps to our applications. If we had a way to specify back to the application that the print did totally occur to real hardware, then we could block the application until we get that notification.
 * Phil: Within the kernel we do this in hardfault handlers: synchronous writes with blocking. But if you do this in the kernel and halt things for many milliseconds there's no guarantee that things still work afterwards. You'll get weird issues afterwards.
 * Leon: I'm arguing that synchronizing between two apps and app/kernel are similar and we expect both or neither to work.
 * Alyssa: Especially with IPC I expect the ordering to be needed.
 * Phil: I think this is a virtualizer ordering issue, but that sounds doable. Big flushes in the kernel are harder.
 * Alexandru: Would a dedicated events capsule help here? It could keep track of causality for events.
 * Alyssa: Sometimes I want to dump, say, the SHA hash or the contents of memory. And it would be nice to guarantee that. Every method I try just gives me "debug buffer full".
 * Pat: Is this something you continue afterwards? Or you just want information and then to stop?
 * Alyssa: I want to continue. It would be useful to do a print, do some work, do another print. They're on the order of 20 kB prints.
 * Pat: I mentioned because the hardfault handler tears down the world and prints things synchronously. That would work if you don't want to continue, but doesn't work if you do want to continue.
 * Phil: Here's the line that has ordering for the virtualizer process console: https://github.com/tock/tock/blob/1e3f8c1757a4582cf870ef20e9445cb5ed949e9e/capsules/src/console.rs#L330 When you finish a write from an app, it just search all apps linearly. So changing that to a FIFO with ordering would help, with the caveat that it's possible multiple application writes are combined. We don't track write state, just data.
 * Alyssa: You could add some ordering tag in the app to do this.
 * Phil: If you get the callback when it's actually written over the UART, that will serialize things.
 * Leon: I think this is the right way to think about this. It would make apps capable of synchronization but not enforce costs.
 * Alexandru: I think maybe a dedicated capsule with a large buffer might be more useful. Or did you just try making the debug buffer really big?
 * Leon: I think a general synchronization primitive makes sense here for printing. I doubt we could make a truly general primitive even if the idea applies.
 * Alyssa: I still see the easiest primitive on the app end is a synchronous flush.
 * Phil: Waiting for the callback is the flush. I'm trying to see when the upcall is triggered.
 * Alyssa: That's when it finishes copying to an internal buffer, not when it's written out.
 * Phil: So that plus the lack of ordering is the issue. We would like the semantic to be a notification when it's actually written back.
 * Leon: It can sometimes be useful for applications to write and not flush.
 * Phil: Printf is sort of like that, but write doesn't do that.
 * Alyssa: Write is often line buffered.
 * Leon: If we could change semantics for only synchronous writes that would be great. I just think changing the semantics for both operations would be a mistake.
 * Alyssa: When we receive back from the print and it's done a flush to the internal queue, that's still useful.
 * Alexandru: So your problem is actually that you print large buffers from apps, but the issue is that they might sit in the console buffer?
 * Alyssa: That was an issue where increasing the buffer size in the app didn't fix things because there are multiple buffers.
 * Alexandru: I think right now when it copies it overflows. You could check and just do the copy piecemeal, but then you lose the event.
 * Alyssa: I think a flush with an overflowing buffer should write what it can and copy the rest. That might be hard though.
 * Alexandru: We could change the console to copy things and if it can't copy everything to delay the print. But you'd lose the order.
 * Alyssa: What I care about the most is that flush actually flushes. The team discovering that flush didn't actually flush was frustrating.
 * Phil: It shouldn't have been named that since it's not really a flush.


