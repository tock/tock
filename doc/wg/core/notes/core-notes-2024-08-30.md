# Tock Meeting Notes 2024-08-30

## Attendees
 * Branden Ghena
 * Amit Levy
 * Johnathan Van Why
 * George Cosma
 * Leon Schuermann
 * Hudson Ayers
 * Micu Ana-Mihaela
 * Pat Pannuto
 * Brad Campbell
 * Alex Radovici
 * Ben Prevor


## Updates
 * Alex: Started working on Tockloader-rs at the beginning of the summer. George plus two students worked a lot on it. So, I wanted to introduce George and Micu Ana-Mihaela who are joining today to discuss it.
 * Branden: No Network WG updates this week
 * Brad: No Open Titan WG updates this week
 * Pat: There's a survey from the embedded Rust working group that everyone should fill out! Good for sharing pain points with the Rust community. https://blog.rust-lang.org/inside-rust/2024/08/22/embedded-wg-micro-survey.html
### Stable Compilation
 * Hudson: PR to remove the ASM const nightly feature. At that point, out-of-tree RISC-V boards can compile on stable.
 * Leon: What are the remaining nightly features?
 * Hudson: Some code size features and custom test frameworks. I don't think there's anything left in kernel, arch, or capsules
 * Brad: Confirmed
 * Leon: That's cool so we could really choose stable or nightly on boards
 * Hudson: We're basically there
 * Brad: Can you choose a RISC-V board to make stable upstream?
 * Hudson: Yeah, in a follow-up PR. One that doesn't use the custom test frameworks (not Open Titan)
 * Leon: What would we want to target?
 * Pat: Maybe the HiFive rev B? It's real hardware
 * Hudson: I don't think it matters if it's widely available. The HiFive board is pretty limited on size, so maybe we can't remove those features
 * Leon: I ordered the RP2350 Pico 2 and received it. They have switchable ARM and RISC-V cores. They don't support the PMP mode we need right now, but I think we're going to have to switch to a power-of-two protection system for RISC-V since others aren't supporting it either. I intend to try a quick port of that. That could be a target
 * Brad: On the features we use. We don't technically use naked functions anymore, but we'd sure like to
### CI Warnings
 * Leon: We also found a ton of warnings we're ignoring in CI. I have a half-working solution to deal with that and I have a PR for this. They're warnings that only show up in test configurations. We need to overwrite rust flags to deny warnings, but that gets rid of the cargo config. But we also don't want to modify a file in the filesystem, because that will affect other builds. So we want to overwrite the cargo home environment variable to use a different path for the alternate config file.
 * Hudson: To zoom out a level, we've never enforced deny warnings for tests. But during the switch from make to cargo, we accidentally started allowing warnings in CI for a bunch of different builds. So, warnings have crept into non-test builds.
 * Amit: Many of the test configurations are difficult to run locally. For example, all of the warnings that have to do with shared mutable statics should have been gone with our hack around those, but they show up in test configurations because it wasn't clear how to run that locally.
 * Leon: You can build in the test profile locally, and they show up
 * Amit: Even that's not trivial. There are some magic incantations to get it to compile
 * Leon: Agreed. And another issue is that some tests could have been written without global statics, but they currently use them everywhere. It's a multi-day effort to fix these. They aren't unavoidable like some Tock usage, they don't need to be this way.
 * Hudson: I agree with Amit. Every time I have to build those QEMU setup targets, something goes wrong.


## Outstanding Pull Requests
### Update UART to Match TRD
 * https://github.com/tock/tock/pull/3256
 * Leon: It's something we want, but it's a ton of work. Mechanical changes that are hard to test because they touch ALL boards.
 * Amit: Does this need a review? Or more work
 * Leon: This is an effort that needs to be restarted from scratch.
 * Hudson: This is already the second PR here with an attempt here, but it keeps falling out-of-date before it's worked on. The rebase would be huge
 * Amit: So what do we do? It seems that perhaps this PR should become a draft or be closed or something?
 * Leon: I agree. This PR in itself isn't useful. The code changes aren't applicable anymore. But we ultimately do want to make changes to match the implementation and the TRD. We need some forcing function to get someone to invest time into this
 * Pat: Is this something we could put on Ben's queue after testing? It's an engineering task of low priority
 * Amit: Maybe
 * Amit: Okay, for now, let's convert this to an issue
 * Leon: I'm doing that
### Allowed Process Slice Buffered API
 * https://github.com/tock/tock/pull/4023
 * Brad: We have a design, we just need someone to implement it
 * Amit: Okay, so this could be waiting-on-author
 * Amit: Is this related to the buffer management stuff from Network WG?
 * Leon: No. Not related to PacketBuffer work, that's in capsules. This is userland-to-kernel data transfer, which is still important, but different
 * Amit: And what's the status here, now?
 * Brad: The design isn't quite implemented yet. What's in the PR could probably be used as a basis for the implementation
 * Hudson: I'll make a comment here
### Dynamic Userland Application Loading
 * https://github.com/tock/tock/pull/3941
 * Brad: Yeah. This one is _pretty close_ finally. The last piece is tracking where there are apps installed to find a valid spot in a way that's robust. The current approach makes some assumptions.
 * Brad: The person working on this has been off this summer, but hopefully we can resolve this soon
 * Hudson: Added waiting-for-author to it
### Non-XIP Flash Info Document
 * https://github.com/tock/tock/pull/4081
 * Amit: This is actually in a pretty good state. This is a document outlining the wants and scope for supporting non-execute-in-place platforms in Tock. The main issue is that we haven't gotten feedback from the people who are using non-XIP platforms in practice. We discussed at Tockworld, but haven't gotten comments since the creation of this doc
 * Amit: We could block, or we could merge as-is?
 * Leon: I wanted to review this to read the updated version. I think we should have feedback from the actual stakeholders
 * Johnathan: I'll ask OpenTitan if they have time to look at it
 * Amit: Marking it as Blocked for now and commenting
### Bus Library Support Address Widths
 * https://github.com/tock/tock/pull/4099
 * Leon: I think this isn't stale actually. We have concerns and Brad commented that the semantics are unclear on some parts of the implementation. I think the discussion needs to be resolved first
 * Brad: Right. This will probably work as-is, but not be clear about how to use it
 * Leon: I still think the types here aren't entirely appropriate for the goal
 * Hudson: The most recent comments are earlier this week, and no commits since then. I think the appropriate label is actually waiting-on-author


## Tockloader-rs
 * George: I've worked on Tockloader-rs last summer, and several students have worked on it this summer to rework it from scratch. One of the students is on the call today
 * George: We've made significant progress. We have a fork with efforts. Have info, list, and listen working. The listen is based on the work of another person, which implemented the ability to listen to multiple apps simultaneously
 * George: Another significant change in the design is that we moved away from OpenOCD/J-Link and moved to probe-rs. That's made our lives significantly easier
 * George: The listen interface we demonstrated with an interactive console at Tockworld
 * George: The list and info commands work now through probe-rs. We want to send a PR to the main repo soon. This is still a work-in-progress.
 * George: Next is a working "install" command
 * Amit: Is there a path for functionality through things like the Tock bootloader? Rather than probe-rs? Boards that don't have an SWD interface?
 * George: No plan on this yet. We tried to reduce the scope to start.
 * Alex: Some boards have a Tock bootloader. I do think it would be important to support that protocol. It wouldn't require a third-party library, so I think it should be on the Roadmap and is very possible
 * Alex: However, there are no plans for OpenOCD or J-Link support
 * Amit: We think probe-rs will work for all those cases right? The only downside is the speed?
 * Alex: There might be brand-new boards that probe-rs doesn't support yet. But they are updating it quickly
 * Leon: Once we support more than one interface, presumably there will be some abstraction in the code base. Even if it's not on your roadmap, do you think it would be possible to have a debugger backend?
 * George: Sort of. One part is the CLI and parsing data. The other crate is tockloader-lib which allows interfacing directly with Tockloader-rs. Probe-rs is very heavily integrated with this right now. We'll have to think about the abstractions.
 * Alex: I do think it would be important to have other backends. Particularly tock bootloader. So someone else could add it if they wanted some time
 * Amit: Ultimately, there would need to be an interface to support anything that's not probe-rs. So I suspect once you support both probe-rs and tock bootloader, other things will be possible too
 * George: In the case of the Tock bootloader, we'll use serial directly, not through probe-rs
 * Brad: I want to add here, I'm always a little concerned that probe-rs could disappear one day. If there's a good interface for swapping out backends, that's great. But it's hard to trust that probe-rs will still exist a decade from now.
 * Alex: We'll discuss offline
 * Branden: Entirely different question: Are the info and list commands outputting exact matches with the python version of Tockloader?
 * George: Yes. There are still some minor formatting differences, but the data displayed matches


## Treadmill CI
 * Leon: Been hard at work getting the system ready.
 * Leon: Doing a live-share demo. Doing an example of Github workflow on push event. Runs a job on Github infrastructure to develop a test strategy to select subsystems to test. That'll send a request to start a Treadmill job. Github talks to a server running our coordinator. That'll talk to our server at Princeton with a board connected over USB. Uses `make flash` to interact with board.
 * Leon: We're starting to get to the phase where this whole setup works more-often-than-not. So, we can give people access to the system by giving them workflows they can push to, or with interactive access to Treadmill for developing tests
 * Leon: Question for the broader audience on how we should move forward from here
 * Hudson: I notice you're talking about giving people access so they can develop tests. Would it make sense to have just one or two tests to run in CI to start?
 * Leon: We are working on that. Ben has been working on writing Python infrastructure that can compile and flash applications. We had discussions in the Matrix channel for this where we talked about different formats for defining these tests and their formats. We really want to get expertise on tests from everyone else, rather than just make stuff ourselves. Ben is working on a basic "hello world" test right now
 * Ben: So on commit, we can get all the way to real hardware. So now we're focusing on writing tests to go on that hardware
 * Branden: This is awesome. So what do you want from all of us now?
 * Leon: We can extend our scripts to encapsulate more behavior and write some basic tests. We chatted about implementing some testing things, for example, writing them in a language like Python instead of Rust so they're more accessible to others
 * Leon: So, for now, we're sort of looking for what the next steps are for others to use this
 * Hudson: I'm curious what kind of other host access on the machine we're going to make available to interact with the boards? I assume to start, nothing. It's just a single USB attached to the board and that's it.
 * Leon: I want to have RPis connected which will have GPIO connections to the boards. So you could use scripts to interact with the GPIOs on the RPis to simulate a button press. Ben has been developing on those, and the interface is the same for those as for QEMU. So this workflow should translate well. The goal behind the platform has been to make it very easy to run on different platforms. You should be able to copy-paste one workflow into a new workflow with new tests
 * Leon: So, one concrete proposal for now. Do people want to have access to play around with this and see what it can do? And should we have some kind of meeting to iterate on the testing scripts?
 * Hudson: I think the easiest way to get people to look at stuff and iterate: the sooner you have something using one of these scripts in CI, people will look at it. I do think we could have a synchronous meeting about it first.
 * Leon: I think making this a required CI check isn't ready yet. But we could make it an optional check pretty soon here
 * Hudson: Yeah, that would be great. See what kinds of failures we get
 * Branden: It would be great to give us some pointers to this stuff. Where the code lives for instance?
 * Leon: We are making a guide too. A book for it.
 * Branden: What about a short tutorial session? That could be the synchronous meeting, show some people how to use this so they can start trying it
 * Leon: Okay, we'll do some work and plan on a separate meeting/tutorial in the coming weeks


