# Tock Core Notes 2020-09-25

## Attending
 * Pat Pannuto
 * Johnathan Van Why
 * Philip Levis
 * Amit Levy
 * Leon Schuermann
 * Alistair
 * Brad Campbell
 * Hudson Ayers
 * Branden Ghena
 * Samuel Jero
 * Vadim Sukhomlinov

## Updates
 * Phil: Timer merged!!
 * Brad: elf2tab rounds application outputs to power-of-2, because of Cortex-M MPU limitations. For Risc-V, this power-of-2 isn't a requirement, so looking at moving the logic out of elf2tab
     * Amit: Sounds reasonable, perhaps tockloader responsibility?
     * Brad: Actually, it is already a tockloader responsibility. The multiple compiled binaries already a major R5 complexity
     * Amit: Yeah, seems like a pre-tockloader choice whose time has come to move
 * Johnathan: OpenSK is working on adding NFC support for 52840; also working on CryptoCell, but that's moving slower b/c very complex

## Interrupt control semantics in Tock/HIL
 * Amit: Summary: Who in the kernel is responsible to enable/disable NVIC on ARM / equiv on R5?
 * Phil: Not just about who, but about what happens
 * Phil: Q's are: Can an interrupt wake a processor, and do you do something in response to it?
 * Phil: Today, semantics are unclear and somewhat varying on what happens when interrupts are enabled/disabled
 * Phil: Things seem to be tied more to how it's implemented rather than a concerted design
 * Phil: ARM originally did a FIFO queue in top-half handlers
 * Phil: What happens if queue overflows? Replaced with a no-queue design that scans over what interrupts are pending. Problem: no way to tell the kernel, "don't call this handler" (i.e. active interrupt wakes processor and masked interrupt is pending, it will be called)
 * Phil: This came up again with R5; thought is maybe that core kernel loop should control which interrupts should be enabled; but what if a peripheral wants to disable and the core loop overrides?
 * Phil: Personal opinion: peripherals should own their interrupts -- they should decide if the core will wake and if their handler will be called
 * Phil: If that's not tractable, will have to push the operation into handlers (i.e. first line of handler just checks if handling now and ignores if needed) [editor's note: may be more complexity here]
 * Amit: The current state is that capsules do not control whether interrupts can wake the CPU. In general, they cannot access the NVIC methods that control this
 * Phil: Directly, indirectly, or both? e.g. if SPI capsule tells SPI HW to turn off, does that count?
 * Amit: That would count... as far as I can tell, there should not be and are no references to nvic enable/disable except for board configurations and in chip handle_pending_interrupts functions
 * Amit: Today: capsules & low-level drivers should not and cannot affect interrupts
 * Amit: Not necessarily a thought-out design decision; was a solution to the FIFO queue at the time
 * Amit: The way that peripherals do have control is ensuring that their peripheral will not generate interrupts at all
 * Hudson: There was an exception in R5 around mtimer, but that was just removed
 * Amit: Is thre something like NVIC on R5?
 * Brad: Yes. There's two levels. There's an architectural state register, which has bits for different interrupts, one of which is the external interrupt, which maps to something akin to NVIC [there are multiple implementation of this, commonly the PLIC]; inside PLIC there are mappings to peripherals
 * Brad: Caveat: that's a simplification of course
 * Leon: Important to note: Many chips have a PLIC, but the thing named PLIC varies somewhat wildly
 * Phil: On idea on Cortex-M was maybe to do this in a handler via software bits. For R5, there are 1024 which is very expensive to scan for wakeups
 * Amit: And you don't have to do that to find the pending bits?
 * Phil: Yeah, R5 has a magic "give me next interrupt" register
 * Amit: Ah, which is effectively what we were trying to do in software on ARM...
 * Amit: To move forward...
   * 1. Agree on what current semantics are
   * 2. Whatever PRs we have outstanding should go in if the match the existing semantics
   * 3. We should then [task force?] figure out what desired semantics should be
 * Amit: Towards that last point, what do we actually want for flexibility for drivers; what's the ideal interface?
 * Alistair: Drivers, or for handling interrupts?
 * Amit: Well, the interrupt logic is in service of what drivers want to do. My understanding is that the current interface doesn't give enough flexibility to drivers. e.g. on ARM a driver cannot mask themselves off on the NVIC
 * Alistair: Think drivers shouldn't have affect on the global interrupts -- they can stop themselves from generating interrupts, but shouldn't suppress interrupts. But a *board* should have the ability. e.g. board can disable USB interrupts b/c the hardware is broken
 * Alistair: The problem with the current PR is that the timer core is making assumptions about interrupt state (e.g. always turn on timer) rather than checking what was there before
 * Phil: Q, are you saying that drivers should not be able to clear out PLIC or NVIC, even for themselves?
 * Alistair: Yes, I think so
 * Phil: What if I have a driver with multiple things that can cause interrupts (e.g. 7 gpio pins), how do I make an atomic change to behavior .. eg if this is across multiple registers?
 * Alistair: Is it any different if thery're across multiple registers on the PLIC?
 * Phil: Typically you would globally disable interrupts; trying to think of cases wehere interrupts are spread across multiple registers, but can't really think of any
 * Amit: e.g. sam4l where each GPIO bank has different NVIC entry – but think there is still one register for interrupts
 * Alistair: but whether devices can or cannot clear interrupts, we still shouldn't be forcefully enabling things in the interupt handler
 * Phil: Yeah, agreed. The question is whether the 'driver no access' works always; 90% of the time, probably; but there are cases where you need to disable in order to be able to do atomic operations
 * Leon: If it's only atomic operations, wouldn't it make sense to have a general atomic closure?
 * Pat: We do have this, but there's inherent unsafety here to gloablly disabling interrupts
 * Leon: Yes, but this is a rare use case, it's already `unsafe`; likely within the scope of auditable
 * Amit: I'm not convinced that doing an atomic CPU operation would give the semantics that you wanted. e.g. I believe that on ARM it prevents ISRs from running. It does not, however, prevent pending bits from being set. So if the semantics that I care about is I get interrupts for all or none of the events that I'm messing with, then disabling interrupts doesn't help
 * Amit: Conversely, the ISR running has some performance overhead, but likely not a funcitonal issue
 * Amit: Could check in ISR if things have associated interrupt; only interrupts execution temporarily
 * Phil: I think the way to think about this is, have complex periph trying to set up operations for, and while this complex setup is happening, don't want ISRs to run. Given Tock's concurrency model (i.e. push everything to bottom-halfs) it could be that this already just goes away
 * Amit: Yes. Though it would be good to cover this expliclty
 * Phil: Yes.. because of Tock's sync model, many of the typical problems are elided, but would put $ down that there are still problems hidden somewhere here
 * Phil: Ran into this on H1b
 * Amit: The more dramatic option would be moving the whole kernel to the top half
 * Amit: We only use this top/bottom mask model to allow pending bits to sit there
 * Phil: Just realized one use case: Have an on-chip peripheral, with multiple interrupts, during power-down either clear all config or just turn off interrupts
 * Amit: Practically speaking, is that a way that chips behave?
 * Amit: e.g. on SAM4L interrupts are disabled implicity if peripheral clocks are disabled
 * Phil: Yeah, again this is a 90% time works out; but need to support the weird cases
 * Vadim: How might this interact with future multi-core considerations?
 * Vadim: Some of our projects are looking to things such as I/O core and compute core, might want to figure out how to adapt Tock to this; know there are a lot of assumptions about single core in Tock now, but this is an opportunity to look ahead
 * Amit: We've thought about this a few times; it's involved, but we've largely punted on the issue – we should chat more post-call
 * Phil: What I was hoping for from this discussion is being able to say, "when you disable an interrupt, and you ensured that your handler will not later be invoked"?
 * Amit: For clarity, let's call NVIC interrupts and ISR the top-half handler
 * Amit: if you disable an interrupt in your driver, the gaurentee that you have is that hardware will not set an interrupt bit [however: it may have already been set]; this means that the ISR may have run and your `handle_interrupt` function may be called in the future
 * Phil: That's usually not that expected semantics, if you disable interrupts, you expect pending to be cleared
 * Amit: Yes, so today, you have to do the additional work of clearing your own pending bit in the hardware. But today, driver's can't control NVIC directly and can't clear directly; there's one case in upstream STM where `clear_pending` uses unsafe correclty to do this, but not explored deeply
 * Amit: Generally, seems it should be safe to allow access to pending and not break the contract that interrupt handling code expects
 * Phil: Comfortable if that's the preferred approach, want to minmize unsafe of course, but this may be a needed escape hatch
 * Amit: Current answer is that this is all low-level logic happening in chips crate, so unsafe is accessible. Calling enable/disable will do unexpected things because it'll get re-enabled elsewhere later. Messing with pending will probably do what they want.
 * Amit: Again, this is descriptive of today's model, not necessarily right
 * Phil: Looking at the NVIC for cortex-M, instantiation is unsafe, but none of the methods are unsafe
 * Amit: Right; however I believe that we never pass an NVIC to any driver
 * Amit: Again, the one exception found is in the STM who made an NVIC on the fly
 * Amit: Really, that NVIC interface is a holdover from when we were passing NVICs instances around, and maybe should go away

## Libtock C Switchover
 * Phil: New Alarm API in kernel with new associated syscall
 * Phil: Current driver also allows old syscall
 * Phil: Would be good to update usersapce
 * Pat: We do parallel libtock-c releases; so this should be a 1.6 blocker and testing should be atop this new interface
 * [consensus]

## Tock 2.0 syscall ABI
 * Phil: Primary goal today just to raise awareness
 * Phil: Long ago decided that it'd be good to switch over to results rather than `ReturnCode`, hasn't happened yet
 * Phil: Issue is that in the kernel the simplicity of `ReturnCode` as a value was useful early on; may be able to just move over to `Result`
 * Phil: All of the complication really is translation to formal syscall return values; ideally this goes away with 2.0
 * Leon: Proposed a transition phase where both are used, is that in there?
 * Phil: Yeah, this code in there now doesn't change the Driver trait -- this just changes the ABI. Haven't had a chance to look over your proposed soultion yet
 * Phil: Instinct: transition periods stretch on, easier to do a clean break
 * Amit: Really an orthogonal question
 * Phil: Today, uses current driver trait and translates into syscall return values
 * L/Phil: We'll look at each other's code
 * Phil: Long-standing Rust wishlist miss: Can't match on enum values, ergonomics a bit worse as a result

## USB/CTAP
 * Brad: Hard to talk specifics w/out Guillaume, but can talk higher-level issues
 * Brad: Have two different takes on a chunk of the USB stack right now; need to figure out how to harmonize
 * Alistair: Really don't both, that seems like a bad idea in the long run; constant USB HIL refactoring etc
 * Alistair: Big thing that I think is important is separate USB HIL
 * Hudson: Seems like part of the concern is that OpenSK has had this implementaiton for a while; hesitation to swap out working code for something new
 * Amit: Presumably their version has a working userspace? As it's been there for a while?
 * Alistair: Yeah, theirs is in one of the SK repos; my PR has one in libtock-rs
 * Brad: Userspace impl's is a bit of a red herring; shouldn't drive choice here
 * Brad: Can expose two syscall driver interfaces in the short term
 * Brad: Don't want different USB stacks underneath those syscall drivers
 * Amit: That seems reasonable – how do we move forward w/out G? How different are they under the hood?
 * Alistair: They aren't that different
 * Alistair: I don't like two syscalls as much, but that's not as big of a problem as two stacks; seems like one stack two syscalls good place to be for now
 * Amit: Also probably easier to just maintain the syscall layer out-of-tree
 * Brad: Right. So the core issue seems to be should we have interfaces/traits in USB for CTAP/HID?
 * Pat: I think this boils down should USB have a "CTAP+HID" (current OpenSK) or USB adds "HID" and then a separate "CTAP" atop that (other PR)
 * Brad: Q is whether we should add layers of abstraction when there's only one user of each layer [tinyos problem?]
 * Phil: HILs are hard to change once they are in; seems priority should be sorting what USB HIL looks like
 * Alistair: That's the big difference; OpenSK version has no HIL
 * Phil: Having a HIL seems important
 * Brad: Important to remember that this isn't a HIL that touches hardware -- just a software HIL. Not going to have 15 impls, maybe just 2
 * Phil: Doesn't mean you can be sloppy about it
 * Brad: Yes, but also not the same barrier to change as something like the time HIL; more realistic to change
 * Amit: Maybe closer to some of the interfaces in the networking stack rather than timer HIL
 * Hudson: Yeah, those networking traits are just in a net/ folder
 * Amit: Seems the thing to do would be to have a call with Al + G + other salient folks. If the underlying implementations are really that similar, should be able to come to a consensus
 * Amit: If we have an interface, should be able to support both use cases
 * Brad: We're having an email discussion already and that's not really resolving – need a dedicated call
 * Phil: Important result is that this ends in one API
 * Amit: Userspace or kernel?
 * Phil: both.
 * Amit: Think in the short term, converging in the kernel is important; but userspace can be deferred
 * Amit: I can organize and moderate this
 * Alistair: I really don't think they are actually that different
 * Hudson: Yeah, I really think this is talking past each other a bit on github, and a phone will should hopefully resolve this quickly
 * Hudson: Really important for Tock to support both OpenSK and OpenTitan well
 * {consensus on these points}
 * Alistair: Happy to keep updating, just want to make sure work doesn't go to waste
 * Amit: Sounds good, I will set up call
