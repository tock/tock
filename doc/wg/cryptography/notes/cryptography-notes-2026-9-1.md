# Tock Cryptography WG Meeting Notes

**Date:** 9-1-26

**Participants:**
  - Amit Levy
  - Hans Martin
  - Roy Bachynskyi
  - Alex
  - Tyler Potyondy

## Modular Arithmetic
- Alex: The idea is to setup a chain of operations for the operations that need to be performed and then pass the input into this chain.
- Alex: This uses the data movement pattern used by digest and aes. 
- Alex: If you only implement addition/subtraction, you then can only implement the traits that are supported.
- Amit: And these are async operations?
- Alex: Yes, essentially, you chain these together and you start an operation and the driver then notifies the client when it is finished.
- Amit: And this is only an accelerator for a modular arithmetic?
- Alex: Yes.
- Amit: I could imagine an accelerator for other modes, I'm not familiar with the hardware here. Is this accelerator usually decoupled from other crypto accelerators?
- Alex: Typically yes. It seems like it would be strange to have a HIL for just one specific algorithm. 
- Alex: More concretely, for ecdsa was implemented in software previously. For this you need HMAC + some elliptic curve calculations. The idea for this would then be to still use HMAC, but leave the modular arithmetic.
- Amit: The point is that the hardware provides acceleration for ECC and maybe HMAC but it does not provide ECDSA end to end. There is acceleration for modular arithmetic.
- Alex: Yes, you would need all of the peripherals and they are separate.
- Tyler: One other thought, we may want to look into some other hardware that supports this modular arithmetic to confirm that the propsed HIL is in fact hardware agnostic.
- Alex: I can look into this.
- Amit: The main parts here then are the start_chain and start_operation functionality. What's the difference?
- Alex: You first call start_chain and chain together the needed operations (setup) then after this you call start_operation.
- Amit: So start_chain is beginning a new declaration of a formula?
- Alex: Yes.
- Amit: And start operation is saying, I'm done declaring the chain so now "go"?
- Alex: Yes.
- Amit: Do the chained operations correspond to hardware operations?
- Alex: Not necessarily, but this helps in my opinion to organize.
- Amit: So this isn't clearing the previous chain?
- Alex: Not necessarily.
- Amit: So when does a chain get cleared?
- Alex: When the operation ends.
- Amit: Is the chain operation stored in the driver or hardware?
- Alex: There isn't support for adding a chain into hardware and this is stored in the driver. The motivation for this is to allow a sense of memory to the equation to avoid as many fn calls / returns for each sub operation in the larger equation. 
- Amit: So the chain is an abstraction that lives in software. The one issue though is that if this trait is implemented for hardware, then we have to reimplement this abstraction for every piece of hardware.
- Amit: And presumably there would need to be some storage in the driver for this chain and this is limited to some reasonable ammount?
- Alex: Yes, and this is somewhat necessary to keep the order of operations intact. 
- Amit: And we don't store the operands in memory with the chaining?
- Alex: We do store them in memory, but not in the chain itself.
- Amit: In practice with this interface, it looks like even though you have some declarative chaining, you still need to reproduce the state machine for the formula.
- Amit: It is looking to me like that if you got rid of the chaining and just exposed what the hardware provides (e.g., do modular addition or subtraction) then I don't think this would look much more complicated then it is since you need to track the state machine anyways with the chaining.
- Amit: I'm guessing you have some static buffer that is arbitrarily sized for some length of the chaining. I can see a version where getting rid of the logic can be declared up front and the driver acts independently. 
- Amit: I wouldn't be surprised if replacing this with something closer to the hardware would not make the chip impl of the trait much more complex?
- Tyler: Why is that?
- Amit: It seems like the state machine to setup the chain and this complexity ends up in the client code itself.

## Digest HIL
- Roy: I went ahead and wrote another capsule and am doing some libtock-c capsules. I haven't had a chance to take a look at opentitan. I also noticed that there is an i2c chip that does some hashing it seems.
- Roy: I will keep working on this and have more updates in coming weeks. 
- Amit: I think this component was the atec508. I think that is a fairly reasonably common hardware example. 
