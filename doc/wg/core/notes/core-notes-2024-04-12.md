# Tock Meeting Notes 2024-04-12

## Attendees
- Branden Ghena
- Hudson Ayers
- Phil Levis
- Andrew Imwalle
- Leon Schuermann
- Johnathan Van Why
- Tyler Potyondy
- Alexandru Radovicii
- Brad Campbell


## Updates
* None


## Libtock-C Rewrite
* https://github.com/tock/libtock-c/pull/370
* Brad: Since last week, there's not been a ton of progress except for Hudson on SPI.
* Hudson: Started looking into libtock-c rewrite and ported SPI driver. Also renamed that to SPI controller to match the kernel capsule
* Brad: For the remaining ones, is it okay to call some "outdated" and leave them alone?
* Hudson: I think it makes sense to me. We can move forward incrementally, as long as everything builds. Notably, currently things don't build
* Brad: What doesn't build?
* Hudson: Examples, which haven't been ported to the new APIs. So I guess you can build, but you can't test
* Brad: Yes, a handful have been adapted, but not all of them.
* Brad: I think it does make sense to not let a perfect end-goal stand in the way of progress
* Phil: Can you provide background?
* Brad: It's a format for writing the libtock-c library drivers, and the actual implementation of that rewrite. Here's the actual documentation: https://github.com/tock/libtock-c/blob/4d03cc5c0b1cbd464d5efac021534d0177ed2d3c/doc/guide.md
* Phil: Is there anything technically difficult about it, or just lots of work?
* Hudson: There are a few drivers like timers, that don't fit well into the new design, so we have to think about those. But mostly it's not crazy difficult.
* Phil: I would be interested to pitch in, in a few weeks, if stuff still needs to be updated
* Hudson: I was planning to spend time on alarm/timer next week, but maybe I should spend time on porting examples instead?
* Brad: I'm conflicted. Timers are so common that they seem pretty important.
* Brad: The timing requirement here: tutorial is going and needs documentation, but we really want to focus the docs on the updated version of libtock-c. The current version is rather inconsistent. So we don't want the tutorial to wait too long, but we want it to be on top of this update.
* Phil: If we plan to have to the code updates done by the end of April, that might be okay.
* Hudson: Another though, I wonder if it's better to port apps "one driver at a time" rather than "one app at a time". Because you could just change names of functions.
* Brad: It's just like the drivers, they vary. Some apps are straightforward and some are very much not. I'm not sure what's easier.
* Hudson: Okay. So you have run into places where you can't just rename the function.
* Brad: Yeah, they range all over the place. Some are trivial and some are a very different model.
* Phil: From the tracking item in the PR, there aren't many left
* Hudson: True, but the examples aren't on this tracking list and most haven't been updated.
* Phil: Looking through, there are like 70 apps between examples and examples/tests. So there are a lot
* Branden: That's its own mess too. We could address that now, but don't have to in this PR
* Phil: So backing up to the tutorial, what do we need to do for it? The important thing is the tutorial code being in good shape. But we do also want to avoid leaving junk around.
* Hudson: So maybe I'll do alarm/timer first, then focus on examples
* Brad: IPC, Console, and Alarm are the big three. And of those Timer is most important. Printf isn't going to change, and we don't use IPC in most stuff.
* Phil: How about, since I was deep in the timer stuff, I could lend Hudson a hand so he doesn't get lost in it
* Hudson: When I looked into it, it seemed that most stuff wouldn't change. One thing that's weird is that we have an alarm.h and timer.h but one alarm.c which defines things across multiple header files. There's some internal stuff that would go in syscalls. Also that we have delay_ms and yield_for_with_timeout, which would probably go in the libtock-sync folder in the redesign. But maybe these are things they would need for any platform/application.
* Brad: delay_ms is clearly synchronous.
* Hudson: I'm about an hour in to looking at it and wrote down some notes on things that are unclear
* Phil: I can't code until next weekend, but I could advise on questions
* Hudson: At the end of the day, until all the examples are ported there's no way to use this for the tutorial. But I don't think it'll take longer to do examples first, then timer after, then go back and fix some more examples.
* Brad: I will try to make the same checklist for examples. I don't think it's as bad as it seems
* Brad: I do want to know that we're going to merge this
* Hudson: I think as soon as most of the examples work and can build, we can merge. It's definitely an improvement
* Hudson: We do have to be careful to test things, as there are places we could introduce bugs. But also, I fixed a couple bugs in SPI when porting, so maybe things are improving too


## PicoLib Support in Libtock-C
* https://github.com/tock/libtock-c/pull/357
* Brad: This has come up various times. But until recently, it seemed like we had to use PicoLib OR Newlib, but not both. Now that we have Makefile support for libraries, it's actually more straightforward to just support both.
* Brad: So this PR adds PicoLib. It kind of worked. There are a few low-level, read write seek, functions that have different names. But I added a wrapping to convert
* Brad: So, this compiles. I probably works. A question is whether we want this.
* Hudson: I'm interested in how different the compiled binary sizes are.
* Brad: No idea
* Hudson: I also assume that Alistair would be interested in this, as I believe he was advocating for PicoLib a while ago
* Leon: How did you work around the issues Alistair ran into? PicoLib apparently had an sbrk implementation we couldn't overwrite easily.
* Brad: There was a configuration option somewhere in the PicoLib documentation. It was hard to find. Here's my build script: https://github.com/tock/libtock-c/blob/21174ddfd0965069a6ab75fdc609200a74bf4a2e/picolib/build-arm.sh
* Phil: To check on https://github.com/tock/libtock-c/pull/353, does that PR mean that you can't build things the first time without an internet connection?
* Brad: Yes
* Brad: So, the question from me: is there interest in PicoLib and is it worth trying to merge this PR?
* Johnathan: I think it would be great to add it. There is a file or two in PicoLib that are AGPL licensed. That makes it very scary for proprietary software authors. So there was a question of whether that file needed be deleted or something. My opinion is that PicoLib support is almost all of the battle. Then it can be on the people who are concerned to provide their own PicoLib files that don't include that.
* Hudson: So the other issues Alistair had are resolved in your PR?
* Brad: Yes, as far as I know.
* Hudson: So it seems like the top-level ask here is for someone to approve this PR or suggest any remaining changes


## Handling static mut
* https://github.com/tock/tock/pull/3945
* Brad: So what's the status here?
* Leon: The solution proposed should address our concerns, and doesn't make things more unsound than they maybe already are. I haven't been able to follow this lately due to other deadlines
* Leon: Brad's comments are valid concerns and we need to express some of why we need this. Some of the confusion is that this is derived from a multi-core version that we also have. But we distilled it down to the APIs that are needed for both, and then the PR only has a single-core system. It gets us something sync that also has interior mutability
* Brad: My understanding is that after last week, we are in general agreement with what you're saying. I believe Amit was going to open a new PR with just the types and new documentation. Then we'd have a separate PR for integrating the types.
* Leon: Okay. I'm not sure what the current progress on that is.
* Brad: To me, it feels like this was farther away from mergeable than indicated on March 1st. And maybe we should implement the hack solution so we don't have to wait on this and can move on
* Leon: I think with documentation this is mergeable now. I am personally fine with a different stop-gap solution though, such as the address_of macro. That change wouldn't break anything additionally.
* Leon: A concern is that doing that would reduce the pressure to come up with a good solution. We do things now that are very much unsound, and CoreLocal does address some of those issues. So we really want to integrate this kind of a change
* Brad: I don't think there's hesitation to including this. The challenge is actually doing the documentation and thinking about the upgrade path for other systems with different architectures. Those things take time and are necessary. So, I don't think that a stop-gap will remove the interest in this. But I also think this won't get merged without those things.
* Johnathan: Reminder that Alyssa is giving a talk on global mutable data after this meeting
* Leon: Okay, maybe by end-of-weekend Amit or I will push the stop-gap solution. And then we can get back to this PR after other deadlines


