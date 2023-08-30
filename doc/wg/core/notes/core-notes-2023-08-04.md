# Tock Core Notes 2023-08-04

## Attendees
 * Branden Ghena
 * Hudson Ayers
 * Phil Levis
 * Leon Schuermann
 * Chris Frantz
 * Johnathan Van Why
 * Brad Campbell
 * Alexandru Radovici
 * Pat Pannuto
 * Alyssa Haroldson


## Updates
 * Brad: After tutorial at Tockworld, I went through the Tock book to clean stuff up and remove redundancies. Also, I added more setup documentation so we don't need to maintain separate kernel infrastructure to support the tutorial. Live in the book now.
 * Leon: I've been working on porting the tutorial userspace to libtock-rs, working on the HMAC driver now. I am envisioning a minimal example first without app-flash and without USB HID. Maybe we'll add TicKV support at some point.
 * Brad: That's great. I was curious if we should make this a separate app from the libtock-rs repo to see what it looks like to do a totally separate app.
 * Leon: I do think it makes sense as a separate app in the tutorial.
 * Brad: I meant as a model for an out-of-tree libtock-rs app
 * Leon: I'll think about that. The context is that my company wants to try the demo, but without libtock-c
 * Branden: Wanted to announce the Network Working Group. https://github.com/tock/tock/pull/3578 It's going to have calls every other week to talk about network interfaces in the kernel and to userspace and support for network stacks, such as buffer management. Others are welcome to join if interested. We discussed at Tockworld and had support from the core team. Phil and Johnathan weren't there, do you support creation of the working group (Yes from both)
 * Chris: I'm working on updating open-titan support to bring support up to the tape-out version. https://github.com/tock/tock/pull/3586 is the initial work on that. There will be a few separate PRs to handle everything. Some constants for base addresses and register constants. We'll take on the drivers one-at-a-time moving from hand-crafted registers to auto-generated ones. We'll also complete the drivers for the features that the chip supports. Current drivers are an incomplete start, but a little work will bring them up to compliance with the hardware and the capabilities in the HILs.

## Yield-for RFC
 * Pat: There's an RFC for yield-for: https://github.com/tock/tock/pull/3577 Thinking about moving the yield-for logic into the kernel simplifies correctness in userspace. You don't have to deal with the concept of multiple parallel upcalls that could occur. For example, `printf` in a callback wouldn't have to worry about reentrancy. We'll always get back to exactly where we are, if we want to. Also helps efficiency for userland syscall stuff, potentially.
 * Phil: This is interesting. I'm curious what the similarity is here with a blocking command. While you're blocked here is that timers can't expire. That's a concern.
 * Pat: I view it as an opt-in to synchronous behavior.
 * Phil: But we have multiple ways of doing things. So we have to be clear which way to do things when. When do you want to do which?
 * Pat: We don't have blocking-command as a concept upstream. Even in the batched system calls, you could skip some of that.
 * Phil: From an idea this sounds great. From a sense of what are the tradeoffs.
 * Branden: To confirm, this is the same blocking-command that Alyssa has been talking about (Yes)
 * Hudson: There is some discussion of this on the PR that we could push. Amit is interested too, and we could have a longer discussion next week.
 * Alyssa: A question here is whether users would need to implement command and blocking command or if there's some restriction. I think I have a design where anything can use either.
 * Pat: I suspect that's similar to the implementation of yield-for internally
 * Alyssa: You do have to specify in userspace which number you're waiting for. If blocking commands are implemented by the capsule, the kernel could ask which command and which subscribe it's waiting for. So it could go in the kernel rather than userspace.
 * Pat: Yes. I will go through the writeup for this RFC and add more details. I think I missed some of your concerns. If there are multiple possible upcalls, the design now is that you just wait on one. But if you wait for data and there's an error upcall, you're in trouble
 * Phil: A common way there is return values to determine what happens. There are cases where there are one-to-one mappings between sync and async interfaces in other OSes, but there are cases where it doesn't make sense to have both.
 * Alyssa: If yield-for lands first, blocking command could be built on top of it.
 * Pat: Yes, that was the design.
 * Alyssa: I think there should be discussion about the name to make sure it's clear too.
 * Pat: Sure.

## Libtock-rs Updates
 * Brad: Two classes of changes. I was trying to get the examples working and ran into some bugs I made PRs for. Also as part of the key-value overhaul, I thought it would be a good example driver to implement. I did that, although we're updating that in the kernel, so I don't want to pull that right away.
 * Brad: Leon also opened a tracking issue with missing drivers that libtock-c supports but libtock-rs doesn't. https://github.com/tock/libtock-rs/issues/489 The idea here is that each core team member would implement at least one driver as an example and to get used to libtock-rs
 * Brad: The second part is the build system. Right now you pick boards when you compile, but it would be nice to have multiple fixed-address versions compiled and have Tockloader pick which one it wants. So the update in https://github.com/tock/libtock-rs/pull/482 builds for a bunch of possible addresses and lets Tockloader choose.
 * Hudson: The approach to build for multiple locations, I'm interested to hear Johnathan's thoughts on how it would interact with the existing design.
 * Johnathan: I was going back-and-forth with Brad in the comments. Everything has been changing on that PR pretty quickly. I think Brad came up with a solution last night that would make me happy. It's a really hard problem. Cargo doesn't really understand the concept of building binaries multiple times and then combining them into one TAB. Cargo doesn't get that, and the only other system is `make`. So, it's not easy to expose without extra build infrastructure.
 * Alyssa: We do have some Bazel in our downstream working for Tock. It is really good for this
 * Brad: Let me give you a quick overview of where things stand on my end. Johnathan's idea of having a build-focused crate that handles the linker scripts and is a build dependency. So if you want to choose board based on  build variable, you just import that crate. It should handle the linker script nonsense. The second step is make, which is indeed bad at being externally usable. I did some hacking and made a way to define targets for flash-ram-arch-tuples. So out-of-tree boards would have to copy something, but I think it might really be minimal.
 * Johnathan: So the stuff with the fixed-target function could be in a separate makefile and apps could include/call it if they want.
 * Brad: Yes. They could also include other addresses with the same function and it should "just work".
 * Johnathan: That might be a good approach.
 * Johnathan: One extra thing, I'm seeing a lot of stylistic things that I never wrote down that I disagree with from your changes. No way for you to know because it's only in my head. I'll make those comments on the PR after we agree on a design.
 * Hudson: For issue #489, we talked about at Tockworld that people should each sign up for one driver to get a feel for how things work. That would be good for everyone to get a feel for how stuff works.
 * Brad: I wanted to mention that it is still a little challenging to get libtock-rs apps that will work on a particular board. There is some other machinery in progress that we should talk about on the repo. The main challenge is that if Tockloader is going to load an app on your behalf, it's pretty good about choosing the Flash address, but doesn't have a great way to choose the RAM address. So you can get apps that are at the right Flash address, but not the right RAM address. So you have to watch out for that.
 * Leon: This is identical in libtock-c for RISC-V. We might be able to have a start-ram address in the kernel, which we could know at compile-time if we violate. Right now we just start app RAM right after the kernel RAM, so it moves. By having it be fixed instead, we make it must more likely that apps will run.
 * Brad: That will work for RISC-V on libtock-c, but it won't work or libtock-rs. Because different platforms might put their RAM at totally different addresses.
 * Leon: But we do have linker scripts per-platform, so we could encode it there.
 * Brad: Yes, that would work.
 * Leon: It's not a full fix. Multiple apps at once still has issues. But it would at least make a single app work.
 * Brad: I do think I have a solution for all of this. The idea would be to encode the memory address in the kernel binary in an attributes section. That way Tockloader can discover where app memory is at, which would let Tockloader be aware and adapt. https://github.com/tock/tock/pull/3588 
 * Leon: The compile chain would still need to pick addresses correctly when compiling though. (Yes)
 * Branden: A question for Johnathan. I think there has been confusion about the limitation of libtock-rs to wait on multiple things. Can you explain and clarify.
 * Johnathan: You can do all of the asynchronous stuff you want, 20 things at once, as long as it's all in one function. You can't have multiple threads running at once: blinking a light in the background while running cryptography. There isn't a limit on how many callbacks you can do, but the subscribe/allow are tied to a lifetime that's on the stack. It's possible you could do that. But that's not the design right now. There's not a hard limit, but all of your asynchronous stuff is supposed to be local or lifetimes will mess up
 * Leon: Is that similar in semantics to a "select" system call?
 * Johnathan: I _think_ it's weaker than select. With something like select you can build a library that allows multiple things to run and build stuff on top of it.
 * Leon: So you'd have to wait for all callbacks to return?
 * Johnathan: Not really. It's just a lifetimes issue.
 * Leon: So this does sound reasonably flexible though. You can have an event loop with multiple callback and switch between them.
 * Johnathan: Yes, all of those callbacks need a lifetime though. And I do have like four or five emails that have been sitting in my inbox with ways we could incorporate futures without a huge cost. It's a HUGE API overhaul though. It would be great if they pan out though
 * Alyssa: I am planning on doing some work on that and will let you know.
 * Branden: To wrap up, this is really a compile-time limitation, so it's not a trap someone is going to fall into. Instead it'll be obvious that it won't work, or at least confusing because someone can't figure out how to do it with lifetime requirements.
 * Johnathan: That's right. It'll be a failure at compile time. The thing that could be sharp is that "share" is used to allow, and it does unallow when it returns, so that could catch you by surprise
 * Branden: Great. Good to hear it's not going to surprise us more generally


## Open Titan Upstreaming
 * Chris: We do want upstream code that can run on open-titan, run some apps, run some tests. Downstream, we'll have our own definitions for things that are actual products. So the upstream won't match the products exactly, but having both of them serves as a good tutorial for new users of the Open Titan stuff. They can look at both as references. We intend to assemble a reference OS in the Open Titan repo with a userspace that demonstrates the basic features of the chip. The stuff in the repo will put together a kernel and userspace that can run on the chip.
 * Brad: That sounds great!
 * Chris: I've done some work on starting this in the opentitan repo. I think Leon has done some work advancing it from there.
 * Leon: I do want to give overview of high-level plans if we have time. (we have time)
 * Leon: I am still new in this area, but here are what I think the ideas are. We've designed Tock to hopefully be usable by downstream, but never really validated. Right now, things are well-aligned. Trying to rely on the upstream code base for Tock, relying on the core capsule crates. But a downstream Board crate, and _maybe_ a downstream chip crate. Chris has a proposal that integrates this and pulls crates from upstream Tock.
 * Leon: Something to talk about next week on the OpenTitan WG call is that the versions of Tock and OpenTitan are out of sync. Now that OpenTitan has a taped-out chip, that should be stable, so how do we go about updating and synchronizing.
 * Leon: There's a question about which regression tests we should/shouldn't be passing. There's also how do we run. You'll need to be on a downstream chip. So how does Tock make sure it doesn't break things? It needs to run on a downstream environment for tests. We could follow the downstream instructions, but that's weird for Tock to want to do.
 * Chris: I think now that OpenTitan has a stable chip, it's probably easier to have something in upstream Tock that will be reasonably close to always working on that stable underlying hardware/software support. I do agree with Leon that the current set of tests is hard for me to say what is/isn't a regression as those tests haven't tracked the project. The tests are out of date, and I'm honestly pleased that many pass as-is. We will have to think about more comprehensive tests. But at least we now have a stable baseline for hardware.
 * Pat: So should Tock just set up our CI with the taped-out version of the chip? Or should we have CI calling into your ecosystem to test stuff?
 * Chris: From our perspective, we'll have tests in our codebase that use our test infrastructure to exercise things we care about from Tock. Leon gave me a quick tutorial of the Tock CI environment and driving test boards. And that's what we have too. Unfortunately, we're based around Bazel, we can't build the tests with Cargo. So you can't even reference them from Cargo.
 * Pat: Yes. Something we've talked about is that there will be proprietary things that could be called from Tock and should be aware if we break something for that user. It could even be a non-blocking test to just make us aware that we would be breaking something for someone else.
 * Leon: Another takeaway is that if the tests are ultimately going to use some third-party build tools, is it worth hanging on to a reverse-engineered README which has some bespoke workflow for building stuff, or is it better to just call out to external downstream docs to say her's how they do it.
 * Brad: I think that we should not duplicate. The setup seems like it should be outsourced. However, Tock likes to be as user-friendly as possible. So we should add some tips for how to get stuff working. We should make sure not to remove stuff from the README if it's not existing somewhere else.
 * Leon: Yes. I think the motivation was to bring the README stuff back once stuff works again. Perhaps the next steps are just building Tock in the Bazel downstream chain, then Tock can reference that.
 * Brad: One more question is if this should be a giant PR or piecemeal. We like it where code works at every PR instead of half-working states. But that might be too much work.
 * Chris: I'm close on the initial PR to having it working. That first PR is an atomic change that brings the upstream Tock in alignment. Then the rest should be upgrades that can be down piecemeal afterwards without breaking anything.


