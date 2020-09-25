# Tock Core Notes 08/07/2020

## Attending
 - Johnathan Van Why
 - Brad Campbell
 - Leon Schuermann
 - Hudson Ayers
 - Phil Levis
 - Sam Jero
 - Pat Pannuto
 - Amit Levy

## Updates
 - Leon: Got a networking stack over ethernet working in QEMU using VirtIO. Want to submit a PR to Tock eventually.
 - Johnathan: Got libtock-rs CI fixed

## Tock 2.0 Timeline
 - Johnathan: ChromeOS team would like to know how long until Tock 2.0 can be expected. They sort of have a deadline of December, but I realize that you probably don't have a firm timeline
 - Amit: I think we all would certainly hope to be done much sooner than December. I think the most heavy weight thing remaining are the allow/unallow changes.
 - Phil: Lets aim for mid september. I think we can have all the major implementation work done by the start of the semester (late August) and then work on testing and stuff during September.
 - Amit: Would have to ask Guillaume about his availability, but this sounds reasonable to me
 - Johnathan: We had also said we would revisit threat modeling. I think we still need to make grant regions round robin and support very small (<3 kB) apps, which can be a breaking change.
 - Amit: I have been spending more time on Tock lately and have a lot of time scheduled out. Unfortunately a lot of that time has been spent on getting a dev environment set up for OpenTitan, but I think mid september seems like a totally doable goal and would make us both happy.

## Alarm System API Tick Width
 - Phil: I implemented the new time/alarm API for the underlying 64 bit RISC-V timer. I expected it to not compile because the virtualizer assumes 32 bits...but it did compile. Which is unfortunate, because it would be totally broken. It compiles because Rust seems to coerce u32 to u64 automatically, but what would actually happen is that as soon as you ticked over the 33rd bit you would enter a spin loop of firing interrupts.
 - Phil: So we have a virtualizer, so we can design it to work with any underlying tick width. Or we could keep the 32 bit virtualizer, and put a shim in between it and non-32 bit hardware timers. Or the virtualizer could be designed to handle anything 32+, but for things smaller than 32 (e.g. 24 for nordic), use a shim there. This is good because 32+ handling is a relatively small change, but there is a lot of logic for smaller than 32 bits, and we largely seem to consider the 24 bit hardware timer as an edge case.
 - Amit: For option b, which seems best, is it simple for anything greater than 32 bits, or only 32/64/128.
 - Phil: Anything greater than 32. Basically the virtualizer is keeping everything in terms of 32 bit time. At the moment at which it actually sets the compare value, it looks at what the actual value is in terms of full width, and then converts the compare value that is set to the appropriate number of ticks in the future. Really just a couple line change.
 - Amit: So this would apply to the system call interface?
 - Phil: This would mean the system call interface for alarms would always have a 32 bit width.
 - Amit: So downside is that we are sort of giving up on potential benefits of userspace being able to rely on a 64 bit counter.
 - Phil: We are not giving up on it, we could transition in the future anyway. This is more about remaining compatible.
 - Amit: So OpenTitan could have a 64 bit stack.
 - Phil: You would need a 64 bit timer system call.

## Generic Bus HIL
 - Brad: This comes from a PR, thought it would be good to discuss. Basic idea: should we add a HIL that is a generic bus, that could be multiple implementations of SPI I2C etc., rather than having multiple implementations of buses, a sensor driver could be implemented on top of the Bus HIL. This is useful for sensor drivers that support multiple interfaces. I am cautious on this because it seems like maybe too much abstraction, because any one hardware platform will only have one version, so testing all options is hard, and in general I think it is going to be difficult to have a very robust generic Bus interface that can actually work on top of multiple different hardware interfaces. Maybe we could make it not a HIL..?
 - Leon: I am not quite sure how you would do the abstraction, because each sensor supports different subset of these transports, and have specific serialization on top of each transport
 - Brad: Good point. Is the implementation on top of the interface always the same if the interface is different?
 - Phil: It often is not.
 - Amit: Right, like sending a command to a register over SPI or I2C might be done similarly on the same chip but not for all chips or across different chips.
 - Leon: The usual approach I would have taken with Tock is to split up my sensor driver into two layers, one for the sensor driver and one for transport. I think we already have a better story for this than most OSes.
 - Phil: My initial position is this sounds very cool for a particular driver, and I need to look at the code, but probably should not be a HIL because I have my doubts about the generality of something like this.
 - Pat: I thought I remembered the reason for pushing this upstream is they are using this on multiple sensors already. I think starting this as a library or a capsule is the right way to go.
 - Amit: This is #2042, right? Yep
 - Amit: Concern of making this a HIL is that it might push too many people to use it where it is not appropriate.
 - Brad: I think making this a capsule is probably best for now. 
 - Pat: Concern for board integrators -- how can they know how to initialize a sensor using this HIL?
 - Phil: Yeah I have some concerns about the implementation now that I am looking at it. There are a lot of edge cases for configuration of these buses.
 - Pat: Yeah read/write is the same but otherwise this stuff is different.
 - Brad: Sounds like we don't want a HIL, lets continue this on the PR.

## Dynamic Grants
 - Amit: PR 2053 from daboross
 - Amit: I spoke one on one with author about this. He found a bug in the interface for allocating dynamic grants. To remind folks, this has never been used except for in the SOSP evaluation. It is an interface for allocating more memory to grants outside of the base structure allocation. Unfortunately the current interface does allow you to leak grant memory, which breaks everything about grants. This has not been a problem because noone has used this interface, but David wants to use it for BLE, and found this issue looking through the code.
 - Amit: We discussed how to solve it and this PR broadly follows what we discussed. I think this is a really important problem to solve, we could just remove the interface, but I think the fix works. Obviously we should think quite carefully about this as it is subtle and we have gotten it wrong before.
 - Brad: Not sure what we need to discuss, just wanted to put that out there.
 - Amit: This is worth having other eyes on.
 - Phil: How do we test to know its not wrong?
 - Amit: Yeah....it can be hard to write tests for this sort of thing, because if it works well the failing tests should not compile.
 - Hudson: I believe you can write tests that only pass if the code they are testing fails to compile
 - Sam: I don't think you can do that on `no_std` code actually
 - Amit: There are other challenges for writing complete coverage tests for this.
 - Amit: However maybe there are tests that can compile that are worth writing for this. Dangerous case is avoiding dangling pointers.
 - Amit: I will put tests on the docket for the PR 

## Rubble BLE dependencies
 - Amit: David is working on porting rubble, a rust library for BLE, to Tock
 - Amit: There is a question of whether to permit in this case external dependencies, because it would be great to have bluetooth
 - Brad: Yeah, my compromise point is to have the external dependency only exist in the boards folder. That would also be responsible for any translation logic between the Tock interface and the rubble interface. Any boards that want to use rubble would have to use that crate and include that external dependency in their tree.
 - This PR is not ready, just wanted to see if we agree this is an acceptable path for integration
 - Amit: I think that is a decent compromise
 - Amit: I think that if we step away from the upstream repo, this would obviously be fine.
 - Hudson: Would restricting rubble to the Tock ble interface make it worse?
 - Brad: not sure...we could change the Tock interface or introduce a rubble specific interface that does not depend on the actual Rubble code
 - Amit: In a sense, it would be an example of the idealized version of the Tock ecosystem where some of these complex drivers would be contributed, and not be part of the core.
 - Sam: So the downside of putting all that into boards, is that if you want an example which uses that stack by default, then the nrf52 has to take that dependency by default.
 - Amit: Right.
 - Sam: So I think the current scenario is all the capsules are in the same crate. Should we have a crate for the bluetooth capsule that takes the dependency.
  - Hudson: The idea here is that the driver implementation could live in capsules but not have any external dependencies. The code with dependencies would depend on this capsule, but not the other way around. So the dependency is still at least constrained to the board.
 - Amit: Side question: In practice, Rubble can be compiled with no unsafe. If we had a way of enforcing that in the build system, would we have the same problem with external dependencies.
 - Brad: IMO no.
 - Sam: I thought it was not just about unsafe, but avoiding magic code from elsewhere.
 - Amit: It is kind of a combo. We think of most capsules as a collection of contributed drivers.
 - Hudson: We still trust capsules for liveness
 - Amit: That is true
 - Amit: Okay, I just thought this helps think about where we would put this. But we don't have such a tool anyway. So putting it in a particular board, or in a contrib subfolder, makes sense to me.
 - Sam: one of the dependencies is SHA2, which seems better than reimplemented SHA2.
 - Brad: sounds like no major concerns

## Chip Driver instantiation/interrupt handling rework
- Hudson: We use static global variables for our chip drivers. This is problem: I came up with a design that solves this. Rather than instantiating as global static must, instead, they are instantiated in main.rs and installed in the platform struct. I took this one step further, so the platform handles the interrupts, give chips a reference to the platform object. The chip can then forward to the platform implementation. Boards can exclude chip drivers they do not want. Currently, a board has to include all chip drivers.  Also, by not relying on all of this static mut, we can get right of const function use (nightly) except in the registers crate. The final advantage is that by not using global static muts, we don't have this potentially unsound thing of taking multiple references to them.
I tried a couple of other approaches, but this seems like the best one. 
One question is whether boards are responsible for mapping interrupts to drivers. Something I looked into last night, a macro for each chip that creates a mapping. There's a default one, you only create it if you want to exclude some drivers. 
I have a PR that does all of this.
- Phil: I think letting boards configure what drivers they use is a big win.
- Brad: The duplication here is really significant.
- Hudson: If you are fine with the status quo, you can use the default macro which will exist for each chip, which is what we would do for all the upstream boards
- Amit: If I were building a product, e.g., with the nrf52, I would start with the macros, then once I knew exactly what I needed reimplement with my specific drivers
- Hudson: Exactly. The alternative is to change chips, which seems exclusively worse, especially from the perspective of supporting out-of-tree boards like OpenTitan.
- Hudson: I will post a PR implementing this for 1 or 2 chips and we can reach agreement on a design. Then we can go from there.
