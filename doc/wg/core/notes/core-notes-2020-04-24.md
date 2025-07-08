# Tock Core Notes 4/24/2020

Attending:
 * Alistair
 * Amit Levy
 * Andrey Pronin
 * Brad Campbell
 * Branden Ghena
 * Garret Kelly
 * Guillaume
 * Hudson Ayers
 * Johnathan Van Why
 * Leon Schuermann
 * Pat Pannuto
 * Philip Levis
 * Samuel Jero
 * Vadim Sukhomlinov

## Updates
 * Johnathan: https://ferrous-systems.com/blog/zero-sized-references/ was just
   posted to Reddit, will be taking a look. May be similar to my libtock-rs
   work.
 * Hudson: Is that for references to zero-sized types?
 * Johnathan: That is for zero-sized types that act as references, I believe.
 * Amit: On our end, we are getting closer and closer on an emulator. Garret, we
   should sync up. May be highly aided by Hudson's scheduler work.
 * Hudson: Cool, I'd love to hear why.
 * Amit: I think Leon shared on Slack a prototype of an Ethernet driver.
 * Leon: Yeah, so I've implemented a simple SPI Ethernet chip. Looking forward
   to getting contributors (who need Ethernet) to discuss HILs/interfaces.
 * Amit: I managed to ping it over IPv6.

## Tock 1.5 Status Update
 * Amit: Brad, can you give an overview of where we stand?
 * Brad: Bugfixes are coming in. Handful of boards testing, including Hail,
   Imix, Nrf, and OpenTitan. I've done a bit of testing on HiFive 1. No major
   issues so far, a few fix-ups. Roadmap from here to release includes a couple
   of pull requests that we need to discuss to decide whether to include them.
   Need to test on more platforms and decide which platforms are essential and
   which are not. The core team does not have every board.
 * Amit: Guillaume, you mentioned you would try updating OpenSK to the release
   candidate. You then ticked the checkbox. Does that indicate OpenSK works on
   the release candidate?
 * Guillaume: Yes
 * Amit: Do we want to have that discussion, Brad?
 * Brad: Yes, so one is the virtual UART that Phil and I have been working on. I
   just pushed new changes this morning. I think if Phil has a chance to look
   those over that would be a good candidate to include. Trying to remember what
   other...
 * Amit: Now approved by Phil, according to GitHub.
 * Phil: There is a virtual UART test. It would be good to run that as a sanity
   check.
 * Hudson: I can run that on imix and will post the output.
 * Brad: I'm sort of realizing the pull requests I have in mind are my own. The
   other one is #1790, which has to do with the process console and addresses a
   minor issue with print ordering. Makes the output look more as they are
   expected. That one is sort of not that important, but makes the console look
   nicer.
 * Phil: If people have tested without this, we would need to restart testing.
 * Brad: Agrees
 * Amit: The virtual UART seems worthwhile. The second seems less worthwhile,
   unless it involves the same tests as for the virtual UART.
 * Brad: I don't know if it would be the same tests. Maybe it's better just to
   wait, not that critical.
 * Amit: Okay
 * Brad: Are there any other PRs? I've been trying to merge things that are
   clearly testing-related bugfixes. I don't see anything else essential for
   1.5.
 * Hudson: I will send a PR soon. While doing Clippy stuff I identified an
   unsound function. Is that a bug that we will want to fix? The solution is to
   mark the function unsafe, as it is only called from the panic handler anyway.
 * Brad: That seems like a reasonable thing to include.
 * Hudson: I'll get that pushed.
 * Brad: The other question is what platforms can we test and what platforms we
   want to test.
 * Brad: The boards in question are the STM boards and the LaunchXL board.
 * Amit: I believe I have the LaunchXL board at home.
 * Phil: I have a LaunchXL but I can't get it going, because the instructions
   ... done't work.
 * Amit: LaunchXL is pretty brittle
 * Phil: It doesn't compile.
 * Amit: I'm putting a TODO to try and fix it. I don't think I have the board
   [changed mind], but I'll try to get it to build.
 * Hudson: Experienced same issue as Phil a few months ago.
 * Amit: What is the other board?
 * Brad: The STM32 is a big question mark.
 * Amit: None of us have that. We can reach out to some folks to test. We can
   also make sure it compiles. The ACD52832 I don't expect problems but I
   definitely brought one up and I can test on that.
 * Brad: We have the RDE21, and I don't have the FPGA it supports, and the
   HiFive1 seems to work okay on QEMU. I tried a lot yesterday and could not get
   the HiFive1 port to work on the hardware. It is a bit demoralizing because it
   is already an outdated board that is not really worth fixing.
 * Alistair: Did you get anything to work on the board?
 * Brad: I got the kernel to run, sort-of. Most of the time it seemed to be
   stuck in an interrupt loop (always a pending interrupt). If I disable
   interrupts, then ignored the PWM interrupts, the kernel seemed to behave as I
   expect. Without manually disabling PWM interrupts I could not get it to work.
 * Alistair: Okay
 * Brad: There is a bootloader running on the board; I think it can get into an
   unknown hardware state as far as the kernel is concerned. In theory it is
   getting cleared but that doesn't really seem to work. Content to leave at "it
   works in the simulator".
 * Amit: Yeah
 * Alistair: That's fine
 * Amit: Has anyone tried reproducing my instructions for the emulated -- did it
   work?
 * Brad: I used make and it did work.
 * Amit: Okay
 * Brad: I changed it so we do a bit better testing. QEMU can get the bytes out
   faster.
 * Alistair: For the HiFive, it stalls until the print is completed.
 * Brad: Gotcha. Having two prints in the kernel requires the interrupt to
   trigger to print the second. Let me summarize: Amit will press the ACD52832.
   Phil might test the LaunchXL.
 * Phil: If Amit doesn't have one and I can get mine going, yes.
 * Brad: Amit, you will ask someone to test the STM32 boards.
 * Amit: Yes
 * Brad: but we will not block on STM32. I have made the comment on the 1.5
   release with checkboxes. When you are happy, check the box.
 * Amit: The release will just mean changing the RC2 tag to the release tag.
 * Johnathan: If I am done testing the boards I can test on, but I see PR
   bugfixes waiting to be merged, should I check the box because I've done all I
   can or should I wait because it doesn't appear to be ready?
 * Brad: I don't know. I think you should check it if you would be happy if the
   most recent release candidate would be a release. I think the answer to your
   question is no.
 * Amit: We're checking off the tagged release candidate, not master or things
   waiting to be merged.
 * Amit: Any other thoughts/hopes/concerns?

## CI Infrastructure (Travis to GitHub Actions?)
 * Amit: Hudson and Leon wanted to discuss switching some or all of the CI to a
   combination of GitHub Actions and/or self-hosting.
 * Hudson: Leon and I spoke a couple days ago. The reason we originally spoke
   was the context of my PR that added the size reporting. It doesn't work when
   PRs come from forked repos. The fundamental issue is GitHub makes it
   impossible to do this from forked repos unless you are doing some amount of
   validation to ensure the PRs cannot exfiltrate the credentials. We were
   discussing doing the size reporting and making it work for forked repos. In
   the context of that, we discussed how using GitHub Actions instead of Travis
   may make things easier. Noted recent issues with Travis splitting into .org
   and .com, incompatibility between .org/.com, and anecdotal reports of Travis
   firing lots of employees. Also heard GitHub Actions is faster and more
   capable. There's some separation between concerns. One concern is doing
   self-hosting to enable on-device tests. Another is migrating from Travis to
   GitHub Actions. Wanted to get feedback from the group on those two options
   before we undertake them.
 * Leon: We don't necessarily need to do a direct move to GitHub Actions. We can
   allow it to run alongside Travis and/or self-hosting. Can use GitHub Actions
   for building/sandboxing and self-host board execution.
 * Amit: What do people think?
 * Guillaume: We started using GitHub Actions for OpenSK in parallel with
   Travis. GitHub Actions has much lower latency because it can parallelize, but
   we sometimes have quota/deadline issues and some fail, especially on Mac OS.
   May not be able to migrate everything immediately, but worth experimenting
   with.
 * Hudson: The approach would initially be to run GitHub Actions in parallel
   with Travis.
 * Amit: Not a super high bar.
 * Brad: Our CI is so minimal in terms of real testing that this seems like a
   relatively easy change for us. I think one of the biggest things is running
   the formatter and doing compilation. If we can mirror that this seems like a
   fine change.
 * Amit: Hudson and Leon, can you recount what we would want to do on a
   self-hosted machine.
 * Leon: When we run something in GitHub Actions and want to generate reports to
   attach to the PR then we cannot attach them to the PR from forks due to
   credential issues. Instead, host our own webhook that downloads the GitHub
   Actions result, runs it, and posts the statuses. Can use this to run binaries
   on boards too.
 * Alistair: You would need an internet connected and exposed server to do that.
 * Hudson: Yes
 * Leon: I have no problem hosting, but it is a trust issue. Haven't thought
   about it.
 * Amit: Have resources where we can spin up VMs or dedicated machine, or could
   get cloud credits. Hosting a VM is solvable.
 * Alistair: I wanted to make sure I was understanding what was going on.
 * Leon: We don't have any intention to host CI ourselves, just some
   infrastructure around it doing some automation.
 * Hudson: In the long term, the ability to flash Tock on hardware and run tests
   on the hardware is exciting. Would like to be able to automate release
   testing.
 * Leon: This would be a step in the right direction.
 * Amit: We can stick a box somewhere at Princeton with hardware connected to
   it.
 * Brad: What happens to bors?
 * Loan: This was one of our concerns. Bors' requirements are configurable.
 * Pat: GitHub Actions and Travis are both explicitly supported in Bors, and we
   can configure what it needs to pass.
 * Amit: Sounds like everyone is in favor.

## Better kernel testing
 * Amit: Because Phil had to leave I suggest we push this to next week and will
   discuss this early in the meeting?

## Kanban board
 * Leon: I wanted to discuss the Kanban board I want to implement. There are
   ways to have a GitHub project associated and linked to tracking PRs so we
   would not have two sources of truth. Ultimately, am asking how we should
   continue -- should I open an issue to discuss this formally, or do we want to
   implement now and see where it goes?
 * Amit: I suggest we go for it unless someone has reservations about doing
   that.
 * Brad: If there is enough motivation to try, I think that's great. My
   reservation is always that a little bit of buy-in means it is destined to
   fail, and is a waste of time. Needs a critical mass to succeed. Recognize
   that my view is pessimistic and perhaps optimism is a better strategy here.
 * Leon: I can volunteer to maintain it at first, it's low effort.
 * Amit: Let's try it out.

## Tockloader 1.5
 * Brad: Tockloader 1.5 will be released very soon.
 * Amit: I've been using that implicitly. Thank you.
 * Guillaume: We've been relying on latest master Tockloader in OpenSK, so
   thanks for releasing 1.5.

## Automated QEMU testing
 * Alistair: I wanted to add automated tests in QEMU for OpenTitan and HiFive to
   test for regressions without physical hardware. I want to add it in Travis
   like in libtock-rs -- should I wait for the GitHub Actions work?
 * Amit: I doubt it will be much different.
 * Leon: I think it's just the system dependencies that are per-CI.
 * Alistair: It is just downloading and building QEMU. I'll add that to Travis
   for now and we can migrate later, or add it to the Makefile?
 * Hudson: I think it is reasonable to add it to the Makefile then call that
   from Travis. Then it will automatically move over to GitHub Actions.
 * Leon: Not sure whether it's worth it if we move in the next week or so.
 * Alistair: Should not be much wasted effort, just a few lines.
 * Amit: Is there a version of QEMU that has OpenTitan as a board?
 * Alistair: I wrote it up yesterday. I'll send the patches and hopefully it
   will be in mainline next week.
 * Amit: Very cool
 * Leon: I am currently doing SPI for the HiFive1 in QEMU and have created a PR
   in Tock for that. Would you mind helping me do QEMU development.
 * Alistair: Email me, I'll help you.
