# Tock Core Notes 09/11/2020

## Attending
 * Branden Ghena
 * Amit Levy
 * Alistair
 * Leon Schuermann
 * Johnathan Van Why
 * Samuel Jero
 * Phil Levis
 * Hudson Ayers
 * Brad Campbell
 * Pat Pannuto
 * Johnathan Van Why (a second time!)
 * Vadim Sukhomlinov
 
## Updates
### Tock 2.0
 * Amit: Spent a lot of time revisiting last week's system call interface discussion. Also talked about allow/unallow and subscribe/unsubscribe issues. Most important point is that we have idea of important goals to achieve and a prototype implementation. Hoping to be able to share and present that in the next few weeks.
 * Amit: We will be implementing the new system call interface in parallel with the existing one for a while before we switch over fully. That way we can test some things without changing everything. Leon has an implementation of two system call interfaces working in parallel.
  * Phil: There's a bunch of work, but it's pretty mechanical. I think the most of it is deciding the details that we're doing now.
 * Amit: Timeline is getting delayed unsurprisingly.
### CTAP
 * Alistair: Mostly working on Tock. It's the ubikey authentication thing. FIDO spec. U2F used CTAP version 1. We've got CTAP version 2.

## Alarm Redesign
 * Amit: Ready to go, modulo some help testing on some devices.
 * Phil: As a reminder, there were a bunch of bugs with the prior interface, especially with race conditions for alarms that are soon in the future. For most hardware you just wouldn't get an interrupt (for 2^32 ticks) so the system would hang. Instead now, we are providing two values: the current time at the call and the delta-t for when in the future it should trigger. This ended up changing a BUNCH of things throughout the kernel because everything uses the timers. But now it passes all the checks and is implemented for all the chips.
 * Phil: Also, system call needs to deal with 64-bit and 32-bit values for OpenTitan. I am having a very hard time testing userspace for OpenTitan though. So this remains to be tested.
 * Phil: Tested nRF, OpenTitan, SAM4L, but have not tested the others. This is what we are blocking on.
 * Phil: We still need to test: ArtyE1, HiFive, MSP, RedBoard (Apollo3), STM32F3, and STM32F4. Multi-alarm test should be sufficient for these boards. If anyone has one of these, it would be great to add the test and run it.
 * Brad: I would propose that we put a time limit on this if we're at final testing. Some of this testing can just be a part of the Tock 1.6 release.
 * Phil: My concern is that if you're somebody using one of those boards, master might be broken for you.
 * Amit: Worried that some of the boards are active and supported, but might not be testable on a short timeline.
 * Brad: I'm worried that having the PR open and letting master diverge has a chance of introducing bugs too.
 * Phil: I would feel really bad about committing totally untested code though. That would upset me on the receiving end.
 * Amit: So, ArtyE1 is Brad, HiFive is Amit, MSP is unclear (Hudson will contact through github), RedBoard is Alistair, STM32F3 and F4 are the European academic crew (will contact through github). Phil will put testing instructions in the PR. https://github.com/tock/tock/pull/2089

## Freeing Grant Region Memory
 * Hudson: Realized there were lots of places where Drop for AppPointer and Owned types are called, but the implementation of Drop calls Free which is unimplemented. Despite it being empty, because it's hidden behind a proccessType trait object, each call to Drop takes 150 cycles for these types. For something like the Imix app, a single cycle of the loop calls Drop many times. So there's a bunch of overhead for no purpose.
 * Hudson: Previously, I thought we should implement the Free method. But we're not using any dynamic grant memory allocation stuff. The new dynamic grant allocation PR doesn't have a Drop method. So they're not called and not being replaced. We should either agree we aren't going to worry about freeing it, which is the status quo, and remove the implementations. Or we should decide that this needs to be fixed, and it should be fixed everywhere, including that new PR (https://github.com/tock/tock/pull/2052)
 * Amit: No one uses the dynamic allocator at all, except for a branch from SOSP testing. It's never been useful until David the Western Digital person needed it for the BLE driver. If we have this dynamic allocation, then I think we have to have free. If we're not merging the PR, then there's no point for free because all the other grants are only freed on reboot.
 * Amit: I agree that all the calls to free Hudson is seeing are bugs, because memory should not be free there.
 * Hudson: So we should probably block PR on a method to free?
 * Amit: Unclear. The current allocator is indeed unsound, so fixing it is still valuable in his proposal. Without free the process will eventually run out, but it isn't unsound.
 * Hudson: Currently, if a capsule tries to allocate a grant but doesn't have memory, the process should be killed?
 * Amit: Yes. Right now, I think the allocation fails, and the capsule can do whatever it wants with that response.
 * Brad: So what's calling Free so often on memory we don't intend to free right now?
 * Hudson: AppSlices going out of scope calls Drop which calls Free. But an AppSlice is just something given to it by the process. So it shouldn't be free'd here. So I think that's a bug. For the Owned type, I don't understand how that's supposed to work, but going out of scope calls Drop which calls Free. I think that's also a bug?
 * Brad: So what does this boil down to in what we need to change in code?
 * Amit: I think it's just Hudson's PR: https://github.com/tock/tock/pull/2101
 * Brad: So we'll leave Free in the processType, just not call it from Drop.
 
 ## Rubble Dependency
 * Hudson: We did already talk about this, agreeing to put it in Boards so it only affects the Board we care about. But something we didn't consider, is that since the dependency is in a top-level crate, it's downloaded for any use of Tock. So even out-of-tree boards end up downloading the dependency with cargo (and just not using it). So the real question is what the policy is for optional-dependencies. We may have to always put them in a separate cargo workspace.
 * Johnathan: I ran into a similar issue with libtock-rs. If Rubble is big, we don't always want to download it. But I found that it's really easy to accidentally pull in dependencies in libtock-rs. I did play around with this some, I think maybe the answer is that there needs to not be a Cargo.lock file in the workspace.
 * Amit: So maybe if you don't have a Cargo.lock file and you run cargo build with only a subset of the dependencies, then the lock file would only contain what's needed for you?
 * Hudson: If I build Imix, it downloads Rubble even though I'm not using the board with that dependency. But I think that out-of-tree boards wouldn't download it, which is probably most of the problem.
 * Johnathan: I can test this against a PR if you can point me at one. Cargo behaves strangely.
 * Hudson: Okay, so the decision for the call is: should external dependencies in Tock, not require downloading the dependency for boards that don't require it?
 * Amit: To me, it seems bad to have to download dependencies for everything in a workspace, even if you only need a subset of it. I suspect there is a way to do this, because there are large Rust repos, where it seems unlikely that depending on the core library requires every single one of its possible dependencies.
 * Hudson: Maybe I ran into this because in an out-of-tree thing, I still used in-tree components. So maybe it would have been fine if I had done it the normal way.
 * Amit: It definitely makes sense to avoid out-of-tree boards downloading things they don't need. For stuff inside the tree, I don't care as much, we download a bunch of things that might be helpful anyways.

## Tock 1.6 Release
 * Brad: With the timer PR getting really close, that suggests we should do 1.6 release soon. It's been a while since we thought about it though, so if there are new things that ought to be included, please reply to the issue. Especially since this is likely to be the last release before Tock 2.0.
