# Tock Core WG meeting notes 2024-11-08

## Attendees

- Amit Levy
- Leon Schuermann
- Benjamin Prevor
- Brad Campbell
- Johnathan Van Why
- Branden Ghena
- Lawrence Esswood
- Hudson Ayers
- Tyler Potyondy
- Pat Pannuto
- Kat Watson

## Updates
### Treadmill
* Tyler: Worked with Leon and Ben on Treadmill. Been working on it to port 15.4 tests. It's been working well so far and I really recommend it. Sometime next week I'll have some of the 15.4 tests moved over, then Thread will be next. Then we can add that to the CI for changes to Network stuff.
* Leon: For the CI, Ben has been working on porting lots of tests. I made an overview in Markdown here: https://github.com/tock/tock-hardware-ci/actions/runs/11746206983
* Leon: We have multiple different boards which run tests. And some tests only correspond to some boards. So we have a table with what is and isn't working for each board. Some of the tests are still failing, which we're working on.
* Ben: You can also go into the output to see what's going on with a test
* Tyler: Where are these tests right now?
* Leon: On a branch here: https://github.com/tock/tock-hardware-ci/tree/refs/heads/dev/test_ci_branch/hwci/tests
### 15.4
* Amit: Tyler and I found some 15.4 configuration issues with boards like the nRF52840DK. There was some complexity with the configuration. Hoping to resolve this soon. The context is working on the nRF52833 which is on the Microbit.
* Tyler: We could add that board to the Treadmill setup.
* Leon: We do have one nRF52DK, which is an nRF52832
* Amit: Generally, we want all of the boards eventually. But I do specifically think that having a 15.4 test on multiple different boards would be particularly useful. Maybe even a mix of boards.
* Branden: Make an issue on github about the problem please!
### Type-Syscall Data
* Johnathan: Working on a document on how System Call argument passing could work in the future, like in a potential Tock 3.0
* Johnathan: Lawrence is trying to make stuff work in the Tock 2.0 syscall rules right now. But there is this explosion of different return types that can happen, and lots of arguments that are vague about being usize or integer or pointer. So I thought there was a way for things to be more flexible and robust. So those thoughts are going in a document
* Johnathan: Thanks to Lawrence and Branden for feedback
* Johnathan: Lawrence also gave me some new ideas that I may or may not include. Thinking about it


## EWSN Tutorial
* Amit: Several folks are running a tutorial at EWSN in Abu Dhabi in a few weeks. Wanted to bring it up here for what needs they have for the group, such as priority PRs or changes. https://tockos.org/events/ewsn-2024
* Brad: Most of the changes are to the tutorial discussion itself. In summary, we got feedback from Tockworld that it starts easy and has a huge jump in difficulty. So we think we can rearrange the material to make a smoother difficulty curve. If we have time, we'd also like to add some smaller modules that we won't necessarily get to, but advanced people could try them out. Particularly introductions to the kernel and editing it.
* Brad: As far as technical kernel changes, we'd like to integrate the new process buffer queue thing Alex made for the 15.4 driver. It should hopefully be a minor change since it already does something similar. We also want to upgrade the non-volatile storage driver to work on a per-app basis.


## Cortex-M4F Crate
* https://github.com/tock/tock/pull/4224
* Amit: Pat sent this PR, which moved some chips to a new microcontroller architecture: Cortex-M4F. But only organizational changes for now
* Leon: For now, it's just an alias of the Cortex-M4 crate. Which comes from Cortex-m (or v7m) crate. So this PR moved chips to this new more-appropriately named crate. It does NOT add any hard float support in kernel or userspace yet.
* Leon: To me, this felt weird because it signals that we could maybe turn on hard float in Rust, but we actually can't without further changes
* Leon: It also felt weird to move stuff before floating point support existed. And I'm curious about the timeline and plan here
* Pat: For an external project, I've been working on hard floats in userspace apps. So that's the long-term impetus.
* Pat: In the short term, I was annoyed that I keep forgetting which chips in Tock do or don't support hard float. And Cortex-M4 and Cortex-M4F are really just different architectures (just named similarly). So the M4F is the same as the M4, but an M4 is the same as an M3 anyways. (M4 has DSP extensions we don't use. M4F has float extensions we don't use) So, I think this is just as justified as having a separate M3 and M4 crate.
* Leon: Okay, that makes sense. I was confused before about the goal
* Pat: So, there will be a trickle of work for doing hard float support, and discussing whether kernel or apps or both should support. Probably just apps. The ARM architecture actually has some good optimizations for register stacking based on app float use, which we could think about how to use too
* Brad: I'm worried this is misleading. Crates are sort of for marketing, and if we don't have real M4F support, then this seems misleading to me
* Amit: I think we can enable floating point only in the kernel with basically no overhead, right? Because it would still be disabled in userspace by default and wouldn't require any changes there.
* Pat: I think it would be like one assembly instruction in init, and one assembly instruction on each side of a context switch
* Amit: I think you can make it supervisor-mode only, which would remove the context switch instructions too
* Amit: So that would, roughly-speaking, solve Brad's misleading concern
* Pat: But we have to play with that. I want to check what it would do to code size, and that everything still works.
* Amit: Some of the capsules do floating point math, I think. Although we might have gotten rid of that
* Leon: We also have target definitions in libtock-c and have both Cortex-M4 and Cortex-M3. But right now, after a context switch, the "virtual machine" we expose to applications is still an M4 (not an M4F) if floating point isn't enabled for it
* Brad: How does a board decide what to use?
* Pat: That should be in a design document I'm drafting. My thought is to have something in a Tock application header that requires hard float support. Then the process loader would enable or reject it, and do something when loading/switching to the apps based on that need. In the long term, you have to multiplex multiple apps having floating support use, which could use the optimization in ARM to decide whether to save floating point registers or not based on whether they've actually been used.
* Amit: That's just an optimization though
* Pat: Well, floating point context switches are like three times as long in ARM. So it's valuable
* Amit: I think you really have to do it most of the time though.
* Pat: But we could disable floating point and only turn it on upon exception if it's required, and we can track that
* Lawrence: I think ARM even suggests doing that
* Pat: Yes. The better way is the bit in hardware that tracks whether floating point instructions were used. So if the kernel isn't using hard floating point, you can just use that.
* Amit: So it seems like something is on the horizon that could make hard floating point real

