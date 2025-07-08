# Tock Meeting Notes 02/09/24

## Attendees

- Branden Ghena
- Amit Levy
- Leon Schuermann
- Brad Campbell
- Alex Radovici
- Jonathan Van Why
- Alyssa Haroldson
- Tyler Potyondy
- Pat Pannuto


## Updates
### Display Support
* Brad: OLED screen work is in a good state in libtock-c. Support for the monochrome display is good
* Amit: Where does the display driver live?
* Brad: It's in the kernel
* Amit: And the C library just does like fonts and stuff
* Brad: Yes. The C library layers on top of the syscall interface we already had

### Hardware CI
* Leon: Update on Tock CI development. System I present a few weeks ago, is now being used for a CW310 OpenTitan board. You can start jobs and get access to a Linux container that has access to the board. So it's a good solution for boards we don't have many of, and in the future for automatically running jobs.
* Amit: Remote, reservation-based build environment. On the path towards building a CI system
* Brad: Would CI be essentially the same as a person using it?
* Leon: The base for this can talk github API, so whenever a github workload is created we get a webhook request to the server along with which type of job. The job is scheduled with a parameter of the workflow ID. So the environment would run a github action runner in "ephemeral" mode. Then that can run the job and reports back. The only difference is that we won't have to pretend to be a user and send SSH commands. We'll start a workflow that actually just runs some tasks

### Tockworld
* Amit: Tockworld update. Thanks to Pat we're moving forward with securing space at UCSD


## Mutable static references
* https://github.com/tock/tock/issues/3841
* Alex: We have a problem with the new version of Rust. It won't compile mutable static references in the future. It's a warning right now. We get 25 warnings on the Microbit right now, which will become errors soon.
* Brad: CI is checking this warning here: https://github.com/tock/tock/pull/3842
* Alex: Leon incidentally fixed one of these issues in the boards file, with the `ptr_address_of` change Alyssa proposed
* Leon: Yes, I still need to rebase that, but the change is mostly good.
* Alex: Deferred calls and several other places use static mut references too
* Amit: So that'll be important to address
* Alyssa: We might have to change to unsafe cell with a helper that gives a reference. Would have a .get_mut() function
* Amit: Unsafe cell has a const constructor, so that's probably fine. Just extra wrapping code
* Johnathan: `static mut` is stabilized as part of the 2021 edition. So the stable compiler can't deny that anytime soon at least. I feel like they can't just switch it off like that with Rust's stability promises
* Amit: Yeah. So it might not be urgent.
* Branden: Maybe this is real because we're on a nightly? The warning seems to promise that it will be an error
* Amit: It's an edition thing. The Rust 2024 edition will break it, even if the Rust 2021 edition allows it.
* Alyssa: Dependencies can stay on 2021, even if you're on 2024
* Amit: We should still fix. Especially if it's as simple as Alyssa suggests
* Alyssa: You could do `&mut* ptr_address_of()` still. That overcomes the lint
* Johnathan: That's fine for now. But it would be good to move towards what they want at some point. They're trying to get us to avoid this for a reason, so we'll likely eventually have a Tock cell that wraps unsafe cell in some way.
* Alyssa: Ti50 has a version like this. It acts differently on chip than on host for testing purposes. Unsafe sell or a checked, sync cell of some type

## Removing naked functions
* https://github.com/tock/tock/pull/3802
* Amit: This would get us to stable, I believe
* Brad: Yes. We've handled the last few other things. So that means we can merge this to remove the last nightly feature on ARM, and the Hail board compiles on stable!
* Branden: What's the nightly feature on RISC-V?
* Brad: ASM-const for the CSR library. It's more difficult.
* Jonathan: I see a rewrite coming when we update Tock registers
* Brad: The point is, if we're going to compile 99% of our boards on nightly, because we still like our nightly features, this is just a proof-of-concept that we _can_ compile on stable and will test roughly half of our code on stable. Importantly the capsules and kernel crates
* Alyssa: I see progress towards stable as unambiguously good
* Leon: I do think this is a great idea. What has made me skittish is that while we were making these changes, we had problems with the linker placing assembly code and not being able to branch between them. So I'm worried that the linker won't be able to make some optimizations now. The benchmarks CI run doesn't show any massive memory increases (or decreases) so we're probably okay, but I want awareness
* Amit: How did we fix it?
* Leon: I think we jump to a constant now, instead of a relative jump
* Amit: This is now on the merge queue
* Branden: Why do we want to be on nightly at all?
* Brad: Two makefile-level things that boards can opt into. Compiling our own Core library with optimizations, and a testing framework.
* Leon: I think we can use macro_rules to hack around the RISC-V CSR stuff. We could have an expansion which builds the actual CSR assembly strings from a macro
* Pat: I think we had the macro version before, and left it because it was ugly. So we could certainly go back
* Brad: So overview, we could go back to that. If there's someone who's motivated to see RISC-V on stable we wouldn't preclude that. I don't personally care though, because I think one example on stable is good enough.
* Brad: Our plan for stable, is that once the Tock registers update that will affect the CSR stuff, and then we'll chip away at that feature too.
* Branden: And it is nice to have some things on nightly and some things on stable, so we're testing both
* Johnathan: We also want to test stuff with MIRI, and that'll require nightly.
* Alyssa: Unfortunately MIRI won't be stabilized until the Rust memory model is stabilized, so it's going to be a _while_. More than 3-5 years out
* Amit: Comparatively, naked functions are basically dead though

## Process Checking
* https://github.com/tock/tock/pull/3772
* Amit: Phil isn't here to discuss unfortunately
* Brad: I do want to handle this, but I agree we could push it

## Libtock-C Refresh
* Amit: High-level, Libtock-C could use some love. Tyler brought part of this up
* Brad: Yeah, Tyler brought up how clumsy libtock-c is to use. It's been cobbled together at low-effort for testing kernel features. Then I realized I wasn't sure what it meant to "fix" it or how to start. So, I want to crowdsource ideas on how to improve it
* Branden: A trivial answer is documentation, which is quite lacking
* Tyler: An example, there's an assumption listed in one of the alarm files that may or may not be valid anymore. It would be great to know "what are the assumptions" about how you use APIs. For example, requirements based on the frequencies they're running at. Probably falls under documentation
* Amit: Aside from documentation, two things. First, we could consider the build infrastructure. Even with the best C tools, stuff is hard. But things like CMake are more ergonomic for relying on libraries. That could be a big deal. It's a chicken-and-egg thing where people aren't asking for this, partially because it's not easy to use right now.
* Leon: It would be great to pull libtock-c into an out-of-tree library too
* Leon: Also, we have pretty inconsistent APIs in libtock-c. I'm not sure even all kernel changes ever made it into userspace. It would be great to version individual subsystems. We basically don't have any stability guarantees for libtock-c right now
* Tyler: As part of the CI tests for libtock-c, we build things, but I don't think we actually have unit tests for anything.
* Leon: There's one very limited one. The Tock CI tests an ancient version of libtock-c for the litex board
* Amit: The second one, was that it could be worth separating the libtock-c which we currently have which is a useful proving ground and isn't particularly good for real applications from an alternative userspace. We could have a new one designed from the top down. Although there's a question for who would do that
* Brad: Where do those interfaces come from?
* Amit: There _are_ users that are building C apps. They don't currently contribute upstream. I'm not sure what their libtock-c stuff looks like or how it works. I just know that they are building applications in C for Tock.
* Amit: In general, the interface would come from applications. We might need to talk with groups who do make applications
* Amit: There is this IoT application thing now that Tyler is revisiting. C seems important in that domain.
* Amit: We could replicate some similar API from an existing system. Proton-style for example. It would be much more constrained, but that enables better documentation since there's less stuff
* Brad: So how much would be exposed and where are the pain points is the question. If a more-documented smaller API would be helpful, that's one answer
* Brad: One idea I had was just reorganizing things. Making a folder-structure here for categories of drivers seems useful
* Branden: The examples folder is a mess too. And there's a tests folder in there where it's unclear what goes where
* Pat: Could we match the syscall numbers which are in tables in the kernel by types. We could use those same types for our folders
* Brad: Yes. I'll look at that
* Brad: Does any of this seem like the thing that would move the needle Tyler?
* Tyler: I think it would help. As someone who came into the project recently, and I'm working with two undergrads on the open-thread abstractions, there are function prototypes where we're connecting OpenThread to Libtock-C. The biggest pain-points are the documentation, but specifically and worse the inconsistencies in APIs and places where the kernel has changed and we just get some generic error when making a call.
* Tyler: I'm not sure if a simpler redesign would be good. It would help. But I can't decide on restart versus repair
* Brad: I think the consistency thing is a clear issue. Part of what I would hope to do with a reorganization is separate the testing stuff from interfaces that are more preferable. It's fine to have an interface to a specific IC, but we really want a Temperature interface. So we could guide people to more general, well-made APIs
* Brad: One question, are you saying some things just don't work on Master for kernel and userspace?
* Tyler: There are syscalls that are deprecated and just do nothing in the Thread stuff for example. You do get an appropriate error back at least
* Brad: That's a bug in my mind. Someone should have "fixed" libtock-c
* Tyler: I don't know if there's a way to automate this, but it would be neat to tag things in the kernel with what they associate with in userspace, so making changes in one would flag a required change in the other. The same issue will otherwise occur in a few years even if we fix everything now, as long as nothing is checking that we keep the two in alignment.
* Leon: I don't fully agree with classifying them all as bugs. One of my conclusions is that in the kernel the development of kernel stuff that guides releases goes mostly independently of capsules. We don't use capsules to inform kernel releases often. So they change and get out-of-sync. Having some relation of which capsule API we want and whatever libtock-c currently expects would be good
* Brad: Anyone is free to stabilize a capsule syscall interface, although we do so rarely. Then userspace wouldn't have to change because it's stable
* Leon: I guess. I'm concerned that's not realistic though. For example, was the alarm stuff a part of the release?
* Brad: I think we deprecated the old stuff, but left it there. I am aggressive about not breaking userspace after we agreed on stability
* Brad: Users do expect functions to work. So some way to check that the functions work would be useful. Possibly CI testing which could block PRs for kernel or userspace. I think fixing a lot of entry-level stuff would help a lot. But then we still do need to decide on what our interfaces _should_ be. I'm not sure
* Tyler: Depends on who's using it. They might define usability differently
* Amit: Hopefully we could talk to some of the users and ask questions about their use
* Tyler: One more thought, speaking to the alarm infrastructure, there's a surprising and troubling amount of bugs in the implementation. The time-as-tick PR but also I think the queue of alarms has some issues with sorting. I'm not sure it's actually a real-world issue but it could be an edge case. So I think we should be thinking about unit tests too
* Branden: I think that's a rare case though. Most drivers just rely on the kernel syscalls to do almost all work
* Tyler: The alarm has a lot of logic. Maybe there are others with a lot of logic and we should have testing for them. It is a shame if the kernel does all this hard work, and libtock-c ruins it.
* Amit: I suspect there's not much logic, primarily because libtock-c was for testing the kernel. A redesign could have more logic though
* Tyler: So moving forward, documentation push first, and I'll help there. Then we could kick-off the process of talking to stakeholders. Would be useful

