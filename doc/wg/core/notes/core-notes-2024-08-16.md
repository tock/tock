# Tock Meeting Notes 2024-08-16

## Attendees
 * Branden Ghena
 * Alex Radovici
 * Brad Campbell
 * Johnathan Van Why
 * Leon Schuermann
 * Alyssa Haroldsen


## Updates
 * Alex: Almost ready to upstream to configurator and get feedback! Draft PR next week. Microbit works so far and RP2040 is next. They're rather different, so that's helpful. We are looking for lots of feedback, definitely not final
 * Alex: Started porting Tock to ARM64, for RPi boards. Demo on PI 3 to start because it works in QEMU. Tock could be a second-stage bootloader for these boards.
 * Alex: Students working on a bunch of PRs. Servo and Distance sensor right now. Tockloader-rs coming soon.
 * Branden: Ultrasonic sensor code is very high quality
 * Alex: Yeah. The only big problem with it has been timing: Tock can't do very fast turnaround for GPIO so we had to stretch 10 us into 1 ms, but that works fine for this sensor.


## Treadmill
 * Leon: Treadmill platform has been under work. We've been tracing bugs. We had one with connecting nRF52840DKs to VMs, but it's pretty rare and we can't reproduce it. So let us know if you see it.
 * Leon: We're also reconstructing the release tests with some automation.
 * Brad: JLink will always be hard. They just release new versions without care about what they keep or break/change
 * Leon: We have three moving targets: different JLink chip revisions on different nRF boards. Those chips run different firmware versions. Finally, the host toolchain is possibly different versions. Between all three, it's a mess. Physically power cycling the board is sometimes needed. Treadmill will support that though.
 * Brad: It feels to me that we'll find some version that works and pin to that for a while
 * Branden: Right, we won't swap out which board we're using for testing
 * Leon: Yes. I see this as education for users getting started. It was a valuable lesson to me that even our best-supported platform has issues that we might not be aware of on our own machines/boards. New setups and new hardware has quirks
 * Leon: Probe-rs did not fix these, by the way. It's error messages really don't help and often just say "internal error"
 * Branden: Besides the error messages, probe-rs is not worse though?
 * Leon: The only other aspect that's bad is that it's the slow programming speed, like OpenOCD has. The JLink proprietary toolchain does some magic to speed things up.
 * Brad: The other question, for the release testing do you have a sense of timeline?
 * Leon: No right now. We've been focused on getting the platform in a usable state so testing can be spread between many people, not just us three developers. So while Ben is now shifting focus to testing, our goal is to get the platform open to people as fast as possible. So people can try interacting with hardware and find pain points that we didn't notice
 * Brad: Makes sense. I'm really just asking for planning purposes. Probably going to take a lot of time the first time, to get everything straightened out. So we need to watch for when we really need to start pushing for release and merging PRs. Whatever works for timeline is fine, but be sure to make people aware so we can put a freeze on PRs and decide which ones must be merged before release
 * Leon: We'll definitely discuss here on how to use Treadmill to advance the release, when it's ready
 * Leon: Should we have a meeting at some point to get other people involved?
 * Brad: I don't want to complicate your end. Having more cooks in the kitchen isn't necessary, unless it's helpful. It doesn't have to be perfect before the release, and won't be
 * Leon: Yeah, it'll be a few releases probably before the vast majority of tests are on it. So I think that once we have a small handful of boards and can try stuff, we can manually test but using Treadmill. Then when that works we can automate parts of it. So we get experience with tests and what works
 * Brad: I think I understand what you're saying. I do think it would be helpful to have some kind of format to put the output in. If I'm manually running a test, I can just do that on my own device. So what's the advantage of Treadmill? But if we have to put together some configuration.
 * Branden: I think the motivation was instead of testing on our desk, do things manually but through Treadmill as a test for it.
 * Leon: Yes. However, I do see that it's problematic unless we have some intermediate format for writing down expectations in a usable way.
 * Brad: I'm being wary of adding extra steps or roadblocks here
 * Leon: Agreed. Current state is that we can launch jobs on targets and talk to boards. We'll have credentials soon. So for the next couple weeks we could have very alpha-stage python testing scripts that call cargo and flash boards. And we can call that a first release and then take the experience into the next release.
 * Brad: That sounds great. I think that's a good use of expertise. You and Ben can troubleshoot the platform faster, but we have more experience with the tests and platforms than, say Ben does. I started going through a few tests on my own too
 * Leon: There are definitely some assumptions and things to look out for in tests that we never really recorded anywhere. For example, there's a console timeout test where you really have to understand console and userspace to understand if the response is expected or not
 * Brad: Yeah, our current approach is very human understanding in the loop. So we should put that burden on us instead of Ben and you. But we shouldn't spin our wheels when we don't understand some Treadmill thing. So division of that labor would be great.
 * Leon: I do think it's valuable for us for others to use Treadmill, but with the knowledge that they should give up and ping us if anything goes wrong even slightly
 * Brad: Sounds good
 * Leon: I remain very excited for Treadmill. We've been paying off technical debt rewriting things and it's in a good state


## Libtock-rs
 * Brad: There was some discussion of libtock-rs. Is that still going on? Two things: Hudson's mem-op changes and the 15.4 stack
 * Johnathan: I think the 15.4 stack is still an area of discussion. Not really sure
 * Branden: I do think the 15.4 PR is in a waiting-on-author state, but with the knowledge that the author is unlikely to make any progress. The Mem-op stuff, I think, is the first step towards being able to hand that author something so they could make progress again. But I don't know the state of the mem-op stuff at all.

