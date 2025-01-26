# Tock Meeting Notes 2024-12-20

## Attendees

- Branden Ghena
- Leon Schuermann
- Johnathan Van Why
- Brad Campbell
- Kat Fox


## Updates
 * None


## Release Plan
 - Leon: Tagged 2.2 release candidate. Previous 2.1.1 was Jan 2023, so we're almost at two years. I want to release before the end of this year
 - Leon: From prior 2.2 issue discussion, we changed our release testing strategy to just test a few core boards. Those are Imix and nRF52840DK. We have nRF52840DK in Treadmill CI. Ben's also porting some additional tests to the CI so that all of the one-board tests can be automated. Then there are a couple of nRF-specific tests, which usually require two boards to work. Those we're going to run manually for now, but in CI eventually
 - Leon: Open questions are how long to give for testing and when we want to move forward. We also might discuss which things to test on Imix too.
 - Brad: Are all the Treadmill tests passing on the release candidate?
 - Leon: Yes. But we merged the tests after the commit that tagged rc-1. So they're on the tock-hardware-ci repo. You can see a workflow that ran and passed here: https://github.com/tock/tock-hardware-ci/actions/runs/12399190636
 - Branden: There's a tension here. Testing all obscure boards delays a lot and is a big ask. But not testing them, means that some might not have been run in two years now, since the last release. Probably the compromise is handful that gets us to like 90%+ certainty, especially on the major cases
 - Leon: I'm worried that we're not testing any RISC-V board. So we really ought to do one of them, probably the LiteX arty.
 - Branden: Just doing a couple will make me a lot more comfortable
 - Leon: Getting people to test is hard. Realistically the best path forward is whatever the Core group can do tests on. I'll be doing Imix and LiteX
 - Brad: I think that's right. Even Imix isn't _that_ pressing. Not sure how many people have them
 - Leon: We might revisit what our list of Tier-1 boards is
 - Brad: What about Timing?
 - Leon: I have a plan to do a release by the 25th. I am happy to move if anyone actually wants. But if no one cares, then I want to move forward by then.
 - Branden: There are release notes apart from testing
 - Leon: I will do those unless someone else wants to. We'll probably highlight 5-6 major changes, like security fixes for arch crates. Many changes are just refactors or additions of new targets/capsules though. And refactors/additions are less important for downstream users.
 - Branden: One thing that comes to mind is the capsules refactor
 - Leon: Probably no one downstream actually uses the tagged release anyways, since it's been so long
 - Leon: I'll make a draft of release notes and share around
 - Brad: I just ran some tests on the Microbit while we've been talking and everything is good there. The sensors app, plus some testing during the tutorial. Process restarting and process console and whatnot. Made some PRs that already got merged
 - Leon: We should mark that on the release issue. We don't have to check all tests, just reasonable confidence that things aren't broken.
 - Brad: Debugging the Hail board, which isn't compiling for me right now. Some extern static thing
 - Leon: Okay, I will definitely try compiling all boards too
 - Johnathan: That's compiled on stable, right?
 - Brad: Maybe that pending change about extern statics has come to pass in stable
 - Leon: A warning shouldn't change into an error if you're still on the same rust edition, I think
 - Johnathan: We deny warnings in our CI, but within a single edition a warning shouldn't change into an error
 - Leon: For local builds also, we don't deny warnings
 - Leon: By the way, Hail does compile for me on my end
 - Brad: It's process console that's actually failing, with an extern static that requires an unsafe block
 - Leon: Oh, your Rust is too old. Extern statics no longer require an unsafe block. We should have a minimum Rust version for stable
 - Leon: Okay, so this will be the first release with a minimum stable Rust version. And it should be someone's job to figure out which version that is.
 - Brad: We could just set it to today?
 - Johnathan: Latest stable should be fine
 - Leon: We should have PR that documents that, updates the CI to the MSRV (and probably also tests on latest stable too)
 - Brad: I will submit a PR soon
 - Leon: For the release we just need the MSRV. The CI tests could come later. Once it goes in the README, we should test with CI though
 - Brad: So, how is this supposed to work? If I change the required version in cargo.toml, that's what I'm supposed to do, right? The field "rust-version". How are you supposed to get that though?
 - Leon: I think it just errors. Then you do a `rustup update`
 - Branden: The rest of our tooling makes sure that you automatically update to the correct nightly. Why aren't we doing that for stable too?
 - Johnathan: That will only affect our stuff. Many other users have their own build systems, so setting cargo.toml is all we can do
 - Leon: I'm currently trying to trigger this issue and see the error. The error with rust-toolchain.toml is that we want it to pin our nightly version, since we use that for development
 - Brad: We have a separate rust-toolchain.toml for Hail
 - Leon: Okay, and I can see that we get an error if we compile it with the wrong version
 ```
 error: rustc 1.83.0 is not supported by the following packages:
  hail@0.1.0 requires rustc 1.84.0
  hail@0.1.0 requires rustc 1.84.0
  hail@0.1.0 requires rustc 1.84.0
 ```
 - Leon: Can we pin a minimum version? That would be better than a particular
 - Johnathan: You have to pick a particular one, or cargo.toml can reject old ones
 - Leon: It's silly to require users to possibly install an older version
 - Johnathan: I don't think that's an option
 - Leon: The error isn't terrible as-is. The failure mode here is that new people should always work, they'll be ahead of the MSRV. For anyone with an existing toolchain install, presumably they would have enough exposure to know they need to do a `rustup update`
 - Brad: Okay, so I'm hearing: 1) there's no obvious easy solution 2) and the error there is kind of bad as it seems to require an exact version from the error message, not a minimum? Debugging this
 - Leon: In the docs, the summary says a minimum supported Rust version. But the expanded description says it's the version you support
 - Johnathan: It is minimum, cargo will accept a newer version
 - Brad: Okay, I'll debug and see what we can do
 - Brad: I will also run a few Hail tests to be comfortable with everything working
 - Leon: Okay, we'll move forward then. We'll have nRF52840DK, Microbit, Hail, Imix, and LiteX tests, and that should be sufficient. And I'll share some draft release notes around.
 - Brad: I agree, that's great. And anyone is welcome to test additional boards before the release


## PR Check-in
 * https://github.com/tock/tock/pull/4250 still needs some work based on recent comments
 * https://github.com/tock/tock/pull/4218 seems to be waiting on Alex
 * https://github.com/tock/tock/pull/4255 is in discussion.
    * Leon: There are confusing PANIC comments in code, but I sort of gave up fighting on this
    * Brad: I'm not sure that this is the most important nonzero thing. A zero baud rate isn't particularly harmful
    * Leon: I think we might want to solve all of them, and this is just the first. It can remove panics in drivers and move them to board definitions. So all division operations are now non-panic since Rust knows there can't be a zero
    * Brad: Is that true in all cases, or only if you use the divide symbol
    * Leon: It panics on both debug and release if you divide by zero. There's a checked function that returns an option instead
    * Brad: So why not just use the checked version
    * Leon: But semantically a zero baud rate is just incorrect. So we can remove an error condition by having nonzero
    * Brad: It's a register value. There's nothing _wrong_ with that
    * Leon: Well the driver is supposed to translate the baud rate into a register value.
    * Brad: My argument is that there are many invalid UART baudrates. A baud rate of 1 isn't valid either
    * Leon: That is a nonzero frequency. It could communicate. But a zero baud rate means no communication. And it also gets rid of a panic
    * Brad: Those are two separate things. Getting rid of panics is good, but that's on the division operator. Trying to enforce no zero baudrates seems ungood. It's not protecting anything, there are many invalid baudrates. We can't we fix the panic without this change?
    * Branden: This is also just churn. We're making little edits all over for something without much value
    * Leon: To move this forward, what I'm most wary about here is that I don't want the result of this discussion to be NonZeroU32 is bad. It's a powerful tool that should be encouraged for new code for various use cases. It avoids panics and allows Rust's niche-filling optimization to remove a word of memory. I think I'm fine with the churn argument, but if it was a new HIL where zero doesn't make sense, even if other numbers don't make sense, we should use nonzero
    * Brad: I do really think the issue is operations that can panic. We shouldn't use problematic operations like divide or array indexing without the care they deserve. These harder-to-use types are addressing these, but there are other ways to check
    * Leon: The ergonomics of this type aren't great. But division with a nonzero is no longer an unsafe operation. It makes it a safe operation that can't error. That seems preferable to me to a runtime error, if it comes without expressiveness. That seems like a win to me
    * Branden: The more Leon says about it, the more I'm on his side
    * Brad: Yeah. I will say that you can count the number of panics, but you can't count "difficulty to use codebase because of weird types". We're using nonzero right now for places where zero is totally undefined, not about defensiveness. So muddling those two adds a bit of extra understanding complexity to me. And that's hard to measure
    * Leon: We do sometimes use this for cases where we're just optimizing for storage size with Rust. For example, MPU configs save 4 bytes each because of it. That's not a safety argument.
    * Leon: We also have some operations that never make sense for a zero value, like dividing by it. And that to me is actually incorrect to use a zero for. And checked division turns this into a runtime error. I do think the UART is a bad example of this, as there are other runtime errors apart from baudrate. But for any system without a runtime error already, nonzero is a good trade to not gain one.
    * Brad: That makes sense. This feels like a bad example use case to me, where other use cases probably are really good uses for nonzero. The better design for UART would probably be an associated type with limitations on real existing baud rates, but that's a very invasive change.
    * Leon: Okay, so the reconcile opinions. This isn't great for UART because 1) it generates churn and 2) there are other invalid baud rates. So it's not so attractive for handling zero specially, as the other errors still exist. For other subsystems without other errors where just zero has an error, we should encourage nonzero. And for cases where we save memory.
    * Brad: I agree with that. The other thing with the UART is that zero baud rates aren't a real problem people are running into. No one is setting a zero baud rate
    * Leon: I think what they're trying to solve is actually the panic. I think it's not passing in zero. So I think the panic is the focus and there could be another way, like checked division, to approach that.
    * Brad: This particular function returns a Result anyways
    * Brad: Thanks, this was a helpful discussion for me to understand. I'll follow up on the PR

## Next Meeting
 * Due to holidays, we will not have a meeting next week (December 27)
 * Our next meeting will be January 3rd

