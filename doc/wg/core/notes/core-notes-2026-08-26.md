# Tock Meeting Notes 2026-08-26

## Attendees

- Branden Ghena
- Johnathan Van Why
- Leon Schuermann
- Pat Pannuto
- Alexandru Radovici
- Amit Levy (joined @10m)
- Brad Campbell (joined @30m)


## Updates

 - No updates today


# Release Planning

Let's release a version

## Notes

 - Branden: It feels like a good time for a release, but would like to open the call on whether there are any PRs that should make it into the freeze.
 - Leon: I don't know that there is any one PR that is critical to hit now. But we are in a lot of high velocity efforts—RiscV64 updates, IPC updates. I am concerned about taking an arbitrary snapshot and calling that a release.
 - Leon: Testing is also something we should talk about (next)
 - Branden: I think we would want to do a freeze, and triage which PRs to pull in and which to defer
 - Branden: But I think the efforts are in a better place than you think — RiscV64 is all merged (except the TRD) and IPC is all unmerged. So we are actually at a good snapshot point.
 - Leon: That sounds much better than what I was picturing.
 - Branden: Other things that are moving, lots of QEMU efforts and STM32U from Alexandru's team — what is the status of STM32U series, is that in a good place?
 - Alexandru: I would suggest we try to get several things merged before release: clocks / clock configuration, which has dependencies with other PRs. I think within a week they can all be merged. Clocks is the big blocker to everything else. SPI, I2C, and maybe a CAN.
 - Branden: It would be good to update the release tracking issue with the blockers: https://github.com/tock/tock/issues/4584
 - Johnathan: What's the usage for this chip, 
 - Alexandru: OxidOS demo kit; would like this chip in this release
 - Alexandru: What's the motivation / pressure for release now?
 - Johnathan: One reason is for the tock-registers 2.0 work, which would leave things in flux for a long time, so it would be nice to have a Tock release before that
 - Branden: Plus it's just been a while (almost two years) since the last release
 - Branden: What is the timeline for the chip supported well enough for release? Are the missing PRs?
 - Alexandru: By the longest extent, end of September. We have RTC, crypto..., the biggest missing thing right now is flash support
 - Alexandru: The reason I really want this chip is becuase it's an ARMv8, with the new MPU; and it's the only STM with equal-sized flash pages
 - Alexandru: Plus it only costs 10 euro, and it has an on-board debugger — it's great for any tutorial; the board is super sturdy; students haven't fried one yet :)
 - Branden: Which board is this exactly?
 - Alexandru: STM32 Nucleo-U545RE-Q ( https://www.digikey.com/en/products/detail/stmicroelectronics/NUCLEO-U545RE-Q/22106570 )
 - Johnathan: Candidate Plan: Alexandru and team focuses on STM32U, we run parallel branches ...
 - Pat: We need to answer Branden's QEMU before we make a plan
 - Branden: What's the motivation for QEMU?
 - Pat: It's really useful to have a software-only end-to-end test, both for CI and increasingly for AI agents to do close-loop testing while they work
 - Pat: Brad's filling in coverage on RiscV, and I'm filling in converage on ARM so we'll have software boards for all arches
 - Branden: But this is missing v8m?
 - Pat: The first PR is just cortex-m3/m4, but ARM has a really nice emulation target here where the same peripherals / memory map get hooked to different cores; if you look at the draft PR you'll see there's a feature selecting between m3 and m4 and the only change is the vector table; should be a really trivial extension to the remaining cortex flavors
 - Branden: Cool.
 - Amit: What's the motivation / big win with more arches?
 - Pat: We've had one QEMU in CI for a while, but that doesn't let us exercise arch-specific code reliably; this gets us to being able to test much more of core kernel bits
 - Amit: That's overselling a bit, can't do peripherals, radios, etc
 - Pat: Yes
 - Amit: But can do all the arches, etc; okay sounds reasonable
 - Leon: I'm in favor of this as someone who's been pushing for a while on QEMU
 - Leon: I do want to caution that QEMU != Hardware; there are lots of simplifications (e.g., RISCV traps on unaligned accesses; QEMU lets them fly).
 - Leon: Tl;dr QEMU doesn't replace hardware tests (but are good to have)
 - Brad: [catching up] In sum I think we just need to do it; too easy to wait until the right thing lands to test
 - Leon: Yes, but want to maximize use
 - Brad: Right, can we think of what's in a bad/broken state now
 - Brad: And it probaby does make sense to deviate from the "block the world" master approach, and have a release-2.3 branch
 - Alexandru: We can survive without STM32U in release if we have to
 - Brad: What's the status of the clock PR? I'm ready to merge that
 - Alexandru: There's some asserts and panics and such that need be cleaned up, but they should be fixed
 - Amit: Back to the main narrative, we were talking about features that we want in the release:
 - Amit: QEMU: Should be wrapped up shortly?
 - Pat: I think it's fair to say that within 7 days the RISCV and ARM QEMU boards will be merged
 - Amit: So, do we want to have a feature-gated release or a time-gated one? (i.e. rolling release model)
 - Leon: I am in favor of deadlines (esp. for testing), but I don't think 24 hours is reasonable. We don't need to rush it so much.
 - Leon: If we have a fairly good automated coverage and testing workflow, a rolling release model would be a nice thing to have. We otherwise haven't been good about reaching snapshot of features style releases.
 - Leon: Now that's said acknowledging that the testing harness being down is on my plate.
 - Johnathan: What is the status of treadmill?
 - Leon: Treadmill is in a place where we can deploy it again imminently with the existing testing scripts.
 - Leon: My goal at minimum would be to run the nrf52840dk exhaustive suite.
 - Leon: Long term, I would like to revisit how this is done.
 - Leon: At the moment, treadmill is not running in CI.
 - Amit: Back to the question of what kind of release, feature or time-gated?
 - Pat: I like the sound of feature-based, specifically:
     - RV64 is in (and done); TRD is intentionally out
     - IPC is out
     - tock-registers 2.0 is out
     - QEMU boards are in
     - STM32U is in
 - Pat: With the expectation that QEMU and STM32U will be finished by this time next week, and we cut a release candidate next week
 - Brad: I like the sound of feature-based, but feature lists keep changing...
 - Pat: No, my point is we are setting the feature set here and now.
 - (Consensus)
 - Branden: There are lots of other little things moving, e.g. safety comment PRs
 - Pat: Yeah, and I think we can merge little things between now and next week, when we freeze and test
 - Pat: Concretely, we have two outstanding features (QEMU boards; STM32U), when those land, we will freeze and test; we expect them to land by the meeting next week when we'll implement the freeze; we can discuss pushing timeline for features next week if needed, but we will not push deadline for long
 - (Consensus)


# RISC-V 64-bit TRD

## Context

TRD describing the RISC-V 64-bit syscall interface.
https://urldefense.com/v3/__https://github.com/tock/tock/pull/4906__;!!Mih3wA!ECFXLrjandxa_lTVomdpqQsTK6yJ9lUphDj1dC2j3uWTgueCEG_LsvL5Mxc9pS4itz64JYeBapqZWKoycQ$

64-bit support seemed big enough to require reviews from entire core team,
or at least the majority of us. This has just been sitting for some time,
and I think it makes sense to get this merged before a release.

## Notes

 - Branden: Did we want to push this to after release for some reason? I thought we would want this before release?
 - Brad: I think what Johnathan and I had discussed was intentionally leaving the TRD out and not even putting RV64 in the notes, and explicitly waiting an entire release cycle to do testing
 - Brad: I think there's maybe one person on Earth who's tested this code
 - Johnathan: I don't know if merging the TRD implies stability, but I think we want one release cycle where the ABI is able to change freely
 - Branden: It's just a draft...
 - Johnathan: We could put something about it being unstable
 - Brad: I'm not pushing it, for the reason of it sends a signal
 - Branden: I'm happy to let it linger
 - Pat: I think I favor leaving it out—yeah, there's some RV64 support in-tree, but we're not advertising this, we're not documenting it

## Decisions

1. Defer merge to after release. √


# IPC Check-In

## Context

IPC PRs are in. Handled comments from Tyler, Brad, and Johnathan. Anyone
else have opinions?
  - IPC Capsules: https://urldefense.com/v3/__https://github.com/tock/tock/pull/5015__;!!Mih3wA!ECFXLrjandxa_lTVomdpqQsTK6yJ9lUphDj1dC2j3uWTgueCEG_LsvL5Mxc9pS4itz64JYeBapprEhhE_w$
  - IPC Documentation: https://urldefense.com/v3/__https://github.com/tock/tock/pull/5016__;!!Mih3wA!ECFXLrjandxa_lTVomdpqQsTK6yJ9lUphDj1dC2j3uWTgueCEG_LsvL5Mxc9pS4itz64JYeBapqaVjOpiw$

One area of quick discussion is what a service validation interface should
look like. I wanted to get thoughts and then we can move forward with one
choice:
  - Trait: https://urldefense.com/v3/__https://github.com/tock/tock/pull/5089__;!!Mih3wA!ECFXLrjandxa_lTVomdpqQsTK6yJ9lUphDj1dC2j3uWTgueCEG_LsvL5Mxc9pS4itz64JYeBapr_arZKTQ$
  - Function: https://urldefense.com/v3/__https://github.com/tock/tock/pull/5097__;!!Mih3wA!ECFXLrjandxa_lTVomdpqQsTK6yJ9lUphDj1dC2j3uWTgueCEG_LsvL5Mxc9pS4itz64JYeBaprSM23Shw$

## Notes

 - Branden: IPC stuff is there and still waiting for reviews, though understood it's on the backburner now since it's on the other side of the release.
 - Branden: Would be nice to have a quick discussion on what the interface should be for validating services
 - Branden: Context: Say you're a service app that claims, Hey, I provide this 'useful service' (e.g. networking), it seems like the kernel should have some capacity to weigh in on that
 - Branden: There are two PRs exploring designs: #5089 and #5097, the former is larger and more general and the latter is more minimal
 - Branden: Do folks have opinions?
 - Johnathan: Looking at the type in #5097, I don't quite see how this function gets access to the resources it would need to do actual filtering -- am I missing something?
 - Branden: What resource might it need?
 - Johnathan: This was a result of the threat model discussions we had on the TRD PR? If the validation e.g. needed to show that the service name is a function of the app id, I don't think the validation could access the id?
 - Brad: It can get the short id via the process
 - Branden: But not the full id
 - Branden: It doesn't have access to the kernel
 - Brad: This goes back to the incomplete discussion we have around identifiers and verification and policies
 - Branden: I would agree with the statement that the funciton-only is half-baked
 - Brad: Since you brought this up, I think "Have Rust. Use Trait." Funciton pointers are kind of the land of C.
 - Brad: Can document them and complier can help enforce things more.
 - Branden: The reason I pushed back is because it felt like a lot of machinery for something that's ultimately really small.
 - Brad: Yeah, so why is there this size discrepency — does no one want this kind of control, or is it just that things aren't fully built out?
 - Brad: Now you can do things you wouldn't dream of doing in zephyr; syscall filters get richer, etc
 - Branden: Now you're really growing the scope
 - Branden: If no one else has opinions, I am convinced by "have Rust, use trait"
 - Alexandru: +1
 - Pat: Haven't looked yet (and taking notes today)
 - Johnathan: The Rusty-ier approach likely to be more unit testable; won't need global variable for kernel access, etc
 - Branden: I'll probably just close #5097 then.
 - Branden: Sounds good; last ask is for IPC reviews when you get a chance.

## Decisions

1. Which service validation interface makes more sense? --> Closing #5097.
2. Should IPC wait until after the next release? --> Yes.
3. Are there reviews we're waiting on for this? --> Defer to after release.


# Quick Bonus on #5041

 - Brad: Can we talk about https://github.com/tock/tock/pull/5041 ?
 - Pat: I have a sketch of a design that I think is a better long-term approach; basically a single unsafe creation of a token factory for MMIO address space — can defer this PR till we explore alternatives
 - Leon/Johnathan: Yes, but 5041 is real improvement for now; can replace with something better once that exists
 - Leon: Will prioritize review of 5041
 - Pat: Sounds fine, no need to block 5041
 - Alexandru: Can I join this discussion for new solution? With hard realtime I really need a mechanism to assert no aliases.
 - (Many voices): Seems like a thing that needs a task force / working group
 - Pat: We can use #tock-registers, I think MMIO address space management is something very reasonable to be in scope for tock registers
