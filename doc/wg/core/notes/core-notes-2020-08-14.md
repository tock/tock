# Tock Core Notes 08/14/2020

## Attending
 - Samuel Jero
 - Alistair Francis
 - Philip Levis
 - Brad Campbell
 - Branden Ghena
 - Hudson Ayers
 - Leon Schuermann
 - Amit Levy
 - Garret Kelly

## Updates
 - Leon: VirtIO updates
   - network interface seems to be working pretty well
   - been routing several calls over it through the week; gigs of traffic at this point
   - Q: How to upstream? Pretty complete package (new machine board; packet drivers; etc), piecemeal PRs or one big shot?
   - Purpose: VirtIO circumvents peripheral emulation, which is often incomplete or buggy anyway
   - Currently provides queue and event interface. Also have a RNG working. Working on block devices.
   - e310 device model modified in QEMU currently; eventual goal would probably be the `virt` target
   - Amit: Intuition is that one big PR makes sense here, hard to evaluate otherwise
   - Leon: Scope is currently ~1 kLoC
   - Branden: This is also something pretty brand new to Tock, so should be easy to classify as experiemental
   - Pat: Yeah, makes sense to bin as 'new board' ish, which are often many-kLoC
 
 - Phil: Timers, 2.0
   - Continuing to test; waiting on new HW to arrive; will be out next week
   - Proposal for allow/disallow out to 2.0 WG
 
 - Hudson:
   - Updated board-based instatiation of chip drivers PR to include Sam4l and hail/imix; now comfortable that it should work for everything
   - To evaluate the gains/potential, made imix-mini; LED, GPIO, and timer only (+panic uart and power mgmt); before this PR, was 65k, now 52k of flash
   - These are huge changesets; thinking will do a 1-PR-per-chip to accomodate, and hopefully get stakeholders from each chip to review more carefully
   - Q: Also removed many uses of `const fn` since constructors no longer need to be const. We are probably stuck with `const fn` because of the register interface, so this won't get us off it. Should that change be kept or dropped?
   - Status of upstream const fn: Register interface is blocked because of generic parameters with generic traits applied; doesn't look like stabilization progress is really moving there
   - Stylistically, it's nicer to have `const fn`
   - More than stylistically, it's semantics: we currently enforce that constructors can't do anything at runtime, which simplifies reasoning about boot
   - This change will now only enforce that if constructors opt-in to `const`; likely to happen via copy-ing, but now things could opt-out
   - Can we add a 'device creation' trait of some form that embeds the `const`?
   - Conceptually, it would make sense, but currently there's not support for `const fn` in traits, so not really an option
   - Q: For folks for whom stable rust is important, are there strong feelings on const?
   - A: Given that it's on the path to stablization, it probably makes sense to keep using it; not much worry on our end to using it somewhat libearlly either
   - Hudson: Okay, will pull out the `const fn` changes
 
 - [Side Discussion] Remaining stabilization blockers?
   - https://github.com/tock/tock/issues/1654 
   - Lingering blockers _beyond_ `const fn` are mostly implementation at this point (removing atomics from deferred call; finishing asm macro porting)
   - But the const fn use is likely a hard blocker for a while
 
 - [Back on]
   - There is non-trivial RAM tradeoff in the current PR; currently bumps by 8k because of MPU alignment?
   - Will investigate more and post more details on size to the PR
   - We should also look into where the 8k min alignment choice comes from; we use smaller values on other boards
   - Yeah; these are somewhat orthogonal questions, but the goal should be to get more "fair" comparison numbers
   - The eventual fix to this is probably to start doing app memory dynamically at runtime rather than via the linker script; likely some other wins related to padding and such too
   - This would also address the issue of the fixed size memory block not growing when/if the kernel shrinks
 
 ## Allow/Disallow
 - Q: Semantics of (dis)allow have come up a few times (expiring app slices, etc), but the ABI question is what should the function interface for userland look like?
 - Idea 1: allow() and disallow() to give and take back
 - May not work well with a Rust runtime. When a buffer is given to the kernel, you no longer have a reference to the buffer!
 - Idea: new allow() causes syscall to give back the old slice
 - Problem: How to grow the buffer? Now needs to be two calls, and would invalidate all existing kernel ref's
 - [disc]
 - I think this now codifies that allow minor numbers all reference one buffer; were doing that anyway, but new to enforce; probably good
 - Should replacement require that kernel gives back the same buffer, or can it give you back a different buffer?
 - How would that happen?
 - Malicious capsule; returning a different shared slice
 - Following thought experiment; imagine driver with read and write buffers, when reclaiming read buffer, given the write buffer back instead
 - This probably doesn't break Rust memory safety?
 - We will have to think through this: Easy answer is that kernel is required to give back the same buffer, but if we can be more flexible, we probably should
 - This question also influences how much state would have to be held in the kernel
 - Any reasonable use case of a write-only allow?
 - Not really.
 - What about a buffer that can't be disallowed?
 - Drivers can always not return the buffer.
 - Digging into the realloc case:
     - To extend *same* buffer, currently need to get the buffer back before sending a bigger one; this would require tearing everything down just to rebuild it once the next, bigger buffer comes
     - Could imagine an allow with a bigger, disjoint buffer that copies over everything
     - This procedure will need to be mindful of hardware (DMA, USB endpoints, etc) that point into allowed buffers
     - Kernel should enforce non-overlapping allow?
     - Probably necessary, otherwise AppSlices alias, breaking Rust memory model in kernel. This is a potential problem currently
