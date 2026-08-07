# Tock Meeting Notes 2026-08-05

## Attendees
- Branden Ghena
- Amit Levy
- Pat Pannuto
- Brad Campbell
- Johnathan Van Why
- Leon Schuermann

## Updates

### Interprocess Communication
* Branden: IPC PRs are up: https://github.com/tock/tock/pull/5015
* Branden: This is a design from the Network working group. It's got the ability to discover processes and do basic communication between them. It's not everything we want to do with IPC, but it's a starting chunk of it and it's ready for people to look at. Comments and questions are very welcome.

### Tock Registers
* Johnathan: Safe DMA design in tock-registers pushed to PRs
* Amit: How close are we to being ready to upstream tock-registers changes into Tock?
* Johnathan: We were going to release 0.11 first. Checklist for release is here: https://github.com/tock/tock-registers/issues/45 Most of this is 1-2 weeks of work and straightforward. DMA support is the biggest one

### Removing VolatileCell
* Pat: I have PRs to remove VolatileCell entirely as I think we've been using it incorrectly. All uses were either unnecessary or subtly wrong. Existing types can replace it, so that's what I'm doing.
* Pat: The biggest open question is here: https://github.com/tock/tock/pull/5031 There are other problems with that PR which makes it problematic.

### Treadmill
* Leon: Rewrite work still underway. Some basic deployment of images running with a board exists, and I'm hoping to be able to demo this soon.
* Johnathan: Is the rewrite in Rust?
* Leon: The prior version was in Rust too. This replaces more than 50% of the existing codebase. Basically working, and hopefully more reliable and maintainable.


## nRF52 DMA Register Manager
* https://github.com/tock/tock/pull/4990
* Brad: Revisiting lingering issue. We switched some peripherals to use the new DmaSlice infrastructure. That does things to encapsulate unsafe code in its own struct that just manages DMA. The bulk of the driver then is safe and could exist in a forbid-unsafe crate. We have some examples of creating these DMA managers for nRF, STM, etc.
* Brad: The manager encapsulates register access and DMA operations. One safety guarantee is that there must only be one of these register managers created. For the UART we just sort-of ignored this for now. The AES also needs this. The UART is special though, since we need it for normal operation and for panic cases. Right now we create two managers, in violation.
* Brad: So this PR fixes that. The register manager is created by a board now and passed into a UART capsule. So the board can make this manager once, give a reference to the UART system normally used, then later give a reference to panic handling.
* Brad: This also enforces only creating a single manager with the only_once! macro, which panics if ever called twice.
* Branden: So you have one manager in the board, and two references to it, which go to two UART capsules.
* Brad: Close. The references go to the default peripheral instantiation and to the panic resources.
* Leon: I think this is close to the design I had in mind. The only major thing is that we don't handle the safety invariant that we're constructing this on a board with a UART at that memory.
* Johnathan: Also, nothing prevents other code from accessing the DMA. If you board file were to pull in embedded Rust crates that talk to this DMA UART, there's no unsafe anywhere that prevents this from racing.
* Leon: I think that's the same thing I'm saying. You need to assert that this is running on the proper chip.
* Johnathan: We know this is a single-core chip, but we don't know that an interrupt isn't also touching these registers.
* Leon: Are you saying that the manager implements send and sync?
* Johnathan: Like if a completely different crate is using code in the interrupt which accesses the interrupt. If the main thread is using the UART Registers Manager and is setting up a DMA operation, then later code runs in an interrupt context which runs different code which modifies the UART, then later when the UART Registers Manager continues state is messed up.
* Leon: So this would be an entirely different driver not accessing the registers through our Manager, and runs concurrently
* Johnathan: Yes. We'd need to assert we're on a single core system, and also disable interrupts while modifying DMA
* Leon: I think having the construction of the Manager be unsafe is enough to handle both of those issues.
* Branden: I think it's reasonable that by creating the Manager you are asserting that nothing else will mess with these registers.
* Pat: And in the Tock context, there won't be other alternative drivers running from an interrupt context. If you have multiple concurrent drivers in Tock, that's not our design. The board owning the entire peripheral address space is a reasonable assumption in Tock.
* Leon: I agree. But the presence of a load-bearing safety comment suggests that the function under it needs to be unsafe.
* Pat: Okay, so the alternative to that would be that we have something that is a single authoritative provenance for access to all MMIO memory and hands out strictly unique slices. So if a manager takes an existing slice to memory, then everything is fine.
* Leon: It's totally fine to have a single-use capability to create a Manager. Created in chip or board instantiation and consumed in the driver.
* Amit: Returning to the specific topic, I can see in theory where there are scenarios where this Manager is useful. In this particular scenario, it's needed because we have to use the same UART controller in normal operation and in panic handling. And the UARTE on the nRF52 only operates through DMA. So I wonder if this is avoidable in this particular case, by using the UART peripheral instead of the UARTE peripheral on the nRF52, which has normal register-based operation instead of DMA.
* Brad: That would work I think
* Branden: I believe the NRF documentation goes out of its way to avoid talking about the UART and treats it as deprecated. It does probably work though.
* Amit: I don't see that in the documentation.
* Leon: Part of the motivation is that we have the UARTE peripheral in the normal context, and reuse it in the panic handler. 
* Brad: Yes, if you panic in the interrupt and a single-thread-value in the peripheral isn't available, then you just won't print anything. That would be another reason why using a non-DMA peripheral would be nice.
* Leon: We could have a good argument for safely re-initializing the hardware to print from an interrupt context.
* Brad: Every other board uses a byte-by-byte synchronous transport for panic handling. So switching the nRF to that would be nice.
* Amit: The problem might occur again later, but punting on it for now is nice. So that's my opinion, without prejudice about the design of the Manager.
* Brad: So there's still the lingering issue of the DMA Manager. I'm just comfortable with saying that Tock is unsound in the presence of other non-Tock code which touches MMIO and breaks the kernel.
* Johnathan: That sort-of pushes the safety invariant to the crate itself. Saying it's unsafe to include this crate in your Cargo.toml except in specific cases. That doesn't make me very happy though. The invariant is still there, just pushed to the build system
* Amit: We could mark some specific function as unsafe to fix this?
* Leon: Specifically the register Manager constructor.
* Amit: There is something fundamental about this problem. The vast majority of the Rust standard library isn't marked unsafe for use, but caries an invariant that you're running the Linux version on Linux, and not on some other system. And more subtle invariants about the meaning of buffers you got from a filesystem through libc. So there is some line here where we don't have to guarantee everything about the state of the world in code.
* Leon: That is a fair argument to make. But there are conditional compilation attributes where std is only built when actually on Linux. It does check for properties of the system. It makes it virtually impossible to create a mismatch which ends up running. What we are doing is creating a crate, which is different from Rust's safety arguments if we decide that importing a crate has safety implications. There's no precedent in Rust for this
* Amit: What's the problem with making the Manager constructor unsafe?
* Brad: It ends up propagating all the way to the board. Making peripherals requires a call of unsafe, and then requires the author to validate safety invariants that they likely won't understand and are just copy-pasting safety invariants for anyways. And Tock will end up working in both cases. Seems like a bad user experience to resolve an issue that the chip crate author should really be the one dealing with, rather than the board author.
* Brad: The UART manager constructor isn't unsafe right now, while the AES manager constructor is.
* Branden: The same argument applies for UART and AES though, right?
* Brad: Right now we wrap it one layer up before it gets to the board.
* Leon: We can do that for others too. Have one unsafe function for the board to call, and it handles the others.
* Johnathan: The safety invariants it would have to support are 1) it's running on the right chip and 2) nothing else is touching the MMIO registers
* Amit: Right chip is fine. Being the correct address is the static ref constructor which is buried in the chip. Then it needs to ensure uniqueness of that reference, that I'm not passing two different static refs that are the same MMIO memory anywhere. I think those are actually the invariants. Are those actually handleable by the safety comment on the static ref?
* Leon: Not really, I think. One good counterexample is that way we're using static ref and a constant means that a separate crate could use that same constant.
* Amit: If the safety comment on static ref says it's only safe to call new if this MMIO address is indeed the proper peripheral and it's the only reference to this MMIO address. If that's a reasonable requirement on static ref, then I think the DMA Register Managers can be safe because the invariants we care about are carried over from Static Ref.
* Johnathan: I think static ref is a bit of a red herring here. The chip crate does know the peripheral's address and should assert that. The board knows the chip and asserts that. So the addresses in the static ref don't show up in the chip crate's API.
* Leon: I don't think the fact that a peripheral exists behind an address is worrying. The invariant I'm more worried about is that only one thing accesses this MMIO address. The problem is that this is a global safety invariant, which we really can't reason about from within a chip driver. It doesn't know how it's composed. Only the main crate can reason about that.
* Amit: In the case of the nRF52 default peripherals, this would work if new was unsafe. With the invariant that you're calling it at least once
* Leon: Yes, and you're not constructing other drivers over those addresses.
* Amit: It's okay for unsafe in the chip to have that kind of requirement. I'll point to the standard library. There are things I can create references to, including arbitrary memory, from a crate a program happens to use, where the std crate also needs to be the only unique use of it.
* Leon: I think the std library argument is the closest I get to accepting that you can reason globally. I think it's a stretch though and not a common safety argument. We'd really want to document this somewhere as an important claim.
* Amit: The nRF52 default peripherals does still need an unsafe constructor.
* Leon: Okay, that I think can make sense. So you're still bubbling up global invariants to the main board crate.
* Amit: Yes, you're requiring the user of static ref to assert a global claim. And that will need to bubble up.
* Leon: Cool. So it doesn't matter who creates the static refs
* Amit: I think the useful part is that there is a meaningful safety invariant for the caller at each level.
* Johnathan: I'm not sure I'm following. So the invariant on static ref is that there's no other code touching the peripheral it points to. Then which crate constructor creates the static ref?
* Amit: The regular nRF52 crate internally creates a bunch of static refs which are private. Created inside nRF52 default peripherals new constructor. What would change is that he nRF52 default peripheral constructor becomes unsafe. The safety invariant of that constructor becomes something like we're calling this at most once and nothing else constructs peripherals for the chip. I think that's a straightforward invariant for a board author to reason about. Then specific peripherals can be marked as not unsafe. Because the invariants that matter for its safety are carried by the static ref that it operates on.
* Amit: Now, if a board wants to create a different set of peripherals using the same stuff. It would indeed need to create static refs which would have invariants that couldn't be satisfied anymore.
* Amit: Note: I didn't consider if static ref is copy or clone or something
* Leon: We'll be replacing it in the tock registers rewrite anyways.
* Leon: I agree that what you're proposing is soundness which is well encapsulated.
* Brad: You claim that the board can understand this unsafe thing. It would help to see what the comment is. The author is just going to trust that comment.
* Brad: I'll also note that unsafe is infectious. If you mark the peripheral constructor to be unsafe, then it invites other unsafe things to go into that function. Which is frustrating. The more functions that are marked unsafe, the easier it is to sneak unsafe operations into it without a new safety comment.
* Amit: Actually in rust 2024, there would be zero unsafe blocks inside the constructor, just around creating static refs. In order to do new unsafe operations inside that function, you'd need to make an unsafe block and comment it.
* Brad: The default peripheral is just a convenience thing though. What if you don't use it?
* Amit: If you don't use it, you need to create the peripherals yourself. In which case, creating those peripherals requires static refs. Which require unsafe.
* Brad: But if there's no unsafe in the default manager constructor?
* Amit: There is implicitly through the consts.
* Leon: The problem here is that it's implicit. The only remaining issue I see in what Amit is noting is that putting it in consts isn't sufficient. So there should be unsafe construction of static refs inside the constructor.
* Amit: Okay. Sure. There are unsafe blocks in there for creating the static refs. And then if a board didn't use default peripherals, it would create the static refs itself which requires unsafe.
* Brad: But the static refs are in the crate.
* Amit: But they're not public. So if the board wanted to create its own, it would need unsafe.
* Brad: But the board doesn't know the address is correct. That should really be a chip assertion.
* Amit: It's fine to have public addresses.
* Leon: Then the converting of an address into a type is what is unsafe.
* Brad: I have a design that splits these two https://github.com/tock/tock/pull/4978
* Leon: I do think that unsafe is infectious, but I think this isn't so bad. It's just two layers
* Brad: The concern is that it's easy for people to throw extra unsafe stuff into the function
* Leon: I'm hopeful that Rust 2024 helps with that
* Johnathan: And in the new tock registers, many of the unsafe functions are auto-generated, so it's not possible for someone to just place unsafe code in those.
* Brad: Is anyone convinced that the chip crate should encapsulate all unsafe of creating the chip?
* Amit: I like the idea, but I don't want fake encapsulation.
* Brad: The concern is that this is a crate, and something could attempt to compose two crate?
* Leon: Reuse and composition.
* Brad: I'm still uneasy about this uniqueness requirement. Feels fundamentally different from a lot of unsafe we can reason about locally. This feels like a global property. So we push to the board and have the board main.rs author write a meaningless comment they probably didn't check.
* Leon: The point is to have a set of rules making it clear who is responsible for which invariants. If you discharge the invariant by saying "I didn't create two things that use this peripheral" that is sufficient due diligence.
* Brad: But the problem could still be there
* Amit: But that's worse if the function isn't marked unsafe. As it's harder to notice the invariant exists.
* Brad: Unlike "is this pointer aligned" which is straightforward to reason about locally, this property is global and requires higher-level thought, but with nothing actually verifying it. So I want to make the point that it feels like we're using a blunt instrument for something that's not quite what we need. And we really need some other tool to match the ethos of what Rust really promises. Feels unsatisfying. Feels like a wishful best-effort thing rather than actually being rigorous
* Amit: I agree somewhat. I am not sure that this is so hard to enforce in a system. It is a problem that there isn't a way to ensure which of your dependencies might use unsafe. However, in principle the responsibility of a board author (or anyone composing a safe program) is to establish trust in the uses of unsafe in all of their dependencies. In general, in Tock that's the board crate you're writing, the chip crate you're using, and the kernel. And maybe some crypto crates. It's highly reasonable to say "you should look at all the unsafe code" or trust the author's description of what this crate is doing and what it might touch. In that world it's reasonable to enforce this requirement globally.
* Amit: If the default peripheral constructor was safe, it would make it harder to see uses of it, even though it has this invariant.
* Amit: If we had something enforcing the invariant, it would be nice to have that checking this.
* Brad: We document Tock external policies though
* Amit: But the board crate could call the constructor twice
* Brad: In this PR, it'll panic if called twice
* Amit: The board crate can create its own peripherals as a bad way of overriding what default peripherals does
* Brad: You'd have to use your own static ref and do some unsafe operation.
* Leon: You could see a design in the future where there are multiple crates per chip, and you call multiple default peripherals to instantiate things, needing to be careful about what you're doing to not instantiate two overlapping peripherals
* Leon: I don't like that this is the position Rust puts us in. Grant allocation and number is the same global reasoning issue. If rust had global reasoning like global counters from nesC this would be solvable. But that's totally absent from the language.
* Brad: Okay, I think this is progress on the issue.

