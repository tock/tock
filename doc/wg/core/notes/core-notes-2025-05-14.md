# Tock Meeting Notes 2025-05-14

## Attendees
 - Branden Ghena
 - Brad Campbell
 - Alexandru Radovici
 - Hudson Ayers
 - Leon Schuermann
 - Pat Pannuto
 - Tyler Potyondy
 - Kat Fox
 - Johnathan Van Why
 - Amit Levy


## Updates
### x86 Support
 * Branden: x86 PR is officially merged!
 * Alex: We added a patch to stop QEMU on panic rather than having to kill it
 * Brad: That should be easy to merge
### Async Support
 * https://github.com/tock/tock/pull/4432
 * Alex: Submitted a draft PR to allow us to use await and async in kernel code. Purpose is twofold. Allows downstream users to pull in external drivers and use them. Secondly, we could support writing some drivers in async to handle complex state machines. It doesn't do dynamic allocation, just one future per driver so far. It requires an unstable nightly feature, impl_trait_in_assoc_type. You need to name the type of the future, which the compiler knows about but can't name. Normally you'd box it on the heap, but embassy copies into a bump allocator instead. With this feature, it works without dynamic allocation at compile time instead. So this PR will add an executor that can run these futures
 * Brad: Are you thinking of drivers for specific sensors?
 * Alex: Yes, but hardware-independent. Some generic screen, for instance.
 * Brad: Okay, so like an external IC driver from the embedded HAL. But we'd need something to map the embedded HAL onto hardware somehow?
 * Alex: Yes. I implemented some basic traits so far, but we'd have to add more. Those would be within Tock.
 * Brad: So those would map from the embedded HAL to our HILs.
 * Alex: Yes. We can't use the synchronous ones as they would block the kernel. But we can do the async ones. So a downstream user could pull in a driver from crates.io if it was async.


## Capability Creation Policy
 * Amit: We have discussed this before, but I want a more clear plan on how to handle it
 * https://github.com/tock/tock/pull/4418
 * Amit: At the moment, we implicitly creation capabilities primarily in components, which hides them from boards. A couple of weeks ago, we decided that we don't want that and they should be in top-level boards. However, this also exposed that the macro we created as a helper to make capabilities allows creation without unsafe, which was not the design intent. Finally, Leon highlighted a prior discussion from Slack that we also rely exclusively on visibility of the type to protect other parts of the system from creating them. Generally, they're zero-sized types with no constructor. So, if you can name it at all you can create it.
 * Amit: So, overall, at least three big issues with capabilities that we want to resolve. So questions are should we resolve them, how, and who will do it?
 * Amit: For the first, the grant capability is ubiquitous but we have a plan for it. Do we have consensus for removing capability creation from components?
 * Brad: Yes. That makes sense. We should not allow _any_ unsafe in components if possible.
 * Amit: Don't we have to? Components have to do some unsafe stuff for simultaneously binding two parts of a stack together
 * Brad: Yeah, maybe. If we're using unsafe now, it's NOT apparent though if you do a grep
 * Leon: We use maybeuninit there, which makes it a safe operation for binding
 * Amit: Yeah, looks like mostly capabilities and something in sequential loader that should be looked into and debug writer
 * Amit: Okay, so we could handle those and move all unsafe out of components, including capability creation.
 * Amit: Brad brought up grant capability being ubiquitous. I went through this and did it in boards. It was easy although a bit more verbose in the boards
 * Brad: I was thinking that not just the creation of capabilities but the use should also be exposed to the boards. But if we're just doing creation, that's straightforward. And I no longer think that exposing the use is required. It's a different, separate change.
 * Amit: What's meant by "the use"?
 * Brad: Well, you passed in a capability, but it doesn't tell you why or what capsule that's passed to. So we could totally ban capabilities from components which would make things more visible, but it's a big cost and would probably be the wrong tradeoff
 * Amit: I agree with that
 * Amit: So, second semi-rhetorical. Is the current version of the capability macro okay? I think it's not since anyone can use it to create a capability at any time it turns out. It's a bit of a blunder in the semantics of calling macros. I have a fix that's relatively straightforward which declares an inner unsafe function and calling it in a macro. Small differences in where Rust cares about the caller requiring unsafe or not.
 * Amit: One additional question here is whether we should specifically in the context of passing capabilities into components if we should make it easy to mint a type with multiple capabilities. We don't currently do that. We do one at a time with a macro, which might mean you have to pass a bunch into a component. As opposed to one with all the necessary permissions. Do we want this or not?
 * Leon: I'm confused. If I had two capabilities obtained from different interfaces, and if a function wanted a struct with two types, I couldn't combine my two capabilities into that new type
 * Amit: It wouldn't mean that for sure. But it would be a bummer if we couldn't translate back and forth between one capability with a bunch of permissions or a bunch of separate permissions in separate capabilities.
 * Amit: I think it would be safe and not hard to have default implementations for any set of capabilities where you could make a tuple that unions them. That seems okay
 * Leon: Okay. So a larger point. The current way we do capabilities is problematic. The constructor is one part and it's good to fix that. Will that require changing call sites of the macro?
 * Amit: Double-checking. The answer is no. The macro just takes a single capability name right now, but the update would allow one or more. Is this desirable?
 * Leon: Maybe we need an example of it. I'm not sure I follow.
 * Brad: Wait, on components still. Should I just make one uber-capability at the top of main.rs and pass it into all components? That's the path of least resistance, but I think we don't want that.
 * Amit: Well, there's what you're able to do and also what we encourage. Regardless of how they're passed into components, you could always create a capability that has everything and pass it in multiple times if the component needs two separate capabilities
 * Amit: So, main.rs files could choose to create a new capability for each requested capability, or they could create some merged super-capability thing. Those are all possible.
 * Amit: So what's the part to still discuss here?
 * Brad: I would like to see a policy, if we can't enforce it more strongly, where the component takes in capabilities, but we enforce that each capability is separate for each underlying capsule that needs that capability. So if the component makes two capsules each of which needs one capability, you would have to pass in two capabilities.
 * Branden: What if they needed the same capability?
 * Amit: You could have to pass it in twice
 * Amit: Okay, so components are sugar. They shouldn't obscure that there are multiple underlying capsules that may require multiple capabilities. Two with the same. Two with two different. Anything. So it's a reasonable policy that you have a separate parameter at least for each underlying capability. So the author of main.rs can see that there really are a bunch of things asking for ProcessCreation, for instance.
 * Amit: So on the creation side, we do need to limit who can do this
 * Hudson: For what it's worth, it's not possible everywhere. If you call create_capability in capsules it fails.
 * Amit: Wait, I tried it too and it worked. One of us is wrong.
 * Amit: Should capability creation require explicit unsafe? (YES)
 * Amit: Okay, so if that's not the case right now then we should fix that.
 * Brad: Wait. Paging something back in. When we created this, we wanted to avoid unsafe right? Because we wanted greps to only demonstrate type safety, not other things. We want the compiler to enforce it, but we want it to not look the same as memory unsafety
 * Amit: Since main.rs is unsafe anyways you don't need to explicitly use unsafe
 * Leon: That changed in Rust 2024. You still need explicit unsafe blocks in an unsafe function
 * Amit: If a macro writes unsafe for you, you don't have to write unsafe. Which is why create_capability can be used in non-unsafe contexts
 * Pat: If you are in a crate that forbids unsafe, it'll block that though
 * Amit: But you don't have to explicitly use unsafe. If you're not already in an unsafe context it will create it for you
 * Pat: Okay, so it still won't work in capsules
 * Amit: For Rust 2024 it would mean wrapping every create capability call in unsafe
 * Leon: I think this is the right thing. An explicit, visible unsafe that something dangerous is happening here
 * Brad: I disagree with that. These are cases that have to do with Tock correctness not Rust unsoundness. Starting processes isn't unsound, it just shouldn't be called from any place
 * Leon: Okay, I'll take that back
 * Branden: I would like to not have the unsafe keyword visible obviously for capabilities
 * Pat: Is there a way to limit to constructor for making capabilities to Board files?
 * Alex: We have a zero-sized type which is unsafe, and you'd only need to wrap 
 * Alex: Instead of capabilities as unsafe, we have capabilities which are safe but must be constructed through a function. That function takes in a zero-sized unsafe type. You can create capabilities in safe code, except for the creation of a token that they have to take
 * Amit: So a capability creation capability where that requires unsafe. That super capability would need unsafe that's not Rust unsafe to tie the knot
 * Alex: Yes
 * Amit: Okay, so in terms of the philosophy of not repurposing unsafe to do things that aren't related to Rust soundness, we only violate that in one specific place. And we wouldn't pollute unsafe everywhere. main.rs would have one unsafe block to crate the super-capability.
 * Brad: Seems a bit silly. Makes it too easy to pass around this super-capability to anyone.
 * Amit: A capsule should never request the super-capability
 * Brad: But it could also choose to request the super-capability and break everything
 * Amit: Well even today it could request a union of all capabilities
 * Brad: But that's more work, not less
 * Johnathan: If you add a lifetime to this super-capability that's non-static it would defend against that
 * Leon: But even with a non-static lifetime, it could be passed into a capsule which then makes everything internally
 * Branden: Yeah, we've recreated the component problem we're solving
 * Leon: So what I'm getting hung up on is that a super-capability could have the same problem where you hide visibility of which capabilities they need
 * Amit: Well, that's not the plan. The plan would be to mint all capabilities in main.rs and not let this bleed into other places
 * Pat: So things that user the super-capability should consume it
 * Leon: What prevents components from taking in the super-capability?
 * Brad: Convention and code review
 * Amit: I think a convention is enough there.
 * Brad: What we have now seems better than this super-capability idea. Right now we can deny unsafe to capsules which stops capability creation there. I'm worried that downstream developer will shoot themselves in the foot by passing super-capabilities around as the easy route. Where right now we're more constrained and stops misinterpretations from happening
 * Alex: They could write code now requesting unnecessary things
 * Brad: But that's not how people do things. Makes code harder not easier
 * Alex: We could make it dynamic where capability creation panics if done after init time
 * Amit: To the extend that it's in conflict with avoiding repurposing unsafe for things that don't relate to Rust soundness, that's the goal of super-capabilities. If that's not as much of an issue for us, the current state is better.
 * Amit: So we forbid unsafe in components, and you need unsafe for a capability, but at least it's limited to boards
 * Leon: I was also making a separate point that capabilities can never be a proxy for Rust soundness.
 * Amit: Which I think we agree with, although we should watch out for inadvertent cases. Process stuff can relate to soundness for example. But that shouldn't allow a capsule to mess with soundness.
 * Leon: Second thing.
 * Leon: https://github.com/tock/tock/pull/3409
 * Leon: There's been this PR from a few years ago for updating capabilities which focuses on the visibility of the type being enough to implement them. The PR says we should make capabilities types not traits. So you hold a specific type. Right now we have traits which leads to a layer of indirection in implementing it. We also want to handle availability of these types and creating them. We could make trait creation an unsafe operation, but we could just make the full transition to types that we previously postponed because of too much churn.
 * Amit: Action items. Fix capability creation macro if it's broken now.
 * Leon: We also have a problem that even if capability create macro requires unsafe. You could create copies of that type without unsafe
 * Amit: Yes, we should fix that too. Easy fix to add a non-explicit constructor.
 * Amit: Then remove capabilities from components.
 * Amit: Remove unsafe from components and mark them as forbid unsafe
 * Amit: I propose we fix capability creation and remove them from components. And separate deal with other unsafe in components
 * Brad: I'll work on the unsafe part
 * Amit: I'll work on the rest



