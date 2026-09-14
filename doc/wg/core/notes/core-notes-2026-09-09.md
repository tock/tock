# Tock Meeting Notes 2026-09-09

## Attendees
- Branden Ghena
- Brad Campbell
- Pat Pannuto
- Leon Schuermann


## Updates
* None


## Testing 
* Leon: Treadmill work has been underway. Platform has reached base functionality and stability again. Enough that we could deploy real hardware-in-the-loop Tock tests on it. Also useful for remote development, but focus is CI and testing right now.
* Leon: Separately, we've seen a lot of new testing efforts such as QEMU support, new tests like the screenshot-based tests, and then efforts on Tock registers which are supposed to allow for unit testing drivers.
* Leon: So, we might want to consolidate and coordinate efforts on the various testing paths to avoid duplicating efforts and reuse testing infrastructure wherever possible. Probably too deep of a discussion for a regular Core call, so we might split off a taskforce. A little design of architecture and direction for efforts.
* Leon: For example, Treadmill is still pretty malleable and we could direct its testing in several different ways depending on goals.
* Brad: Feels like there's an overarching feeling pushing back on small efforts. Comes across as "now that you did work this isn't what we envisioned". AI+QEMU suddenly made efforts possible, so slowing them down is frustrating. This seems like a noble idea, but it'll take a very long time and its unclear it'll be successful.
* Brad: Alternative approach is "let a thousand flowers bloom" where we just try a bunch of things and see what ends up being the most usable, reliable, etc. We'll end up naturally coalescing on those mechanisms.
* Pat: Mixed feelings. Compelling that we haven't figured out a comprehensive testing strategy yet, so we should let parallel efforts happen to see if we can work on stuff. But we also want to make sure there isn't unreliable stuff becoming "check" where we get used to them failing in practice
* Leon: Empathetic to these concerns. Emotional component here is that Treadmill has been such a long journey of working on something ambitious for years now. It's been fairly obvious that just setting a grad student on this wasn't the best way to get it over the finish line in a polished way. AI improvements have helped on speed though. I'm open to the idea of trying a bunch of things, but I do also believe that there is actual value in real hardware CI. But I can't drive this forward alone. It needs other people considering the architecture to create something meaningful.
* Brad: That makes sense and is completely reasonable. So maybe lets start by explaining where you're hitting the wall. Then also a comment is that we agree that QEMU is only software-level checks. There is definitely a clear need for real hardware testing. So maybe the result there is just keep going and work on stuff. A taskforce would just slow that work down, right?
* Leon: So maybe what we can do is talk about existing abstractions and where this leaves us for Tock efforts.
* Leon: (demos treadmill) linux images can be uploaded to treadmill. Hosts are RPis attached to Tock targets. The Treadmill API starts an image on a host. After a couple of seconds you get a Linux session you can run arbitrary tests and tools in which has access to dev boards connected over USB to the host. I'm showing this manually, but there are APIs for everything so it can be automated.
* Leon: Previously a github actions runner built Tock, loaded it on a board, and did basic I/O interactions. It was pretty hard to write these tests though. It was also hard to pin down what we are testing and if those tests are meaningful. So now is the point to decide what are the meaningful targets to test, how to isolate things, how to run tests on drivers.
* Leon: For the tests themselves, I'm no longer the expert here, and I'm looking for support.
* Brad: What do you mean it's hard to run these tests?
* Leon: We ran a userspace app on a kernel, but we never did kernel-based hardware tests. How would we build a set of scripts in CI that reliably does small board updates to test individual parts, without leading to a million upstream configuration boards.
* Leon: Also, what's the important hardware to test. Which parts should we focus on to achieve coverage?
* Brad: Okay, I see. This seems like a hard road to go down. If you ask people what they want, they want everything. I feel like what will work is an explanation of how to write a test and a clear path for people to write their own tests for their own boards. Getting consensus is going to be hard.
* Leon: It's more clear to me that what I'm asking for is direction. We've never formally sat down and reasoned about what should be prioritized for tests. Should we do integration tests with applications or stuff in kernel? Should real I/O be involved or should we emulate that? So I'm looking for concrete goals on where to go next that have buy-in from others.
* Brad: No one gets excited about testing. It's always extra. Anyone will be invested in a specific feature. I do have some opinions on what can be done, but I'm hesitant to have an over-arching plan first because that'll be hard.
* Leon: That's fair. But for comparison, last time Ben and I ventured off and built something without consulting. So I'm concerned that without active discussion about these things, I'm worried that will happen again.
* Brad: Okay, one concrete answer is that I want our tutorials tested, so we know that they will always work. Similarly you can see tests that I'm running on QEMU, just loading some apps and checking output, and I'd love those on several real boards.
* Leon: Okay, that's helpful.
* Brad: I'm also about to open a PR that captures how I see things. Adds another board just for testing.
* Brad: I also think this configuration board testing thing, and making it generic goes all the way back to Phil. It's just too hard in practice. Once you switch your mindset to just committing another config board, life becomes easier. And copying changes to all boards is increasingly easy with LLMs.
* Leon: Okay, this makes sense to me. The way I'll move forward is to try to think about how to have a complete deliverable, then come up with an architecture for both Tock changes and Github CI workflow. Then maybe we can have another dedicated discussion.
* Pat: That seems reasonable. The thing that we want should center around developer and user experience end-goals. I want CI to work well, but I don't actually care about _how_ it works. The end goal is what's important here.
* Branden: Is a goal here to get _some_ Treadmill CI up again soon?
* Leon: No, we could have that quickly. Some frustration before was outages, which could always happen. But a bigger concern is that this architecture didn't seem to have clearly defined value, as no one was willing or able to extend tests and add more of them. So that's what I want to solve, but I can't do that in a vacuum. That needs users and community buy-in.


## Tock Release
* Pat: STMU32 clocks and a few QEMU things were lingering. We could run through those in order.
* Pat: https://github.com/tock/tock/pull/5034
* Pat: STMU32 clocks PR is in a weird spot. I can't tell if this is done or not?? It does work, but also returns errors during boot instead of panicking? Do we want to say this is good enough for now, and we'll clean up errors later? Or we could remove it from the release?
* Leon: Alex was promising that this would be crossing the finish line quickly.
* Pat: Yes, and there's been a little engagement, but it's been slow still.
* Pat: There's a CI failure here, but it's transient. Some dependency issue. I think rerunning this will fix it.
* Brad: I think this is fine. It's a step in the right direction. It's a shame that Rust makes it hard to write const code and easy to write dynamic failures. So a better design could exist, but it's probably okay.
* Pat: Okay, merging. I think it'll pass without a rebase.
* Pat: https://github.com/tock/tock/pull/5125
* Pat: I think I handled comments. Also, does this PR make sense to Leon?
* Leon: Yes, but when I see this is a ton of LLM stuff, I assume the LLM did stupid things by default. So anything I didn't understand I just assumed was stupid. Your comments make sense though.
* Leon: I have only looked at part of this. I would like some more time to look through all of this.
* Pat: Fair. In this case, I really did spend a lot of time reviewing everything here before pushing it. I patched all over the place.
* Pat: In terms of release, this does have the benefit of being an isolated thing. It's a small on-the-side QEMU thing that should affect any testing efforts. Any changes on this PR would on affect this one board. Something nice about including it in the release is coverage for boards we've never covered before.
* Leon: I think ultimately I'm not opposed to getting this into a release, but it shouldn't be blocker. Why should a new chip be pushed right into a release, rather than right after it? I think it is fairly low risk, but I think there is some residual weirdness in this that will need discussion.
* Pat: I do have a backlog of cleanup tasks from doing this, such as applying the patterns I used here across other boards. I think they're better for reasons
* Branden: I'd say that this isn't necessary for a release. We were fine releasing before this existed, so why do we definitely need it for a release now?
* Pat: I can see that. I think it's great to have more testing, but I won't fight so strongly about it
* Leon: If this was from someone external, I'd be more hesitant about it. We could still use this for release testing, but then also merge it after the release
* Pat: Decisions: STMU32 clocks is merged, QEMU ARM will be pushed to after release
* Pat: Last call on any other PRs?
* (none)
* Pat: Remaining questions:
  1. Do we tag a release candidate or make a branch?
  2. What tests do we want to run before approving the release?
  3. Who runs the tests?
  4. What is the deadline for testing?
* Brad: Let's open a branch for release. It's hard to buckle-down and not merge anything before release?
* Leon: Branch also lets us cherry-pick if we really really want to.
* Pat: Okay, let's make a branch
* Pat: What tests should we run?
* Brad: nRF tests, preferably the more interactive ones.
* Leon: I can take that on. That's existing scripts covering 80-90% of our existing checks.
* Leon: What other boards?
* Pat: Perhaps we should redefine our support tiers. Full battery of tests for everything in Tier 1. Although Tier 1 still includes Hail and Imix, which are increasingly unused.
* Pat: Okay, I can make a PR right now that can change the tiers.
* Pat: For the boards in Tier 1, we should run lots of tests. How that is accomplished, I don't really care.
* Leon: Extensive coverage for all distinct chips on Tier 1 boards at the least.
* Leon: It's sad that only nRF52 are in Tier 1. Are any of the others close?
* Pat: Some of the STM boards, have to be close, right? And maybe the RP2040.
* Branden: So some nRF52 boards. Probably RP2040 and an STM32 if possible. Remaining thing is RISC-V?
* Leon: We have virtual tests for QEMU and LiteX that do RISC-V. So it's not untested. Is the ESP32-C3 still viable?
* Brad: Yeah, it should be.
* Brad: I push back on defining Tier 1 based on wishes rather than what we actually use.
* Leon: I think it's useful to have a reminder of what's important and what we _should_ be testing with. Outside contributors won't propose a tier change.
* Leon: So summary: definitely nRF52840DK and other Tier 1 nRF boards. Might kick Hail/Imix out of Tier 1. Might try to test RP2040 and STM32. And ESP32.
* Brad: I have an ESP32-C3
* Branden: I can test on RP2040
* Leon: I have nRF52840-DK
* Pat: I'll do a Redboard for Apollo3
* Leon: I'll update the release issue to point to some tests to run. Should we also do tutorials?
* Brad: We should do our best.

