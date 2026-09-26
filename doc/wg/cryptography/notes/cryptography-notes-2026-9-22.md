# Tock Cryptography WG Meeting Notes

**Date:** 9-22-26

**Participants:**
  - Hans Martin
  - Alex
  - Tyler Potyondy
  - Irina Bradu
  - Bobby Reynolds

## Updates
- Hans: Bobby is working on the driver mutex PR and will be opening that soon. 
- Alex: We talked about this last week and it's going to be driver mutex + digest HIL as a usecase.
- Bobby: I have a pair of branches, one with the driver mutex and another with a usage of the driver mutex with digest operations. I took one of the designs we debated and flushed it out using the driver mutex + an example chip.
- Bobby: This should show across the whole stack what these changes would look like.
- Bobby: The current branch I have now has reference counting implemented, but this is something we talked about waiting to add. I'm going to remove this.

## Modular Arithmetic - https://github.com/theonlytruealex/Elliptic-Curves-Modular-Arithmetic
- Alex: With the feedback, I've simplified the interface further.
- Bobby: Can you walk through the Op generic parameter for the client trait?
- Alex: When you implement the driver, you provide the supported operations. These implement the corresponding traits for the supported operations. 
- Alex: When you perform the computation, you call `Op::addition()` so we can enforce via the type system that the client only calls operations that the driver supports.
- Bobby: This is an interesting way it seems to place some type safety around an enum. You have the concrete type that is the enum and then you have marker traits that map onto the enum variant. 
- Bobby: So this abstracts that the hardware you are interacting with must have an enum that has variants for specific operations.
- Bobby: I'm curious if there are other locations where the op generic adds additional type safety. It seems you can only call start computation via this type enforced interface. 
- Bobby: I wonder if there's a way to use the Op generic to make the client trait interface richer.
- Alex: It might help to open the PR for this and then get some feedback from the broader community.
- Tyler: Agree with Bobby. This looks great. It's an interesting pattern here to use the enum for the flexibility it provides but then to use the marker traits to keep the type safety angle/avoid the runtime errors we had before b/c of the enum.
- Tyler: Bobby, can you elaborate a bit more what you mean by a richer interface by using the Op generic / type enforcement for the Client methods?
- Bobby: So right now we have pub trait MathClient that are usable for all math operations. Imagine if we had something like addition client, multiplication client. I'm not sure if this is better, but it would be neat to have extra callbacks or methods that are operation dependent.
- Bobby: I'm not sure if this could be expressed in Rust, but it seems like there is maybe an opportunity for some type safety.
- Hans: Does computation completed have access to the Op type?
- Alex: Yes, since you might want to start another computation which requires Op.
- Bobby: One critique is that the MathClient trait doesn't need to be constrained by the Op generic.
- Alex: This is correct, we can remove it. 
- Hans: My only concern is how the Op identity might survive async completion. 
- Bobby: Hans is speculating if we changed the signature of computation completed to include the op of what we just completed. This would mean you would need to add back the Op as a generic parameter to the trait. 
- Bobby: There are cases where you are returned information from the operation that is completed. Other times this is implicit and the client must track this based on its internal state machine. I think Tock has both right. I lean towards the client should be self contained and should know which op it submitted earlier. But also, clients need to track less state if this is provided upon completion. 
- Alex: This is true but falls apart if you do something like two of the same operation since you need to be able to differentiate between two operations. 
- Hans: Does this two addition op happen? It is supported by hardware but this is a bit of a can of worms. 
- Bobby: In general, I think this feedback should be deferred to the PR where we can look at the whole PR.
- Hans: Just to confirm, I'm okay with this design.
- Alex: Next I can share what this looks like with the elliptic curve capsule/usecase looks like. 
- Tyler: I wonder if it might be most effective to push this discussion on the elliptic curve usecase to the PR for the HIL.
- Bobby: One comment, it might be useful to having a discussion about the syscall interface sync on a call.
- Tyler: Are there changes to the syscall interface in this PR for the elliptic curve capsule we have already?
- Alex: No, this just alters the backend to use hardware acceleration instead of software based elliptic curve.
- Bobby: Does this work rely on the driver mutex?
- Alex: Yes, for the capsule.
- Bobby: Alright, this will be another good motivating usecase for showing the driver mutex in action.

## PR https://github.com/tock/tock/pull/5168
- Tyler: This PR has been sitting for a couple weeks without comments. We should review async.
- Bobby: Quickly skimming, this looks like an AI generated PR based on changes / description. 
- Bobby: The changes seem good at face value, but it's worth a closer look to make sure altering the nonce length like this is doing doesn't have some unintended consequences elsewhere.

## Logistics
- All: updating cadence back to every-other week.
