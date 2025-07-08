# Tock Meeting Notes 2025-01-29

## Attendees
- Branden Ghena
- Brad Campbell
- Leon Schuermann
- Pat Pannuto
- Johnathan Van Why
- Amit Levy
- Tyler Potyondy


## Updates
 * Brad: LoRa module support in libtock-c. PR4320 in the kernel for support. Now working on libtock-c side of it. It's not perfect and I know it, but I have students who want to move forwards with it and I'm hoping to get support to upstream it
 * https://github.com/tock/tock/pull/4320
 * https://github.com/tock/libtock-c/pull/490
 * Amit: You're talking about the libtock-c stuff
 * Brad: Yes. The kernel stuff is a bug. The libtock-c is what's possibly controversial
 * Leon: To test this, you only need one board right?
 * Brad: Plus some LoRa infrastructure.
 * Leon: Okay, I'll reach out. I'm interested in setting this up and adding it to CI
 * Tyler: Is this over SPI or part of the SoC?
 * Brad: This chip, the WM1110 has an nRF52 plus a LoRa module. So it's treating LoRa as a SPI peripheral
 * Amit: This _should_ work for any board with an LR1110 over SPI. So it's not just one board, it's just tested on one board
 
## Network Working Group Update
 * Leon: We have this Tock-Ethernet branch that I've rebased. Things are working there. It's been lingering but is in a good state. So now we're working to move stuff upstream, then move it to PacketBuffer after
 * https://github.com/tock/tock/pull/4324
 * Leon: We've been discussing on a PR how to do this. My proposal was to take atomic changes and make a series of PRs out of them. In the end to merge all of tock-ethernet into mainline. We could alternatively do one big PR, but that would be like 7000 lines, so it doesn't feel scalable
 * Leon: The first PR for now is just an Ethernet HIL that is known working
 * Brad: I get the point about the huge diff. You're not going to get any reviews on most of those lines of code if we go that route
 * Brad: We could open the series of PRs right now to give good context. Or we could merge into a staging branch individually and then do a big PR of that whole thing. What I want to avoid is getting in a case where we get distracted and merge some things but not all things
 * Leon: Yeah. I'm worried that a huge PR would linger too. We could make that staging branch and move stuff in over the next couple of weeks. But I want to make sure that we'll agree to merge the staging branch into mainline without a huge fight
 * Brad: I think it is important that we can treat contained subsystems, like networking, differently. We moved fast with 15.4 stuff, and I think we can do the same here
 * Leon: It's mostly chip peripherals, capsules, and a single HIL
 * Pat: I would expect people on the Networking WG would approve the big PR right away once we get to that point. And be clear that it's not actually something new, it's something we've already worked on
 * Leon: Great. As long as we're all on the same page I'm happy
 * Branden: And network working group has been making sure that we're happy with these smaller PRs before getting the larger group involved
 * Branden: Last update, we talked about UDP which has a bug. We decided it makes sense to band-aid fix it even if we don't love the interface, rather than doing a bigger rewrite.

## Stabilizing Isolated Non-Volatile Storage
 * Brad: Something that's come from tutorials and real apps is the need for persistent state storage for processes. OpenSK invented something for this for them. We want something that supports all of these use cases
 * https://github.com/tock/tock/pull/4258
 * Brad: So, I want to see us merge this interface but I also want to stabilize it so apps can rely on it. It's been a long time since we've stabilized an interface in Tock though, so I want to know what the path to that is
 * Brad: Specific interface to stabilize: https://github.com/tock/tock/blob/3c513688d48f32438d83240ed611e4104463dfc4/doc/syscalls/50004_isolated_nonvolatile_storage.md
 * Amit: Tough. One version is that we stabilize things when there's been enough experience with it that we'd be confident not changing it until the next major release. The problem with that is that we sort of never get there. So since the first batch, we haven't stabilized anything new
 * Amit: An alternate version would be being more aggressive. A criteria, which would apply to this interface, is that for things that are relatively central interfaces, like storage, there is high value to stabilizing it. And then we'd want to go at least one round of trying it out before stabilizing.
 * Amit: So go, maybe a month trying this after merging? And if there are breaking changes we reset the clock.
 * Pat: We are should discount the general experience the people writing these have. We are assuming right now that our stable interfaces are greenfield, but maybe we should have a section of the documentation comparing them to other interface in POSIX or other systems. That would give us more confidence in it.
 * Brad: Definitely. That's a really good idea, and we want to avoid mistakes others have made previously
 * Brad: We are stabilizing a syscall number and not the only use. We could add new interfaces later
 * Pat: We could end up with create, create2, create3 for example though
 * Amit: Well, there's a difference between the system call interface and the libtock-c interface for it. And, it seems likely that you'd have two different userland libraries for different version of how to interact with non-volatile storage. They'd probably look totally different, or look identical but secretly have different syscalls underneath
 * Pat: You could still end up a few similar syscall options, and some are deprecated and some are choices
 * Amit: I think of it as different filesystems. The system is probably just using one, but a system designer could pick among them. FAT32 is kind of deprecated, but you can still use it and things still work today
 * Branden: Both of those are plausible scenarios. We might have a whole different way of using it, but we also might have an argument that we want to change.
 * Brad: The libtock-c driver implementation does give us more flexibility, as it could hide some things about the exact system calls used
 * Amit: In order to gain insight and proposed changes to the interface, people have to use it. If no one else uses it, then whatever you've got is already right (tautologically). If it's not stable, then there's a concern to me that it will stay un-interacted with because people are reluctant to write general-purpose things for it. Maybe not, but we don't have a good counterexample
 * Brad: I certainly think it's easier to propose to downstream users an interface that has the little checkmark
 * Amit: We could imagine having an intermediate state that like "on the way" to being stable. Maybe with a deadline too. We encourage people to consider it and play with it, and it's probably going to be the final version. That might help solicit users or be enough time to gain confidence
 * Brad: I like that. I think that's a good idea
 * Branden: Is that stabilization timeline just the next minor release of Tock?
 * Brad: It does seem practical to tie it to a release
 * Amit: I'm worried that we release too slowly for that. Maybe faster now
 * Branden: I see this as incentive for releasing
 * Amit: So this would be like a feature flag in Rust. Not enforced similarly, but on the path
 * Brad: This is a sensible way forward to me. I propose that I'll write up a statement about this so we can track it. We can discuss more on that PR if needed
 * Branden: Back to the original PR, I think you have broad agreement that this is a valuable interface, so it _should_ be stabilized

## Treadmill CI
 * Brad: Forgive the ignorance, but it occurred to me that wireless is tricky because we don't know if it's our code or the other end that's broken. I was wondering if there was a roadmap for testing the path, and if that fails retry the stable master code as a comparison and way to avoid false alerts. Not sure if that's a meaningful concept in "testing"
 * Leon: That's a reasonable thing to do. A question is what action to take if Master fails
 * Leon: Hardware testing, and timing, are just generally unreliable, so we might retry tests anyways. That might resolve some wireless intermittent issues
 * Brad: I'm thinking of examples where, if it fails on Master that's okay. That would make it the fault of the other thing, not a Tock thing
 * Leon: Right now, the 15.4 tests we control both sides of the system. We don't have any test that relies on external infrastructure yet
 * Leon: The cool thing is that we can have all kinds of updates, the testing script is just python and we can do arbitrary things
 * Johnathan: Every large project I've worked on has some way of dealing with "flakey" tests. Most of them run a subset of their tests on PRs. It tires to run some relevant tests and some smoke tests, but not the full corpus of tests. The full corpus runs in the background daily. Failures in the daily will flag failures in the PRs as possibly not relevant
 * Leon: That's valuable insight. It's funny that we are provisioning for those kinds of things already. In the workflow, we have two stages https://github.com/tock/tock-hardware-ci/blob/main/.github/workflows/treadmill-ci.yml In the first step we have access to the diff for the changes and select the tests. The second step farms out tests. I don't know which tests to run on which platforms for which changes, but that's an option for the future.
 * Leon: It might be good to have a working group or task force for a testing strategy about which tests to run when, and also what to do when a test fails
 * Johnathan: I suggest that as long as the software infrastructure has the ability to choose tests based on PRs, we can delay that decision until we're actually under test pressure. That'll let us gain experience with the tests
 * Leon: We are currently not under test pressure because we have a bunch of nRF52DKs. In the future, we'll have one or two of many different boards, which may cause test pressure. So this will come up in the near future 
 * Amit: I suggest Leon and I create a proposal offline for what needs to be settled in a future discussion, possibly a small group discussion instead of this call

