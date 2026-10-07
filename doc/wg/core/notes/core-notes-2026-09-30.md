# Tock Meeting Notes 2026-09-30

## Attendees
* Branden Ghena
* Amit Levy
* Leon Schuermann
* Brad Campbell
* Johnathan Van Why
* Pat Pannuto


## Updates
### Code Review
* Johnathan: Looking into improvements for code review. On a better diff algorithm, there are multiple choices and we could reasonably have a dropdown with multiple diff options with different tradeoffs.
* Johnathan: I'm also looking into the ability to diff when a PR rebases and ALSO makes changes, so you can just see the new changes.
* Leon: Exciting, but really doesn't feel like it should be our problem to have to solve. Shouldn't Github be doing this?
* Johnathan: Yeah, but we need more competency than what Github actually provides
* Pat: Github is insufficient for some needs, for sure
### Release
* https://github.com/tock/tock/issues/4584
* Leon: Found a bunch of interesting/scary bugs over the last few weeks that should be fixed before a release. I've been automating a bunch of testing along the way
* Leon: We do have some PRs that will need to be back-ported into the release. Once we have those merged, we can update the release branch to the current master and restart release testing from there. Ultimately our release will be in a much better state with all of this.


## Unsafe Documentation
* https://github.com/tock/tock/pull/5208
* Amit: Do we approve of turning this on? (yes)
* Branden: A lot of work by Brad to document unsafe everywhere and this is a big improvement. Good work.


## QEMU CI Testing
* https://github.com/tock/tock/pull/5155
* https://github.com/tock/tock/pull/5101
* https://github.com/tock/tock/pull/5162
* https://github.com/tock/tock/pull/5213
* Brad: When we added the RISC-V 64-bit support, I wanted a way to be able to run that in CI. I had Claude create a runner for it. It has some features that are useful: you can give it a list of tests to run, it can run multiple apps, in can expect text output in various orderings, it can take a screenshot...
* Brad: So these PRs expand upon that. Add more boards, make test lists easier to work with. Generally makes more things run in CI.
* Leon: Ultimately this is a good change from the high-level description. I haven't had time to look at this in-depth. As I'm working on the hardware CI test harness right now, which is just a prototype right now, I would ultimately want that single tool to effectively incorporate and replace this QEMU runner as it does a superset of what this does.
* Leon: I've relied on what we're doing in CI now, which Brad wrote much of. So I do plan on having it look quite similar.
* Leon: I don't want to nitpick this now, as I can make those changes in my own proposal. In the meantime, more tests are valuable, so I have no problem with this
* Brad: It's helpful to be able to test the screen stack. It's also helpful to just have more of these tests created. It makes sense that if this runner is duplicating what's done in the future, we can totally scrap it. I have no attachment to it. But the value is the kernel definitions, tests, and expected outputs created. That's useful here
* Leon: Initially, I was worried about duplicating effort that will be hard to reconcile since we've been working independently with different goals. But I realized that the test definitions differ, but in mechanical ways that LLMs can trivially reconcile. So I'm no longer afraid of getting stuck with differing systems. Especially if we're willing to make big changes in the future.
* Leon: I am still working on a document about user interactions and workflows for hardware CI. What happens when a PR comes in. What happens when someone merges. Etc. That's tangential to what's going on here, as this focuses on specific tests to run. No conflicts there.
* Brad: https://github.com/tock/tock/pull/5155 is the "first" one of these PRs. The interact weirdly, but they all just add tests
* Johnathan: I am fatigued on code review and keeping up with larger PRs is not really possible. So we might want to explicitly decide that test boards are not going to be deeply reviewed since they're separate. Enough safety nets in place to stop something malicious being merged, but without too much human review.
* Leon: I do empathize with this. There are so many weird quirks and edge cases in hardware that really can't be just spotted without running stuff. I think that ultimately requires fast iteration and changes. We do want to avoid architectural issues that can burn you if no one is paying attention. But it is a pressing issue that the flood of PRs is hard to deal with.
* Amit: Feels like a separate discussion than these particular PRs.
* Johnathan: For end-to-end tests and designing what's written, it's probably worth separating high-quantity, low-quality code from important Tock code which is high-quality, low-quantity.
* Johnathan: So far tests have had configuration boards to support the tests. An idea, which might not make sense, is to separate which parts of the codebase we care strongly about and which we have laxer standards on. This is really a separate discussion though.
* Amit: Ok. So on Brad's PRs, they need to be reviewed. But we probably should move forward with these and the plan is to review and merge the PRs.
* Amit: Separately, there's a discussion out of this about thinking about a more permanent testing solution and how we review tests.


## Rename UART to UARTE
* https://github.com/tock/tock/pull/5092
* Pat: Only look at the last commit on this, and turn whitespace off. The diff looks scary, but it's not
* Brad: So we fix stuff, and rename stuff. Look commit-by-commit
* Amit: Okay, this is good to merge


## Safety Documentation Instructions
* https://github.com/tock/tock/pull/4954
* https://github.com/tock/tock/pull/5129
* Pat: I wrote a first draft that's too pedantic. Brad did a good draft, so we just focus on Brad's version
* (people read PR and approve it)
* Leon: This doesn't cover unsafe traits, which is good because they're complicated. This isn't accidental, we should think strongly about those.


## Multiple of the Same Syscall Drivers
* https://github.com/tock/tock/pull/5176
* Brad: We've always considered having multiple copies of a syscall driver, which are the same but with different Syscall numbers. Let you have multiple temperature sensors or multiple consoles. We've always said you can do it customly
* Brad: So, I figured I'd make the convention and tools for doing this more clear.
* Brad: We group driver numbers by type, and have the MSb mean "external, out-of-tree". So I added semantic meaning to other bits (the second-highest nibble here) where those four bits are the "driver increment number". All 0 for the first driver, 0b0001 for the second of the same driver. And so on.
* Brad: So this PR adds a shared function that will do that automatically for you. Lets us point to something more conventional. Also lets you write tests for duplicate drivers and use libtock-c to interact with them in a sustainable way
* Brad: Specific use case is a second console. One for printf and one for sending TBFs to a board. Both in userspace.
* Amit: One thing good about this is that. It's limited to 16 instances, but we could eat part of the nibble to its left if we really really needed to.
* Branden: What's the libtock-c plan here?
* Brad: I made a console2 PR: https://github.com/tock/libtock-c/pull/588
* Brad: We could also have other tooling.
* Branden: Ugh. Stupid C. The problem here is that each C driver has a hard-coded driver number and doesn't pass it in/encapsulate it in some way.
* Leon: I an concerned about bit packing magic with driver numbers. Every time we've extended the meaning of driver numbers it felt super hacky. It reminds me of IP address fragmentation problems. I'm not going to stand in the way of this, but if we're changing userspace APIs, then we might want to support passing a full driver numbers into libtock-c drivers. Not an instance ID, which feels like a bit of a quick hack.
* Amit: For libtock-c, I agree that if we allow passing in something like an instance ID to an otherwise general-purpose driver, then it ought to be the whole driver number. Which can retain the bit packing meaning, but also enshrines that the driver number at all are made up and tunable by board. Doing that at all is not clean in C unfortunately, which is the main problem for libtock-c. Libtock-rs is a different story.
* Leon: I proposed a driver registry component in C at one point. But people didn't like the dynamism it required, which makes sense for the embedded space. I personally don't love this, but I'm okay with it.
* Brad: To be clear, I have no plan to update libtock-c at this time.
* Branden: I think this is reasonable. For something like console, having a console2 driver might actually make sense. For most drivers it won't, so we won't have them upstream. If someone wants to use this, we'll tell them to add their own out-of-tree drivers anyways.
* Leon: We have one example right now that uses this, and we already intend for downstream boards to have space for their own driver numbers. So it really doesn't feel right to me to define a new standard way of doing this but not actually exercising this in any significant capacity, and not adding proper support to userspace libraries for it. So if this test is already bound to a particular board, why don't we just use the non-standard driver number space for this?
* Brad: I don't know that two consoles will only be on one board.
* Brad: I think this can be the convention, not a standard. People do ask us how to do this, and having some docs to point them to is great. We're not saying you MUST do this, and there isn't a ton of demand. But if you wanted to, here's a way that is okay and would work.
* Pat: Right now we have no guidance, so having some guidance would be great.
* Pat: We could require the MSb to be a 1 for these, since it's non-standard
* Leon: I like that. We could require the MSb to be 1, and still have a suggestion for enumeration from there.
* Brad: But this would be used upstream, with an upstream libtock-c driver. It's not actually out-of-tree.
* Leon: I think it's totally fine for upstream drivers to user a non-standard driver number.
* Brad: It is a standard, upstream driver
* Brad: The MSb is defined in the book.
* Leon: "A driver who's highest bit is set to "private" can be used for out-of-tree drivers". Oh, I thought this had a different meaning. This definition in the book doesn't make sense to me.
* Branden: I think we shouldn't use the MSb for upstream stuff. Using four bits for the rare use case sounds good to me.


## Consolidate Unsafe in Slices from Linker Symbols
* https://github.com/tock/tock/pull/4967
* Brad: In our board main.rs files, we make slices of flash memory where apps are stored and pass those into app loading. That includes having to create some unsafe block to access the linker symbols. And doing the slice command to get a Rust slice from the linker symbols. The safety requirements for all of this is so strange and messy
* Brad: This macro is an attempt to consolidate this to one, well-documented place that all boards can use and not have to do it themselves. That's the goal here.
* Brad: We've iterated a few times to an implementation that I think is reasonable. It's got documentation and checks, and still requires some guarantees from the user that these symbols are valid.
* Johnathan: Does the slice have to be at least 1 byte, or is a zero-length slice acceptable?
* Brad: I don't really care. I forget where we ended up.
* Johnathan: Length of zero is undefined behavior.
* Pat: We should be defensive and allow those
* Johnathan: Change sym-start to be an empty array of u8s. Or some other zero-sized type.
* Leon: Isn't a raw slice pointer a much more appropriate type to return from this macro, as it has fewer guarantees the caller needs to keep?
* Johnathan: Hmmm. Sort-of. We're returning a slice of u8s right now. So the bits have to be initialized. I'm not sure which is more useful. This is intended to encapsulate unsafe, and a raw pointer wouldn't.
* Leon: The reason I'm bringing this up is that we switched major parts of the process loading system to use raw pointers. Explicitly for the RAM parts which are unsound to retain references to. Flash memory is also subject to change... So I'm concerned that this is unsound since they live for the entire lifetime of the board. I'm worried that creating a nice unified macro will still lead to later changes.
* Brad: That's a great point. I'm concerned about that too. I believe that this macro should handle what you describe and we should head towards that. I just don't know how to do that.
* Leon: I could give a half an hour to try to tweak this
* Brad: That would help immensely
* Pat: Will this pick up provenance details? If we're using raw pointers?
* Leon: All the memory we're referencing here must come from outside of the Rust abstract machine. So they're created from exposed provenance,...
* Johnathan: Nope. None of that is right. These have to be within the Rust abstract machine, otherwise every interaction needs to be volatile.
* Leon: I see. I meant that this memory wasn't originally allocated by Rust.
* Johnathan: But it is still a Rust allocation.
* Leon: Okay, you and I need to discuss this offline. We should definitely handle those cases in this though.


