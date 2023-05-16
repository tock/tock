# Tock Core Notes 2023-02-03

Attendees:
 - Branden Ghena
 - Philip Levis
 - Hudson Ayers
 - Jett Rink
 - Alyssa Haroldsen
 - Vadim Sukhomlinov
 - Brad Campbell
 - Johnathan Van Why
 - Alexandru Radovici

## Updates
 * Phil: I just got off a 45 minute conversation with Lawrence Esswood, the person who has been looking into new kinds of allow based on some interesting memory requirements and he has come up with some very cool and promising ideas. Number one is the whole question of "can I actually allow DMA on memory from a process, with the necessitating implementation of this being that the zombie can become a zombie process"?. In the process of working on this he banged his head against grants a lot and came up with an alternative to the Grant type which I think is very interesting, and he is interested in presenting that to the core group next week.
 * Phil: What he sees is us moving away from Grant::enter() and to his new idea which he calls "PRef". His new implementation is significantly simpler because we don't have to worry about grant reentry and stuff like that, and also brings some significant code size reductions, so I said people would be excited to hear about that.
 * Hudson: Do you have a 10 second summary of what is different about PRef?
 * Phil: The idea is that having a Pref is something like having a grant, in the sense that it is something which a process has given me which might no longer be live. Then you can convert a Pref to a Pref-live and a Pref-live is constrained in lifetime to a particular scope. It's basically then the case that as long as a kernel path is executing, I can continue to use Pref-live but then when this kernel path completes it is no longer usable and would need to be regenerated from a Pref. This has a nice property that you are no longer constrained by scope in the same way, so now Grants can reference each other (via non-live references).
 * Hudson: And then a chip peripheral itself could actually hold onto a Pref (unlike a grant reference)
 * Phil: Exactly
 * Hudson: My update from the past two weeks is that I finally submitted the new deferred call PR that Leon and I have been working on. I am pretty excited about it and I think it is a much better interface, reduces LOC in the kernel and saves some size, it is also really nice that we don't have two deferred call types which I think is something that people always kinda tripped on. If you go look at the PR it is going to seem like a massive thing to review because it is like 2500 lines changed, but if you are going to review it I recommend just looking at the kernel changes, and then looking at a couple chip peripherals and a couple capsules. Leon and I have both gone through the two of us and looked at the full extent of the changes, so I don't think that we necessarily need a third set of eyes to do that. I am definitely interested in any feedback on the high-level new design and interface and stuff. Alyssa, I did tag you on the PR but a soundness review would definitely be appreciated for this. There are a couple of places where we are using `static mut` Cells.
 * Alyssa: Oh, that's...fun
 * Hudson: Yeah, so basically this is something we were doing in the old DynamicDeferredCall implementation that I carried over to the new one but I suspect we might not have done it that way on the old implementation if a pair of more discerning eyes had been on it you know 4 years ago or whenever it was. We could get around this by using like atomics with some code size cost, but the idea behind what we have now is that we have these static mut Cells, but they are not visible outside of the file, and every time they are accessed they drop their mutability immediately.
 * Alyssa: So I have a SyncCell internally that we might want to port here
 * Hudson: I assume that would do basically the same thing but enforce it at the boundaries of the Cell rather than having to check every place that the static mut gets accessed in the file?
 * Alyssa: It essentially has two implementations, a thread unsafe and thread safe version, and which is used gets changed based on what system you are on.
 * Hudson: I think we would be interested in having something like that because there are a lot of places where we are kind of using static mut incorrectly and that might be preferable.
 * Alyssa: OK yeah, I think I agree there. The question is how do you determine whether you are in an environment without threading, can we just have like a flag? If you are running unit tests or host emulation that could be in a multi-threaded environment so you should keep that in mind.
 * Hudson: Yeah, for upstream Tock we don't have host emulation and have been operating under the assumption that the kernel will always be run single threaded. Even QEMU or anything still runs the kernel single threaded.
 * Alyssa: Yeah it is a bit of a weird thing with host emulation because any code could spawn a thread using the standard library.
 * Johnathan: I think what we need to safely support host emulation is very similar to what we need to safely support unit tests, and unit tests are very valuable.
 * Alyssa: Yeah we want there to be no way that a bunch of tests run at once cause UB.
 * Hudson: Could we just switch based on the architecture?
 * Alyssa: Yeah, right now I just assume that if you are riscv32 you are single threaded
 * Hudson: And we could do the same for thumbv7-whatever
 * Alyssa: Yeah, how should I upload that? Regular PR? How should we mix it with this DeferredCall PR stuff?
 * Hudson: I think your thing could be a separate PR that we could integrate with DeferredCall afterwards.
 * Alyssa: I would request a TODO on the deferred call PR
 * Hudson: Sure

## Significant PRs
 * DeferredCall PR ()
 * A PR adding a ProcessConsole Command History (), which lets you toggle through N most recent commands entered into the ProcessConsole, the default for N right now is 10, and this does make for a better interface.
 * Jett: This is something that Alex and I talked about a long time ago, since we have a downstream version of this, our version has state machines that help with processing escape sequences and that kind of stuff. Would it be helpful for me to add that to this PR, or do you think it is too late for your students? I feel like the way we do it is nice, it is pretty unit-testable and I don't mind sharing it for use upstream. Or is it too 11th hour?
 * Alex: I think it would be helpful, we just need it in within 2 weeks for our RustNation tutorial. My honest suggestion is to merge what we have now and then have another PR to improve it.
 * Jett: Sure, I will comment on your PR but consider it non-blocking.
 * Alex: That sounds great.
 * Brad: Low-level question: why do we need curly braces in the board main.rs files?
 * (scattered guesses...)
 * Alyssa: You cannot use a full path to a constant, it can be a constant in the same module and then it is not a problem.
 * Hudson: Merged this week, we really only had the RP2040 PWM support that I consider significant. Johnathan also had a couple small PRs for the license header checker.
 * Johnathan: Yeah, those were pretty uncontroversial after the call from last week.


## RustNation Preparation
 * Hudson: Alex, were there any other PRs that you need to get in before RustNation?
 * Alex: Nope, it was just the ProcessConsole PR that we just discussed and the RP2040 PWM PR that you just mentioned, bors did something fishy with the second one.
 * Hudson: Yeah, that was really weird -- I ran bors r+, bors batched that PR with another, the build passed and the commits from both PRs got merged into master. But the PWM PR remained opened, looking as though it had not been merged. I went ahead and just closed it manually, but the commits from the PR are actually in master.
 * Alex: Yeah, apparently there is an issue with bors that they cannot simply mark a PR as merged, just as closed, and somehow Github figures out that it was merged. If you squash commits, sometimes PRs do not get marked as merged (I have seen this in another repository).
 * Phil: I want to say the ProcessConsole PR looks like a great addition.
