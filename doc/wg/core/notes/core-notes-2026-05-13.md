# Tock Meeting Notes 2026-05-13

## Attendees
 - Branden Ghena
 - Leon Schuermann
 - Johnathan Van Why
 - Alexandru Radovici
 - Brad Campbell
 - Pat Pannuto
 - Amit Levy


## Updates
### Tock-Registers
 * Johnathan: Fleshing out design in PR for Tock registers. We're at a state where things could be merged soon. So I'm considering whether any features we need might have design considerations on whether we change the implementation. DMACell stuff for instance.
 * Brad: Does this change require unsafe at all?
 * Johnathan: There's a lot of unsafe in the PR.
###
 * Brad: I was talking to someone last week working at a startup on supply chain code tracking for companies. They vet packages that developers use and track them. They were looking at entirely the development surrounding the open source package: where are the contributors, who's managing it, and not at all at the source code itself. I'm interested to see their takes on our code
 * Branden: Medical was like that when we talked to a company. Cared about the process.
 * Johnathan: And aviation
 * Alex: And automotive


## Unsafe in Chips
 * https://github.com/tock/tock/pull/4626
 * Brad: This was on the back-burner for a while because DMA needed a solution. Now that we have a DMA solution, I picked this up again. It moves things into two crates, one of which is safe and one of which is unsafe. Curious of thoughts on this.
 * Leon: We discussed this briefly in the Matrix channel. I think this is an enormous step in the right direction. There are details to hash out about how we pass StaticRef around. The safe crate probably has access to references at some point, which defeat the purpose just a little. The StaticRef of the AES registers.
 * Brad: That is true, but because the registers are defined in the unsafe crate, which the DMA implementation needs. The only ones that are marked public are the non-DMA registers.
 * Leon: Got it. That's better. I'm not super convinced that there is no other way to use those registers to screw up DMA in a way that endangers soundness. Ultimately it would be nice to not have visibility limiting this, but rather only expose a register manager which doesn't allow bad actions. I am happy to push that off to a follow-up PR. Maybe as part of a Tock registers update. We really want register operations to be safe or unsafe, which catches actions better
 * Amit: I semi-agree. Visibility is actually exactly a good mechanism by which on encapsulates and choose operations to expose. Marking things safe/unsafe would be just as good or better.
 * Amit: I think we should consider what the goal of this is and what the boundary is. I think a goal would be to take all the stuff that's fundamentally unsafe about interacting with the chip, such as modeling the register map, and encapsulating that into a separate crate which is allowed to use unsafe. Then everything using that crate ought to be safe Rust: could break correctness but not soundness. That seems compatible with there being a lot of unsafe and complexity in the crate. That requires the author to consider deeply about actions registers can take and possible side-effects.
 * Leon: I agree with all of that. I think visibility isn't as explicit as I'd like, but it's fine.
 * Leon: One thing still concerning is that we could have multiple instances of register managers interacting on the raw registers since we're passing around a StaticRef. And we can't reason about that. So we should make the new function on the register manager unsafe, as we need the ensure there's only one of them.
 * Amit: Seems reasonable.
 * Amit: On a high level, there seems to be a clear path for this. The split seems fundamentally reasonable.
 * Leon: I also think this really neatly comes together. The solution we have for DMA right now slots in very neatly into the structure we envisioned.
 * Amit: Johnathan, did you get a chance to think about how this will play with Tock-registers?
 * Johnathan: I can take another look. I didn't realize this was mostly in agreement and stable at this point.
 * Amit: I imagine this is a place where you'd use tock-registers in the unsafe crate.
 * Johnathan: Define register layouts in the unsafe crate and call the new function there.
 * Amit: Right. The procedural macros would live in the unsafe crate.
 * Johnathan: But, if you actually have registers that can cause undefined behavior, those would be marked as unsafe. The safe crate could see them, but wouldn't be able to operate on them.
 * Brad: Okay, I can prototype what it looks like to make the construction of the manager unsafe.
 * Leon: I sent an idea in the Matrix channel, which I think should be straightforward to apply
 * Brad: Are we good with this being the first example?
 * Amit: Yes
 * Leon: So AES is the only thing after this PR which still uses DMA?
 * Brad: Yes.


## Init in the Chip Trait
 * https://github.com/tock/tock/pull/4682
 * Brad: Also from last year. This lets chips do errata or other initialization. Many chips have something that essentially does this now, but it's informal.
 * Brad: The history here was considering rolling the deferred call setup into this. If we move that to the chip, since it's architecture specific, that would work but it needs to be initialized right away. But it's a little strange since it's a kernel tool that was going into chips. So I removed that from here.
 * Brad: So, is this a good thing, or do we not need to bother?
 * Amit: I think it's good, although without the deferred call it's less obvious why it's good. In my mind, this would help significantly to eventually make board definitions more boilerplate and ergonomic, but that's an in-the-future sort of thing
 * Brad: Agreed
 * Amit: I don't really see downside except for some API churn
 * Amit: Any thoughts on having the deferred call init inside this? Do we think it might belong or not?
 * Branden: What's the state of initializing deferred calls?
 * Brad: It's still "you better do it or your board won't work"
 * Branden: Oh, okay.
 * Brad: It's also better to have init as a function so we can think about whether it should be safe or unsafe
 * Pat: Could long term fix be a kernel init that takes a reference to the chip, can do deferred call init then call chip init?
 * Alex: I'm thinking about real-time support, and I would need the chip type in the kernel before the kernel loop. I'll probably also need an init in the chip. I'll need to process interrupts better to reduce latency
 * Brad: To Pat's point, I think that seems viable. That would resolve having access to the architecture, maybe? Maybe not? I'd have to think through it more. If we wanted to roll the chip init in there, we'd need this API.
 * Brad: So other than issue of churn, I'm not hearing any concern.
 * Amit: I agree
 * Branden: I think it's good


## Rust Nightly Update
 * https://github.com/tock/tock/pull/4801
 * Brad: Want to merge this
 * Amit: Any problems to discuss? Or just lingering?
 * Brad: There was a RISC-V issue, but we fixed that. And a flux issue, which is fixed. Just mechanical changes now.
 * Brad: They did stabilize the config-include thing we use, so that's nice. Plus clippy updates.
 * Leon: Merged


## Virtualization Priority Algorithms
 * https://github.com/tock/tock/pull/4802
 * https://github.com/tock/tock/pull/4818
 * Brad: Right now we just use insertion order in the mux as our priority. These PRs change that and improve it. The discussion point is how much configuration/standardization do we want? Do we want each virtualizer to determine for itself? How easy should it be for someone to look at a virtualizer and see how it works? Should boards be able to configure how virtualizers work?
 * Alex: We also decided we should just support round-robin by default for virtualizers. The issue is that this breaks CI because print statements when Tock starts are reordered. So, 4818 should be a small change, but needs to fix CI too
 * Leon: I can spend a bit looking at CI to fix it before merging. We could also merge ignoring CI
 * Branden: No way, we have to fix the tests to merge this
 * Alex: But the CI and how the virtualizer works have to match. So if it's changeable, that could be annoying
 * Leon: I think a flush utility is all we need. Sometimes we do want to guarantee ordering.
 * Alex: I think it's the opposite. You need to prevent a flush, as the debug makes two requests to the underlying mux, which are being split up
 * Brad: We definitely have to fix CI, and a big group discussion might not help here.
 * Brad: For a group discussion, I think we should just focus on how we want virtualizers to evolve. One issue is that it's hard to prove to yourself that the code is actually correctly implement round-robin
 * Alex: We need some use cases where insertion-first is a big problem. We need some alternative mechanisms.
 * Johnathan: Could be earliest-deadline first for instance in real-time systems.
 * Brad: I'm not hearing a lot of strong opinions. I'm fine if we can create a round-robin tool that we can wrap around an iterator list of devices and we can just call "next" in virtualizers to make it work. I also kind of like the first version where this algorithm can be parameterized. Maybe the component could have a default?
 * Johnathan: Strong opinion from me is that boards should be able to choose the scheduling algorithm.
 * Branden: A round-robin tool would be a step towards that. If you could make that, you could make another and parameterize it.
 * Brad: Yeah, the round-robin policy is the abstract tool here.
 * Branden: I thought we decided #4818 was a good first step. Then #4802 is the long-term follow-up.
 * Brad: I think it was a hesitation about whether we wanted parameterized algorithms
 * Branden: I'd support boards being able to choose. Maybe the only option today is round-robin. Seems valuable
 * Brad: I agree with that.
 * Brad: Let's talk about the selection policy trait then. Right now it just has a ready function
 * Alex: The design has a current function called select, which asks each item in the iterator if it's ready or not. The element is known, so it could also choose based on some quality of the element. But then the ready function might not be useful at all. You could have some policy which knows each element in the list. The iterator is clone, which is a requirement
 * Brad: So the policy would have to keep track of its own metadata about the list
 * Alex: Yeah, that was the idea. But select could do the thing ready does. But the iterator is clone, so the user could iterate several times to make a choice. Actually, ready isn't completely useless, as you could have a generic policy as well. You could make it specific or generic, and when you define the policy you implement the ready function, so you could split logic from readiness.
 * Brad: But then if ready does more than tell you if it's ready or not, then the virtualizer is choosing the policy, in part.
 * Alex: What we have right now is a round-robin policy that keeps state to do it in a round-robin fashion without knowing the item. If you implement something that knows the item then ready would be useless. The alternative is to ask items whether they're ready or not, so you could just call ready on every item. Then you could create specific policies that don't use this.
 * Brad: But I'm wondering if we would also want to have some additional info, either from the current ready function or from the trait, where each element in the list could tell you something else. So that there's some standardized way to track metadata for the policy. Otherwise if you want to track metadata you have to do it yourself. So you'd have copies of each of these that are different for UART and Timers and whatever.
 * Alex: So what's the set of features that's common between them?
 * Brad: So what if they just provided a usize, which could be anything. Some kind of ID or label. But I guess you could just use order in list instead
 * Alex: That's easy actually. Round robin already uses position in list.
 * Brad: I'm still unclear about what metadata is needed, so I'm inclined to leave this PR as-is right now
 * Alex: I think we should make ready a trait and implement it for the virtual devices, rather than have it as a closure. Then you don't have to pay for things you don't use. Well, maybe everything does care
 * Brad: I kind of like the way it is now. No extra trait needed.
 * Alex: So the second question, do we keep the insertion-first policy as a default?
 * Branden: I think no. I think moving to round robin is the best improvement here. The parameterization is a bonus
 * Alex: So move to Round Robin, fix the CI, and parameterize things. (agreement)


## Resolve Dependencies Issues
 * https://github.com/tock/tock/pull/4811
 * https://github.com/tock/tock/pull/4824
 * Brad: We have these default peripherals to make our lives easier in boards. But when we want one peripheral to have a static reference to another peripheral, this sort-of undoes that. So we really need a better way to create static peripherals, make other peripherals statically, then give those later peripherals references to the first peripherals.
 * Alex: It's completely wild-west right now, and every chip does it differently. We pay a cost for the current design even if we don't use it.
 * Brad: We knew that, but you could do it yourself to reduce that
 * Alex: You have to handle interrupts yourself though
 * Brad: 4824 is a simpler example. You can see the board has a static init for the default peripherals, then you also have to init some others outside to make the static refs work.
 * Alex: I approve both of these PRs.

