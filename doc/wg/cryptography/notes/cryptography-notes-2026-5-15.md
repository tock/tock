# Tock Cryptography WG Meeting Notes

**Date:** 5-15-26 

**Participants:**
  - Tyler Potyondy
  - Bobby Reynolds

## Crypto HIL Redesign
- Bobby: The new HIL is more callback oriented. I added Ecb/Cbc first, but realized the tutorial we are working on needs Ctr.
- Bobby: For Ctr, mode of operation is usually 16byte IV that is part nonce part counter. The question is what happens when the counter overflows.
- Bobby: In many cases this is hardware dependent.
- Bobby: One outcome when it wraps it carries the one to the nonce, another it does not carry the one to the nonce, the final case is the hardware faults. 
- Bobby: The hardware however is variable in the counter length, so this makes our HIL hard to think about.
- Bobby: Current approach I'm using is to encode the counter length when declaring Ctr in the Mode enum.
- Tyler: And how do we treat this hardware wrap case/specify this behavior?
- Bobby: The HIL trait will return an error if an operation results in a counter overflow. 
- Bobby: This is always implementable (in hardware or software), granted this is an artificial guardrail and if someone intended for an overflow to occur. This HIL would be incompatible. However, in my opinion, intending for the overflow to occur is not a sound design.
- Tyler: So just to clarify, what exactly happens when the counter wraps, does it clear the nonce or just increment the next byte?
- Bobby: Treats IV as one big counter so it is addition with carry operation.
- Tyler: Okay, so the current design you are proposing is to return an error when the counter wraps. This would be done by hardware if hardware supports this and would need to be a software check the implementer of the HIL does in the case of hardware not supporting it. 
- Bobby: We could implement such a check in a capsule for the incoming buffer to see if it would overflow. This would be done by dividing the length by the block size to preemptively determine if the conter will overflow.
- Tyler: This seems nice from an avoiding wasted work perspective. I like this.
- Bobby: For our crypt/read_input methods, do we think we should specify a length field?
- Tyler: And the benefit of this is that we avoid unnecessary work?
- Bobby: If we do not have an upfront length parameter, then we potentially fail half way. Downside of having length is not having streaming option.
- Tyler: Somewhat unrelated, but I'm not sure if I understand the purpose of the read_input method.
- Bobby: The chip specific driver implements the crypt method that specifies how to perform the crypto operation. The client trait is then implemented by something like a capsule and the driver then can call the read methods to obtain information such as the buffer, key, iv etc. We also expose methods so that the driver can write/update the IV that is stored in the capsule to support streaming operations.
- Tyler: I like this.
- Tyler: I have two thoughts on the current design. The first is that we should potentially make the mode enum a trait and then encode the counter value as a const generic.
- Bobby: Counter length may not be a runtime property. It is relevant from the client's perspective to know what the wrapping value is. 
- Bobby: Concretely, it is important to track when the wrap occurs because we need to then generate a new nonce when this occurs.
- Tyler: Okay, that makes sense. The enum makes sense then.
- Tyler: My other concern and question is how this design would play with software crypto. Specifically when the software crypto is built ontop of an underlying hardware crypto. 
- Tyler: For the nrf board, we only have ECB (technically also have CCM for bluetooth, but we will ignore that for now). The existing HIL got very messy since we roll our own Ctr crypto built ontop of ECB and then we have the Ctr implementation implement the HIL.
- Tyler: Since HIL stands for "Hardware Interface Layer" this seems to be unideal and seems in my opinion to be an incorrect abstraction.
- Tyler: I proposed in some of the earlier discussions introducing a "Logical Interface Layer" for cases like this. 
- Tyler: We then would have some software crypto that is perhaps an external dependency (like the rust crypto crates) that implements this LIL. On the nrf, CTR is something we are rolling ourselves built on ECB. 
- Bobby: A few thoughts, from a compliance perspective, it is hard to ship a product with a software implementation that you have rolled. Internally at microsoft, we have a policy disallowing rolling our own crypto. That said, there is value in some use cases to have in a software implementation of this, just likely not in enterprise/shipping products.
- Bobby: An example of how to layer a software crypto is a struct generic over some "A" that is an implementation that then does HW crypto for supported but then uses SW for unsupported. 
- Tyler: What you are describing is very similar to my proposal. 
- Tyler: Out of curiosity, are there any Microsoft approved Rust crypto libraries? 
- Bobby: The approved Microsoft sw crypto libraries are in C. 
- Bobby: What we currently do is sw crypto lives in userspace and uses IPC. Otherwise, we would need to use an FFI in the kernel.
- Tyler: One other challenge I saw with the Rust crypto libraries is using an ECB block cypher for their CTR implementation. The cypher function using the hardware is blocking which doesn't play nicely with our kernel's execution model. 
- Tyler: Perhaps a cleaner solution, which we have partially kicked around is to exclude explicitly any software crypto from the kernel. One idea is having something equivalent to a "kernel process". For all intensive purposes, this would be identical to our standard userprocesses. The way I envision this working is to perhaps have a definition in the board file specifying the need for some software crypto, and if this is detected, we flash our crypto library process with the kernel. 
- Tyler: The downside here is overhead since we would be crossing the syscall boundary 4 times for any operation (app => kernel => crypto library app => kernel => app).
- Bobby: From my perspective, the overhead for this isn't too concerning with respect to how long crypto ops themselves will take.
- Bobby: This is fairly similar to what we are doing except we do it via IPC. Having a kernel supported mechanism for this would be very nice.
- Tyler: For our working example of the HOTP tutorial, we are going to run into this very soon since we only have ECB on our board but would need CTR. This will be a good motivating use case.

## TODOs:
- Bobby: Push working changes of HIL to branch, keep cleaning up impl
- Tyler: Begin implementing new HIL for nrf aes ctr
- Agenda for Next Time: Continue discussion of SW crypto built on HW primitives, kernel mediated design
