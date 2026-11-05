# Tock Meeting Notes 2026-10-21

## Attendees

- Branden Ghena
- Amit Levy
- Leon Schuermann
- Brad Campbell
- Pat Pannuto
- Johnathan Van Why

## Updates

### Tock Registers
 * Johnathan: Tock registers update is going well. Should be a release in the next couple weeks
 * Amit: I also made a tool to do translation which is a bit vibe-coded but I've been cleaning it up a bunch. Translates from repr(c) to register structs macro.
 * Amit: It's like 1000 lines, but I'm working on it to make sure it's understandable and trustworthy.

### Website Update
 * Amit: Working on a total overhaul for the TockOS website. I'll share some designs in a few days

### LoRa
 * Amit: I got one of the LoRa boards from Seeed studio with an STM. It works, although using LoRaWAN in Tock is hard right now. I'm using RadioLib for TTN and Helium access.
 * Amit: Application is very hardware-specific right now. Also it need Flash access but doesn't have it on the STM right now.
 * Amit: Planning on some updates there to make stuff better
 * Brad: What you described are my takeaways as well. Similar experience


## CPU State Management Safety
 * Amit: Are `reset()`, `wfi()`, and `sleep()` safe? Right now in the Cortex-M crate they're marked as unsafe. But that seems historic about avoiding anything to call it, and not about actual Rust safety. Putting these behind a capability are a better protection mechanism.
 * Amit: PR for new capability for limiting access to these: https://github.com/tock/tock/pull/5262
 * Amit: Also worth considering: are there any memory/type-safety concerns with these before we do mark them as safe?
 * Brad: Reset _is_ a safe function right now. So that's an exception.
 * Brad: Also you might wonder, does this matter if these are in the Arch crate? I say it does, because if we can move to a world where Chips have split safe and unsafe crates, using capabilities would still be valuable for the safe crates.
 * Amit: Even if we were to keep them as unsafe, we should add a capability. They do have ramifications for the kernel, even if not for Rust
 * Leon: The only way I think this is maybe problematic is interrupt management and whether an interrupt can be taken during any given code path. You might have code that relies on no interrupt being taken, but I don't think that's an issue as we're single-threaded. So that code can just disable/enable interrupts on its own. That's the only thing that makes me even slightly skittish.
 * Brad: Can you clarify for me? There's some API you're concerned about?
 * Leon: If there's a way for any of these functions to enable or disable interrupts we'd have to be careful
 * Leon: Sorry, reviewing the PR, I think there's no issue here.
 * Pat: Putting the core into deep sleep is part of this. That's chip-specific for what it means. You could lose RAM for instance. That's an API that sucks. We could have better semantics, although it's a hard thing so I understand why we didn't do it.
 * Pat: I think the naming issue and what it conveys would improve if we ever improve our APIs here. But I think that can be pushed off until anyone decides to put time into that.
 * Pat: For example, I'm pretty sure on the Apollo chips, at deepest levels of sleep only some interrupts can fire.
 * Pat: Right now, instead of just using unsafe we're add the capability. And I see that as fully positive.
 * Leon: It would make sense to me that these could all be safe. One challenge is that it's unclear to me which of these functions fit the capability and which don't?
 * Johnathan: I think we could even make them safe with no capability. They don't seem dangerous to me.
 * Pat: Hard reset versus soft reset could maybe matter. Slightly different initialization path. Our code should be reset-safe independent of that.
 * Leon: Does a soft reset reset the processor but not the hardware? That could matter for DMA
 * Path: Yeah, our drivers should ensure that on initialization they need to clear things.
 * Johnathan: If peripherals could still be doing DMA across a reset, that would be unsafe.
 * Pat: That's platform-specific whether it's even a problem. We could leave `reset()` unsafe because we're unsure. That might hurt Brad's goal of having a safe API.
 * Brad: Again, reset isn't even unsafe today. We could make these APIs unsafe, but we have to document what to do about it. That's the part where this gets hard.
 * Leon: Why do we have soft reset at all?
 * Pat: The real answer is that we don't have hard reset at all. The only way to reset the whole chip is often an external reset pin that touches the whole silicon, unless I'm incorrect. I think there's no cortex-m way to do a reset purely from software.
 * Leon: Where do we use this?
 * Brad: Process Console
 * Amit: Or maybe some kind of watchdog stuff.
 * Leon: So maybe we should get rid of soft reset, and boards should instead provide some chip-specific mechanism for hard reset. We could make soft reset safe for now and accept that the story around DMA is complicated, but it's kind of a hack.
 * Johnathan: If the DMA controller is easy to disable, soft reset is doable.
 * Pat: Many MCUs have their own DMA engine per-peripheral. So not so easy.
 * Leon: I do think this capability is poorly named. CpuControl feels like it would include context switches, and stuff that isn't part of this.
 * Some suggestions: SuspendControl, ArchPrimitivesAccess, CpuStateControl
 * There seems to be some agreement that CpuStateControl might be better
 * Amit: Okay, so change the name and then we can merge this.


## Imix Static Muts
 * Amit: Specifically, I think these are in the tests
 * Brad: Imix has quite a few tests, which all use global static muts. I tasked Claude with moving those all to static init, but some get ugly because the test wants to keep access to a buffer to read/write it but the networking stack never returns it. So the hack is just keeping a pointer around unsafely. I'm not about to rewrite the networking stack
 * Brad: So, we could have the hacky solution, keep imix on 2021 forever, or archive Imix altogether
 * Pat: I suggest we archive it. It's weird to keep in-tree since no one can get it and no one is making it
 * Amit: When's the last time we heard of anyone using it?
 * Johnathan: I'm in favor of archiving it
 * Leon: Imix and Hail are the only two SAM4L boards, right? I have a few Imixes around here, one on Treadmill. It's a shame to lose support for that chip, but I don't know how likely it is that we see a new board using it.
 * Branden: I really don't see new boards using old chips like that anymore
 * Pat: Google says it's "not recommended for new designs" right now. Still exists, but limited
 * Branden: Sounds like everyone's in favor of archiving
 * Leon: Should we archive Hail too then?
 * Brad: I'm still using Hail at least. They were more widely available.
 * Brad: I'm not in favor. I'd rather delete the tests than archive Imix too
 * Amit: I agree with that.
 * Johnathan: That's fine, although I'm in favor generally of deleting unused code
 * Leon: We keep the code in-tree but we mark it as a lower tier such that we're willing to destroy it if it gets in the way
 * Amit: Okay, let's delete the tests then.
 * Leon: We have to un-archive Imix in the tock-archive repo then. Sorry about that, I merged it too soon
 * Amit: Wait, you also fixed the tests, is that not good?
 * Brad: Pat requested changes: https://github.com/tock/tock/pull/5218
 * Pat: You're right that we shouldn't rewrite the networking stack. The comments about moving SAFETY comments around I stand by. But we can throw raw pointers around in order to get tests working
 * Branden: To confirm: are the revised tests in #5218 better than just removing the tests
 * Pat: Is anything running these tests? And could there be, maybe on treadmill? If so, then it's worth it.
 * Amit: Yeah, we could do that
 * Pat: Okay, I think we should keep the tests around then.
 * Amit: Final result, keeping Imix and fixing the tests


## PR Review
 * https://github.com/tock/tock/pull/4906
 * Amit: I think the outstanding thing is to move the subsection in TRD104 that deals with non-u32-values-and-arguments to section 3. I don't recall why it was in section 4 before.
 * Pat: Does the change to TRD104 need to be part of this PR?
 * Amit: I think this is the only outstanding thing and they can be combined.
 * Branden: I agree that should be section 3
 * Updated and merged
 * https://github.com/tock/tock/pull/5299
 * Amit: This is my tool re-doing Brad's work
 * Brad: My version manually updated the addresses, which I argue are easier to read than than the math equations

